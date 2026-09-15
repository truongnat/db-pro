# MySQL 8 provider — live fixture, measured decoder matrix, and the defects it exposed (#235)

- Session: MySQL 8 live-evidence pass, 2026-09-15 (evening)
- **Base head:** `main @ e1b2eab2` (worktree clean, remote in sync; coordinator-verified gates at that
  head: `fmt` PASS, `clippy` PASS, `cargo test --workspace` **932 passed / 0 failed / 35 ignored**)
- **Fixture:** `mysql:8.4` (server **8.4.11**), container `dbpro-v01-mysql-fixture`, host port
  **33306**, database `dbpro_fixture`, root password redacted here (`dbpro_test_root` in
  `docker-compose.yml`, the same disposable-test convention the PostgreSQL service already uses)
- **PostgreSQL fixture** (CI-mirror run only): `dbpro-v01-pg-fixture`, `postgres:16` (16.15), port
  55432 — started for the run and **stopped again** (it was stopped when the run began)
- **Outcome:** the five live MySQL tests ran for the first time and **2 of them failed**; the decoder
  and introspection defects they exposed were fixed and re-measured; the live half of #235's
  acceptance is now met with evidence. #235 stays **open**: criteria 2, 5 and the write half of 3 are
  untouched by this pass (per-criterion table in §8).
- No `qltx-*` container was touched. Nothing in `.github/workflows/` or `docs/goals/` was changed.

## 1. The fixture

`fixtures/mysql/` (new, committed) mirrors the PostgreSQL fixture's shape:

| File | Purpose |
|---|---|
| `001_schema.sql` | `decoder_matrix` (41 columns, one per class), `categories`, `order_items` (composite PK, FK, index), `order_item_audit`, view `v_order_item_totals`, trigger `order_items_after_insert`, function `order_item_total` |
| `002_seed.sql` | deterministic seed: row 1 carries a value for every class, row 2 is NULL in every nullable family |
| `003_verify.sql` | self-check; `SIGNAL SQLSTATE '45000'` on any mismatch (counts, trigger firing, the unicode row's exact bytes) |

`docker-compose.yml` gained **one new service** (`mysql`, port 33306, image `mysql:8.4`, the three
SQL files mounted into `/docker-entrypoint-initdb.d`, `--default-time-zone=+00:00`) and one new named
volume (`mysql_data`); the existing `postgres` service, its volume and its healthcheck are byte-identical
to before (`git diff docker-compose.yml` is additive only). The container this run used was started by
hand with the same parameters because port 33306 has to be free while the fixture runs:

```bash
docker run -d --name dbpro-v01-mysql-fixture \
  -e MYSQL_ROOT_PASSWORD=<pw> -e MYSQL_DATABASE=dbpro_fixture -p 33306:3306 \
  -v "$PWD/fixtures/mysql/001_schema.sql:/docker-entrypoint-initdb.d/001_schema.sql:ro" \
  -v "$PWD/fixtures/mysql/002_seed.sql:/docker-entrypoint-initdb.d/002_seed.sql:ro" \
  -v "$PWD/fixtures/mysql/003_verify.sql:/docker-entrypoint-initdb.d/003_verify.sql:ro" \
  mysql:8.4 --default-time-zone=+00:00
```

Load evidence (raw): the entrypoint log shows `001_schema.sql`, `002_seed.sql`, `003_verify.sql` each
`running` with **no `ERROR` line at all**, and re-running `003_verify.sql` against the live server
produces no output (the procedure's `SIGNAL` never fires) — i.e. counts, trigger, and the unicode
row's bytes are as declared. `SELECT @@version` through the provider reports **8.4.11**;
`@@session.time_zone` is **`+00:00`**.

**Fixture defect found and fixed during the run (not a provider defect).** The first load stored the
unicode row double-encoded: `HEX(label)` came back as `C383C5936E…`, the UTF-8 of the *mojibake*, because
the `mysql` client inside the server image does not default to `utf8mb4` and read the file's bytes as
latin1. Fixed by pinning the charset in the fixture itself (`SET NAMES utf8mb4;` in all three files),
with `003_verify.sql` now asserting `HEX(label) = C39C6EC3AF63C3B664C3A920E29C9320E697A5E69CACE8AA9E`
(the bytes of `Ünïcödé ✓ 日本語`) so the same mistake cannot pass again. After the fix: `HEX(label)` =
that value exactly.

## 2. The five live tests — first execution ever

```bash
DATABASE_URL=mysql://root:<pw>@127.0.0.1:33306/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test mysql_integration -- --ignored --test-threads=1
```

| | Before the fixes | After the fixes |
|---|---|---|
| Result | **FAILED**, exit 101 — **3 passed / 2 failed** | **ok**, exit 0 — **5 passed / 0 failed / 0 ignored** |
| `mysql_connects_and_executes_query` | ok | ok |
| `mysql_execute_returns_affected_rows` | ok | ok |
| `mysql_explain_returns_json` | ok | ok |
| `mysql_introspect_returns_canonical_model` | **FAILED** — `ColumnNotFound("table_name")` panic at `mysql/introspect.rs:28` | ok |
| `mysql_transaction_rolls_back_on_failure` | **FAILED** — `panicked at mysql_integration.rs:152: unexpected cell` | ok |

Raw output: `/tmp/dbpro-mysql-evidence/01-live-suite-before.txt` and `03-live-suite-after.txt`
(session-local; the assertions are reproduced by the committed tests). The failure was real and the
tests were **not** modified: both are untouched by this pass.

The second failure is the decoder defect in one line: the test reads
`SELECT count(*)` and expects `CellValue::Int64`; it got `CellValue::Bool(true)`.

## 3. The measured decoder matrix (the decisive table)

A temporary probe (deleted before the commit, per the run brief) selected every `decoder_matrix`
column **through the real provider path** (`MySqlConnector::query` → `MySqlQueryMapper::map_rows`) and
printed the resulting `CellValue` variant and payload, next to MySQL's own rendering of the same column
(`CAST(… AS CHAR)`, `+0` for the BIT columns, `HEX()` for the binary classes). Row 1 carries the
values, row 2 is NULL in every nullable family.

Reported type names are what the provider puts in `ColumnMeta.data_type`; they are the prepared-statement
(binary-protocol) shapes: the blob/text families collapse to `BLOB`/`TEXT`, and a `SET` column is
reported as `CHAR` (measured, not assumed).

| column | reported type | before | after | MySQL's own value (oracle) |
|---|---|---|---|---|
| `id` | SMALLINT | `Bool(true)` | `Int64(1)` | `1` |
| `flag` | BOOLEAN | `Bool(true)` | `Bool(true)` | `1` |
| `tiny_count` | TINYINT | `Bool(true)` | `Int64(-128)` | `-128` |
| `small_count` | SMALLINT | `Int64(-32768)` | `Int64(-32768)` | `-32768` |
| `medium_count` | MEDIUMINT | `Int64(8388607)` | `Int64(8388607)` | `8388607` |
| `count_value` | INT | `Int64(2147483647)` | `Int64(2147483647)` | `2147483647` |
| `big_count` | BIGINT | `Int64(9223372036854775807)` | `Int64(9223372036854775807)` | `9223372036854775807` |
| `tiny_unsigned` | TINYINT UNSIGNED | `Bool(true)` | `Int64(255)` | `255` |
| `int_unsigned` | INT UNSIGNED | `Bool(true)` | `Int64(4294967295)` | `4294967295` |
| `big_unsigned` | BIGINT UNSIGNED | `Bool(true)` | `Decimal("18446744073709551615")` | `18446744073709551615` |
| `ratio` | FLOAT | `Float64(1.5)` | `Float64(1.5)` | `1.5` |
| `precise_ratio` | DOUBLE | `Float64(0.1)` | `Float64(0.1)` | `0.1` |
| `amount` | DECIMAL(30,10) | `Text("unsupported")` | `Decimal("12345678901234567890.1234567890")` | `12345678901234567890.1234567890` |
| `scaled_amount` | DECIMAL(10,2) | `Text("unsupported")` | `Decimal("10.50")` | `10.50` |
| `calendar_date` | DATE | `Text("unsupported")` | `Date("2024-03-15")` | `2024-03-15` |
| `wall_time` | TIME(6) | `Text("unsupported")` | `Time("10:20:30.123456")` | `10:20:30.123456` |
| `long_time` | TIME(6) | `Text("unsupported")` | `Time("800:59:59.123456")` | `800:59:59.123456` |
| `negative_time` | TIME(6) | `Text("unsupported")` | `Time("-100:30:15.500000")` | `-100:30:15.500000` |
| `local_stamp` | DATETIME(6) | `Text("unsupported")` | `Timestamp("2024-03-15T10:20:30.123456")` | `2024-03-15 10:20:30.123456` |
| `instant` | TIMESTAMP(6) | `Text("unsupported")` | `TimestampTz("2024-03-15T10:20:30.123456Z")` | `2024-03-15 10:20:30.123456` |
| `year_value` | YEAR | `Text("unsupported")` | `Int64(2024)` | `2024` |
| `bit_one` | BIT | `Bool(true)` | `Int64(1)` | `1` |
| `bit_eight` | BIT | `Bool(true)` | `Int64(170)` | `170` |
| `bit_sixtyfour` | BIT | `Bool(true)` | `Decimal("18446744073709551615")` | `18446744073709551615` |
| `token` | CHAR(36) | `Text("a0eebc99-…-6bb9bd380a11")` | unchanged | same |
| `label` | VARCHAR | `Text("Ünïcödé ✓ 日本語")` | same variant | see §1 — the *stored* value was double-encoded until the fixture charset was pinned; the mapper served exactly what the database held |
| `binary_collation` | VARBINARY (`utf8mb4_bin`) | `Bytes(12, 62696E…)` | unchanged | `bin-collated` |
| `body`/`tiny_body`/`medium_body`/`long_body` | TEXT | `Text(…)` each | unchanged | same |
| `raw_bytes` | BINARY(8) | `Bytes(8, DEADBEEF00112233)` | unchanged | same |
| `raw_varbinary` | VARBINARY | `Bytes(4, FF00FE01)` | unchanged | same |
| `blob_data` | BLOB | `Bytes(4, FF00FE01)` | unchanged | same |
| `tiny_raw` | BLOB | `Bytes(5, 68656C6C6F)` | unchanged | `hello` |
| `long_raw` | LONGBLOB | `Bytes(64, AB…)` | unchanged | same |
| `status` | ENUM | `Text("shipped")` | unchanged | `shipped` |
| `tags` | SET reported as CHAR | `Text("alpha,gamma")` | unchanged | `alpha,gamma` |
| `doc` | JSON | `Text("unsupported")` | `Json({"a":1,"b":[true,null]})` | `{"a": 1, "b": [true, null]}` |
| `geo` | GEOMETRY | `Text("unsupported")` | `Bytes(25, 000000000101000000000000000000F03F0000000000000040)` (SRID + WKB POINT(1 2)) | `0000000001010000…` |
| `missing` | VARCHAR | `Null` | `Null` | `NULL` |
| every nullable column, row 2 | — | `Null` | `Null` | `NULL` |

## 4. Defects proven by that measurement

**D1 (P1) — every integer class was served as a boolean.** The old `extract_cell`
(`mysql/query_mapper.rs:42-61` before the change) probed `bool → i64 → f64 → String → Vec<u8>` and kept
the first type the driver would decode. The driver's `bool` decoder is
`i8::decode(value) != 0` (`sqlx-mysql 0.8.6`, `types/bool.rs:41`), and its `bool` compatibility list
accepts *every* integer column type and `BIT` — so any integer whose value fits in an `i8` became
`Bool(true/false)` (measured: `id`, `tiny_count`, all unsigned classes, all `BIT` classes), values
outside `i8` fell through to `Int64` (which is why `count_value`/`big_count` looked right), and the
driver's signed reinterpretation of an unsigned payload made `BIGINT UNSIGNED` = 18446744073709551615
arrive as `Bool(true)` (`LittleEndian::read_int` → `-1`). `SELECT count(*)` — the query every Explorer
path issues — was a boolean. This is what failed `mysql_transaction_rolls_back_on_failure` live.

**D2 (P1) — value-destroying fallback for six classes.** `DECIMAL`, `DATE`, `DATETIME`, `TIMESTAMP`,
`TIME`, `YEAR`, `JSON` and `GEOMETRY` are all incompatible with every type the chain probed, so
`extract_cell` returned the literal `CellValue::Text("unsupported")` — the value was **replaced**, not
rounded (measured for all of them; this confirms pass 6's code-path reading and corrects one detail:
`BIGINT UNSIGNED` did *not* land on that fallback, it landed on `Bool(true)` — the `bool` list has no
unsigned exclusion, unlike the `i64` list).

**D3 — `BIT(64)` was silently destroyed.** `BIT` was decoded through the `bool` probe, so any bit
pattern decoded by its first byte (`b'10101010'` → `true`, 64 ones → `true`). The width is not in the
reported name (`BIT` for all widths), so the class cannot be split into "the boolean-ish one" and "the
wide one" by a heuristic; the fix decodes the class as the unsigned integer it is (see §5).

**D4 — `information_schema` introspection could not run at all.** `Row::get("table_name")` matches a
label exactly, and MySQL 8 reports `information_schema` labels in **upper case** (`TABLE_NAME`), so the
first read panicked with `ColumnNotFound("table_name")` (`mysql/introspect.rs:28`). Measured live: the
introspection test failed on that panic. After the labels were aliased, the same path failed a second
time with `ColumnDecode … String is not compatible with SQL type VARBINARY`: MySQL 8 reports
`information_schema` *text* columns with the binary flag set, so the driver's compatibility gate
rejects `String` even though the payload is the column's UTF-8 text.

**D5 — every MySQL index lost its table.** Indexes were grouped by `index_name` alone and emitted with
`table_name: String::new()` (`mysql/introspect.rs:111,134` before the change). `SchemaService` filters
indexes by table name, so no MySQL index could ever be shown, and two same-named indexes in different
tables would have been merged into one. Measured after the fix: `idx_order_items_sku` arrives with
`table_name = order_items` and `columns = [sku]`, and the `PRIMARY` index with
`columns = [order_id, line_no]` (asserted by the committed live test; the before-state for this field is
the code fact above, since D4 blocked observing it live).

**Not defects (measured, deliberately unchanged):** `FLOAT`/`DOUBLE` → `Float64` was already correct;
`CHAR`/`VARCHAR`/`TEXT`/`ENUM` → `Text` and `BINARY`/`VARBINARY`/`BLOB` → `Bytes` were already correct
including the byte-exactness rule (`tiny_raw` holds ASCII `hello` and is still `Bytes`, not `Text`);
`NULL` in every family, including `DECIMAL`/`DATE`, was already `Null`.

**Known class-mapping limitation (recorded, not fixed).** A text column declared with a binary collation
(`VARCHAR(32) COLLATE utf8mb4_bin`, a common choice for case-sensitive keys) is reported by MySQL as
`VARBINARY`, so it is served byte-exact (read-only hex) rather than as text. The value is lossless; only
the class is stricter than the user's intent. Splitting these from true binary columns needs the
column's charset, which `sqlx-mysql` does not expose on a result column (`MySqlTypeInfo` carries only
type/flags/max_size), so a fix would have to sniff the payload — which the provider-value contract
forbids. Recorded here rather than worked around.

## 5. The fixes

`crates/infrastructure/src/mysql/query_mapper.rs` — `extract_cell` now dispatches on the class the
provider reports for the column, the way `postgres/query_mapper.rs` does, and arms cover **every** name
`MySqlTypeInfo::name()` can produce:

| class (reported name) | decoder | domain value |
|---|---|---|
| `BOOLEAN` (MySQL's only boolean shape) | `i8` | `Bool` |
| `TINYINT`…`BIGINT` | `i64` | `Int64` |
| `TINYINT UNSIGNED`…`INT UNSIGNED` | `u64` | `Int64` (all fit) |
| `BIGINT UNSIGNED`, `BIT` | `u64` | `Int64`, or `Decimal` above `i64::MAX` (exact digits) |
| `YEAR` | `u64` (ungated) | `Int64` |
| `FLOAT` / `DOUBLE` | `f32` / `f64` | `Float64` |
| `DECIMAL` | `BigDecimal` | `Decimal` (exact digits, declared scale kept) |
| `DATE` | `NaiveDate` | `Date` (`YYYY-MM-DD`) |
| `DATETIME` | `NaiveDateTime` | `Timestamp` — `%Y-%m-%dT%H:%M:%S%.6f`, **no marker** |
| `TIMESTAMP` | `DateTime<Utc>` | `TimestampTz` — `to_rfc3339_opts(Micros, true)`, `Z` |
| `TIME` | `MySqlTime` | `Time` — `HH:MM:SS.ffffff` inside a day, MySQL's own shape (±838 h, signed) outside it |
| `CHAR`/`VARCHAR`/`TEXT`/`TINYTEXT`/`MEDIUMTEXT`/`LONGTEXT`/`ENUM` | `String` | `Text` |
| `SET` | `String` (ungated) | `Text` (MySQL 8 reports `SET` as a string column; a server that reports the type is read the same way) |
| `BINARY`/`VARBINARY`/`BLOB`/`TINYBLOB`/`MEDIUMBLOB`/`LONGBLOB` | `Vec<u8>` | `Bytes` |
| `JSON` | `serde_json::Value` | `Json` |
| anything else (`GEOMETRY`, future classes) | raw payload | `Bytes` — byte-exact, never mojibake, never a failed query |

`get_ungated` (via the driver's public `Row::try_get_unchecked`) is used only where the driver's
compatibility gate keys on a flag the reported name does not carry (`BIT`, `YEAR`, `SET`): the *class*
is still established from the type name, so no decoding is skipped. 7 unit tests pin the pure parts
(`unsigned_integer_stays_exact_above_i64_max`, the three temporal formatters, the two `TIME` shape rules).

**Temporal semantics (#56's class of defect).** `DATETIME` keeps its wall-clock reading with no marker;
`TIMESTAMP` is normalized to UTC and marked. Measured both ways: with the server's *global* time zone
moved to `+07:00`, the provider's new connections report `@@session.time_zone = +00:00` (the driver pins
it) and `local_stamp`/`instant` decode to exactly the same values as before the shift — the DATETIME
never gains an offset and the TIMESTAMP never moves. The committed test asserts the session zone is
`+00:00`, which is the premise that makes the `TIMESTAMP` arm truthful; if that ever changes, the arm has
to change with it.

`crates/infrastructure/src/mysql/introspect.rs` — every `information_schema` column is aliased to the
lower-case label the module reads, text columns are read through one documented `info()` helper that
skips the driver's flag-based gate (the payload is still decoded by `T`'s decoder), and indexes are
grouped by `(table_name, index_name)` and emitted with their table.

## 6. Permanent coverage (new, committed)

`crates/infrastructure/tests/mysql_fixture_matrix.rs` — three live tests, **not** `#[ignore]`d, that
self-skip with a reason unless `DATABASE_URL` is a `mysql://` URL:

| test | what it pins |
|---|---|
| `mysql_fixture_decoder_matrix_covers_every_value_class` | all 41 matrix cells expected-vs-actual, the reported class names, the unicode round-trip, and the NULL row |
| `mysql_fixture_query_path_keeps_integers_and_routine_results_typed` | `count(*)` is `Int64`, the composite-key select, the stored function's result, a NULL quantity, and the UTC session zone |
| `mysql_fixture_introspection_reports_composite_key_foreign_key_and_indexes` | the composite PK's column order, `idx_order_items_sku` **with its table**, the FK's from/to, the view, the trigger (timing/event/table) and the routine |

Skip path (no `DATABASE_URL`): 3 passed, 0 failed, 0 ignored, 0.00 s. Live path
(`DATABASE_URL=mysql://…`, `--test-threads=1`): **3 passed / 0 failed**. Both live MySQL targets under
`--include-ignored`: `mysql_integration` 5 passed + `mysql_fixture_matrix` 3 passed = **8 passed / 0 failed**.

## 7. Gates at the change head

| Gate | Result | Raw |
|---|---|---|
| `cargo fmt --all -- --check` | PASS, exit 0 | no output after one `cargo fmt --all` (the first run listed 6 formatting diffs in the new code) |
| `cargo check --workspace` | PASS, exit 0 | `Finished dev profile in 3.27s` |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 | `Finished dev profile in 0.27s`, no warnings |
| `cargo test --workspace` | PASS, exit 0 | **942 passed / 0 failed / 35 ignored** (baseline **932 / 0 / 35** → **+10 passed**: 7 mapper unit tests + 3 fixture tests that self-skip without a MySQL URL) |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 | `target/release/db-pro-native`, 24,960,512 bytes |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial), exit 0 | 4 passed / 0 warnings / 0 failed / 4 not executed |
| `cargo test --all -- --include-ignored` (PostgreSQL `DATABASE_URL`, **both** fixtures up) | PASS, exit 0 | **977 passed / 0 failed / 0 ignored** (pass 6 recorded 967 / 0 / 0 → **+10**) |
| MySQL live targets, `--include-ignored` (MySQL `DATABASE_URL`) | PASS, exit 0 | 8 passed / 0 failed (5 + 3) |

The CI-mirror run uses the PostgreSQL `DATABASE_URL` because that is the command in the repo handover
(`HANDOVER.md:196`); with it, every MySQL live test **self-skips with a reason**, which is the behaviour
pass 6 introduced (`f2a36b4b`) and which this pass kept — the new file follows the same rule.

## 8. #235's acceptance, criterion by criterion (disposition: stay open)

| # | Criterion (issue text) | State after this pass |
|---|---|---|
| 1 | live MySQL 8 fixture connects and executes representative queries | **MET** — fixture committed, five live tests **5/5 green** (they were 3/5 before, and had never run anywhere), 3 further fixture tests green, service added to `docker-compose.yml`, raw output in §2 |
| 2 | schema Explorer and Query/Table workspaces reuse existing native UI | **NOT MET** — unchanged by this pass: the MySQL driver card is still `is_disabled: true` ("Soon", `crates/ui/src/connection_view.rs`), so no MySQL connection is reachable from the UI |
| 3 | DECIMAL/BIGINT/JSON/date-time values round-trip without silent precision loss | **HALF MET, and now proven** — the read half is fixed and measured (§3/§4: exact `Decimal` digits, exact unsigned integers, dedicated temporal variants, structured `JSON`); the **write** half is not implemented: `MySqlConnector::query`/`execute` take `_params` and there is no MySQL parameter binder, so a value cannot be sent back |
| 4 | composite PK/FK/index metadata represented correctly | **MET for the metadata this pass measured** — composite PK order, FK from/to, and every index now carries its table (D5). Remainder, recorded not fixed: `check_constraints` is still unconditionally empty (`mysql/introspect.rs`), so MySQL `CHECK` constraints are not introspected |
| 5 | table mutations use canonical safety/change-set flows | **NOT MET** — unchanged: `CompositeConnector::dialect` returns an error for MySQL (`crates/infrastructure/src/connector.rs`), so the table-data paths cannot run |
| 6 | service/provider/native UI/runtime tests | **PARTIAL** — provider-level live tests now exist and run against the fixture (§6); no service/UI/runtime test for MySQL exists and criterion 2's UI surface is still disabled |

Criteria 2, 5 and the write half of 3 are provider-wide capability work (a UI capability path, a binder
and dialect, a capability-level reason channel); they are not defeated by *fixing* anything measured, and
two of them are recorded against **#234** as well. They were left untouched on purpose — the run's scope
was the live-evidence gap and the decoder, and changing the capability surface would have meant changing
public interfaces.

**#234 (provider SDK acceptance gap): nothing in this pass resolves any of its recorded criteria**
(no capability path or de-branching, no capability-reason channel, no architecture document, and no
value/decoder case added to the conformance suite). #234 stays open, unchanged, with no new comment
beyond a pointer to this file.

## 9. Recommendation — what would make the live path run in CI (not done here)

The coordinator holds `.github/workflows/*`; nothing under it was changed. To make CI exercise the live
MySQL path, three things have to change together:

1. **A MySQL service in the CI job** (`.github/workflows/ci.yml`, the job that already declares the
   PostgreSQL service): `image: mysql:8.4` with `MYSQL_ROOT_PASSWORD`, `MYSQL_DATABASE: dbpro_fixture`,
   a `3306:3306` port map, an options string already carrying `--default-time-zone=+00:00`, and a
   `--health-cmd="mysqladmin ping -h 127.0.0.1 -uroot -p$MYSQL_ROOT_PASSWORD --silent"`
   `--health-interval=5s --health-timeout=3s --health-retries=10` trio.
2. **Fixture load as a step**, because the service knows nothing about `fixtures/mysql/`:
   `for f in fixtures/mysql/001_schema.sql fixtures/mysql/002_seed.sql fixtures/mysql/003_verify.sql;
   do mysql -h 127.0.0.1 -P 3306 -uroot -p"$MYSQL_ROOT_PASSWORD" dbpro_fixture < "$f"; done`, relying on
   `SET NAMES utf8mb4` inside the files for the charset. Do **not** use `--default-character-set=latin1`
   or a `LANG`-less shell without those statements: the double-encoding in §1 is exactly what happens.
3. **A second CI-mirroring invocation with a MySQL `DATABASE_URL`** —
   `DATABASE_URL=mysql://root:$MYSQL_ROOT_PASSWORD@127.0.0.1:3306/dbpro_fixture cargo test -p db-pro-infrastructure --test mysql_integration --test mysql_fixture_matrix -- --include-ignored --test-threads=1`.
   It has to be scoped like this (not `--all`): `pg_integration`'s `setup()` panics when `DATABASE_URL`
   is not a PostgreSQL URL, so a repo-wide `--all` run cannot serve both providers at once — the skip
   guard in `mysql_integration.rs` and `mysql_fixture_matrix.rs` already keeps the PostgreSQL-URL run
   green. Widening it would need `pg_integration`'s own self-skip (the same pattern pass 6 added for
   MySQL), which is a change to a PostgreSQL test file and was therefore left alone.

## 10. What could not be verified here

- **No GUI/workspace evidence.** Nothing in this pass exercises the schema Explorer or the Query/Table
  workspaces (criterion 2): the driver card is disabled, so there is no path from the UI to this provider.
- **`#[ignore]` still guards the five original live tests.** They run only with `--ignored`/
  `--include-ignored`; the new file is not ignored so a local `cargo test --workspace` with a MySQL URL
  runs it, but a CI job without a MySQL service still exercises neither.
- **MySQL `CHECK` constraints** are not introspected (recorded, not fixed).
- **The `utf8mb4_bin` class mapping** is a driver-level classification limit (§4).
- **Write/round-trip** (parameter binding) is unimplemented; only the read path is proven.
- The probe used for §3 was deleted before the commit, as required; its raw output is reproduced in this
  file, and every assertion it made is now carried by the committed live tests (`mysql_fixture_matrix.rs`).
