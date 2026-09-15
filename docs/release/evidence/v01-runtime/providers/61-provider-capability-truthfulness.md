# Provider capability truthfulness, capability path, contract docs, and the value/decoder conformance case (#234)

- Session: provider-contract completion pass, 2026-09-15 (pass 8)
- **Base head:** `main @ aabdda7f` (worktree clean, remote in sync; coordinator-verified gates at that
  head: `fmt` PASS, `check` PASS, `clippy -D warnings` PASS, `cargo test --workspace` **942 passed / 0
  failed / 35 ignored**, CI-mirror `--include-ignored` **977 / 0 / 0**)
- **Change head:** `46cfed05` (this file's commit follows it; see §5 for the exact heads per gate)
- **Fixtures:** PostgreSQL `dbpro-v01-pg-fixture` (`postgres:16`, 16.15) on host port **55432**, started
  with `docker start` and **stopped again** at the end; MySQL `dbpro-v01-mysql-fixture`
  (`mysql:8.4`, **8.4.11**) on host port **33306**, recreated for this pass from `fixtures/mysql/`
  (the previous pass removed it) and **removed again** afterwards. Credentials are redacted here; they
  are the disposable-test values in `docker-compose.yml` / the verification runbook. No `qltx-*`
  container was touched, and nothing under `.github/workflows/` or `docs/goals/` was changed.
- **Outcome:** the MySQL capability advertisement now matches the code (5 flags corrected, 1 further
  flag corrected on the same rule), the UI has a named capability path instead of `None`, the provider
  contract is recorded in `docs/architecture/`, the two stale release documents are corrected, and the
  conformance suite has the value/decoder case whose absence let #235's mapper defect pass — proven
  live against both fixtures. **#234 stays OPEN**: criteria 1 and 6 are now met, 2/3/4/5 are partial
  (§6).

## 1. The capability truthfulness table (before → after, with the code path)

The rule the corrected set is written to, and now stated on `DatabaseCapabilities::mysql()`
(`crates/core/src/domain/capabilities.rs`): **a flag is `true` only when a shipping code path serves
that capability for that driver.** Where the engine could do the work but the product's only path
rejects the driver, the flag is `false`. Pass 6 named five contradicted flags; reading the tree for
each one under this rule turned up a sixth of exactly the same shape (`data_diff`).

### 1.1 Corrected flags

| Flag | Before | After | Code path that decides it |
|---|---|---|---|
| `query.parameters` | `true` | **`false`** | `MySqlConnector::query`/`execute` take `_params` and run `sqlx::query(sql)` with no binds (`crates/infrastructure/src/mysql/connector.rs`) — there is no binder, so a `?`-bound statement would run unbound |
| `query.positional_parameters` | `true` | **`false`** | Same path: `?` placeholders are accepted by the signature and dropped |
| `features.server_sessions` | `true` | **`false`** | `ensure_server_sessions_config` rejects every driver except PostgreSQL (`crates/core/src/application/user_service.rs`); the only user-management path errors |
| `features.partitions` | `true` | **`false`** | `PostgresApi::partitions` resolves a PostgreSQL handle and rejects anything else (`crates/runtime/src/api.rs`); there is no MySQL partition path |
| `features.backup` | `true` | **`false`** | `BackupService::backup`/`restore` return a validation error for MySQL (`crates/core/src/application/backup_service.rs`) |
| `features.data_diff` | `true` | **`false`** | `DataDiffService::diff_table_data` asks the connector for a dialect (`crates/core/src/application/data_diff.rs`) and `CompositeConnector::dialect` has no MySQL arm (`crates/infrastructure/src/connector.rs`) — the only data-diff path errors. **Not named by pass 6**; corrected on the same rule and the same kind of evidence |

### 1.2 Flags left `true`, with the path that serves them

A flag was not corrected because a `false` value would have been the *new* inaccuracy.

| Flag | Value | Path that serves it |
|---|---|---|
| `query.explain` | `true` | `MySqlConnector::explain` + `mysql_explain_returns_json` (live) |
| `query.multi_statement` | `true` | `execute_batch`/`execute_transaction` on the connector; `mysql_transaction_rolls_back_on_failure` proves the transaction path live |
| `schema.schemas` | `true` | MySQL databases are introspected as schemas; the live fixture tests read the canonical model |
| `schema.functions`, `views`, `triggers`, `indexes`, `foreign_keys` | `true` | `crates/infrastructure/src/mysql/introspect.rs`; live assertions for the view, trigger, routine, FK from/to and index→table grouping |
| `schema.add_column` / `drop_column` / `rename_column` / `alter_column_type` | `true` | DDL is driver-aware (`build_create_table_ddl(info, driver, …)`) and `SchemaService::execute_ddl`/`execute_ddl_batch` go through the driver-agnostic connector methods |
| `schema.enum_types` | `true` | MySQL `ENUM`/`SET` decode as `Text` in the live 41-column matrix |
| `data.insert` / `update` / `delete` | `true` | Plain SQL through `execute`; `INSERT` is live-proven (`mysql_execute_returns_affected_rows`) and the fixture's trigger fires on it |
| `data.json_type`, `data.blob_type`, `data.composite_pk`, `data.generated_columns` | `true` | Structured JSON, byte-exact `Bytes` and the composite PK are live-asserted; `generated_columns` is read by the introspector (`EXTRA` → `is_generated`) |
| `features.schema_diff` | `true` | Compares two `IntrospectResult`s (`crates/core/src/application/schema_diff.rs`) — no provider-specific path needed |

### 1.3 Flags already `false`, pinned so they cannot drift

`query.cancel` (connector returns `DbError::Unsupported`), `query.numbered_parameters`,
`schema.transactional_ddl`, `schema.sequences`, `schema.rename_objects`, `data.uuid_type`,
`data.array_types`, `features.tablespaces`, `features.object_dependencies`, `features.ssh_tunnel`.

**Recorded, not corrected, and not hidden:** `data.insert`/`update`/`delete` and the `schema.*` DDL
flags are `true` because the engine serves them through the connector's generic SQL path, while the
*staged* Table Data Editor path needs `dialect()` and is therefore **not reachable** for MySQL. That
distinction (flag = a shipping path serves it; the editor is a product surface with its own gate) is
what decides the table above; the underlying gap is the missing dialect arm, and it is listed in
`docs/release/provider-capability-matrix.md` and `docs/architecture/provider-contract.md`.

### 1.4 Tests pinning the corrected values

| Test | File | Pins |
|---|---|---|
| `mysql_capabilities_drop_parameters_until_a_binder_exists` | `crates/core/src/domain/capabilities.rs` | the three parameter flags are `false`, naming the connector path |
| `mysql_capabilities_do_not_advertise_postgres_only_features` | same | `server_sessions`, `partitions`, `backup`, `data_diff` are `false` (plus `tablespaces`, `object_dependencies`) |
| `mysql_capabilities_keep_the_flags_a_shipping_path_serves` | same | the 1.2 set stays `true` |
| `mysql_advertisement_matches_the_shipping_code_paths` | `crates/infrastructure/tests/provider_contract_conformance.rs` | the same values **through `CompositeConnector::capabilities`**, so a factory that overrides them is caught too |

## 2. The capability path (criterion 2's first half)

**Before.** `crates/ui/src/app.rs` resolved capabilities from a driver **string** in two
`Option<DatabaseCapabilities>` lookups; anything that was not `sqlite`/`postgres`/`postgresql`
returned `None`, so the MySQL factory registered in `0729d699` had no capability path at all and
every gate read as "capability absent". Two states were conflated: *no connection is active* and
*this driver has no capability entry*. A third problem was invisible: `query_capabilities()` fell back
to `active_query_driver()`, whose display fallback is the literal `"PostgreSQL"`, so it answered with
PostgreSQL's capability set while nothing was connected.

**After.** `CapabilityLookup` (`crates/ui/src/app.rs`) is a named state:

| State | Meaning |
|---|---|
| `Supported(DatabaseCapabilities)` | the driver is a provider this build ships; the set is authoritative |
| `NoActiveConnection` | nothing is connected, so there is nothing to resolve |
| `UnsupportedDriver { driver }` | the connection names a driver with no provider entry in this build |

- `for_driver_label` maps `PostgreSQL`/`postgres`, `SQLite`, and now **`MySQL`** → `DriverType`, so
  MySQL resolves to its own (corrected) set;
- `allows(predicate)` is the gating helper and **never** satisfies a capability predicate for a
  lookup that could not answer, so an unrecognised driver cannot silently enable an action;
- `unavailable_reason()` gives the user-facing why; `query_view::explain_query` prints it instead of a
  generic "unavailable until a supported connection is active";
- `query_capabilities()` now resolves from the query document's bound connection, then the active
  connection, then `NoActiveConnection` — it no longer answers with PostgreSQL's set when nothing is
  connected.

Call sites moved to `allows(..)`: `events.rs` (Escape/cancel), `query_view.rs` (Stop button),
`explorer_view.rs` (routines folder), `navigation_view.rs` (backup settings card). The stale
"Functions are PostgreSQL-only" comment in `explorer_view.rs` was corrected to the capability rule.

| Test | Pins |
|---|---|
| `mysql_connection_resolves_to_its_own_capability_set` | MySQL → `Supported`, with the provider's real `functions`/`backup`/`partitions`/`cancel`/`parameters` values |
| `unknown_driver_resolves_to_a_named_state_not_none` | an `Oracle` connection → `UnsupportedDriver { driver: "Oracle" }`, no capability set, no capability-gated action enabled, and a reason naming the driver |
| `no_active_connection_is_distinct_from_an_unsupported_driver` | `NoActiveConnection` is its own state with its own reason |
| `query_capabilities_follow_the_bound_connection_and_do_not_default_to_postgres` | no connection → `NoActiveConnection`; the bound query document's connection wins |

**Still not met for criterion 2** (see §6): the UI branches on provider name in
`query_view.rs` (formatting/parsing dialect, schema defaults), `connection_service.rs` and
`user_service.rs`, and the lookup is still a driver-*label* mapping rather than a query to the
registered factories.

## 3. Documentation

**Added:** `docs/architecture/provider-contract.md` (147 lines). Follows the existing architecture
documents' shape (`> Status:`, `## Source`, tables, fenced diagrams) and records:

1. what a provider is — the `ProviderFactory` and `DbConnector` port surfaces, and that service/UI
   behaviour is not part of the provider;
2. registration and dispatch — `CompositeConnector::new()`'s factory map, `register_factory`, the
   per-call dispatch table (including that `dialect()` has no MySQL arm today), and that
   `DriverType` is a closed enum so registration is compile-time with no plugin loading required;
3. the capability model — what `true` means (a shipping code path serves it), the worked MySQL
   examples, and an explicit statement that **there is no per-flag reason channel yet**: a `false`
   flag carries no *why*, the reason lives in the operation-level `DbError::Unsupported`, and a
   capability-level reason field is criterion 3's unimplemented remainder;
4. the value-decoding contract — by reference to `docs/release/provider-value-contract.md` and the
   decoder evidence, not restated, plus the rule that a probe-chain decoder is invalid with the MySQL
   D1 defect as the worked example, and a per-provider fixture/proof table;
5. what a new provider must supply — an eight-item checklist (factory, full `DbConnector` surface with
   named refusals, class-aware mapper, introspector, dialect arm, justified capability set, fixture +
   live tests, UI capability entry);
6. five recorded contract gaps so a consumer cannot mistake them for guarantees.

`grep -rl ProviderFactory docs/` returned **no files** before this pass and now returns the new
architecture document (plus `docs/plans/**` if the ledger is included in the pattern).

**Corrected:** `docs/release/known-limitations.md` LIM-002. It said "Only PostgreSQL and SQLite drivers
are implemented", which stopped being true at `640eaf4c`. It now states that PostgreSQL and SQLite are
the **shipped** drivers while MySQL 8 has a real provider on `main` with recorded live evidence and is
**not** a shipped v0.1 driver, and it lists what is still missing (no parameter binder, no dialect arm,
disabled UI card, PostgreSQL-only services) so the entry cannot be read as "MySQL is supported". The
entry keeps the registry's `Accepted v0.1` status vocabulary (the Status | Count table is unchanged at
13), the safe release-note wording is unchanged, and the header records the correction under the file's
existing convention.

**Corrected:** `docs/release/provider-capability-matrix.md`. Title now says "(plus the post-v0.1 MySQL
provider)"; a header note scopes the two shipped-driver tables unchanged; a new
`## MySQL 8 (post-v0.1 provider)` section has one row per capability area with the status it actually
has on `main`, each cell resting on live evidence or marked `NOT YET QUALIFIED` / `NOT REACHABLE` where
nothing verifies it (the Data Grid read/filter/sort path is `NOT REACHABLE` — it needs the dialect;
`CHECK` constraints stay `PARTIAL`; parameters and data diff are `NOT SUPPORTED` and match the
capability flags); a key-differences entry (9) and source references were added.

## 4. The value/decoder conformance case (criterion 4's gap)

`crates/infrastructure/tests/provider_contract_conformance.rs` executed SQLite in memory only and
asserted **no value/decoder contract**, which is exactly why #235's mapper served `count(*)` as a
boolean and `DECIMAL`/`DATE` as the literal text `unsupported` with the suite green.

Two tests were added to that file:

| Test | What it does |
|---|---|
| `provider_value_decoder_contract_holds_against_the_fixture` | connects **through `CompositeConnector`** to whichever provider `DATABASE_URL` names (`postgres://`/`mysql://`), reads that fixture's `decoder_matrix` — the table both fixtures already ship, not a new one — and asserts the class-aware contract; self-skips with a printed reason when the URL names neither |
| `factory_dispatch_drives_postgres_against_the_live_fixture` | closes criterion 1's "no live test drives PostgreSQL through the new factory dispatch": capability set, connect, query and introspect through the composite; self-skips on a non-PostgreSQL URL |

Assertions, per class (the fixture column set is the shared subset of both fixtures' matrices, plus a
per-provider binary column: `blob` on PostgreSQL, `blob_data` on MySQL):

| Rule | Assertion |
|---|---|
| integers stay integers, never booleans | `id` → `Int64(1)`, `big_count` → `Int64(i64::MAX)`, while the one genuinely boolean column (`flag`) still decodes as `Bool(true)` |
| `DECIMAL` is not rounded through `f64` | `amount` → `Decimal`, integer part exactly `12345678901234567890`, scale non-empty, **plus an in-test proof** that `format!("{:.0}", digits.parse::<f64>())` differs from those digits |
| temporal classes keep their semantics | `calendar_date` → `Date` with no time/offset, `wall_time` → `Time`, `local_stamp` → `Timestamp` starting with the seeded wall-clock `2024-03-15T10:20:30.123456` and carrying **no** `Z`/`+`, `instant` → `TimestampTz` with an explicit marker |
| structured classes | `doc` → `Json` with the seeded value; `status` → `Text` |
| binary stays bytes | the per-provider byte column → `Bytes`, hex-equal to the fixture's own bytes |
| `NULL` preserved per family | on the all-NULL row: `big_count`, `ratio`, `precise_ratio`, `amount`, `calendar_date`, `wall_time`, `local_stamp`, `instant`, `doc`, `status`, `missing`, binary column → `Null` (`flag` is excluded and the reason is in the test: PostgreSQL declares it `NOT NULL` and seeds `FALSE`, MySQL declares it nullable and seeds `NULL`) |

### 4.1 Falsification

Two probes were run against the live MySQL fixture and **both turned the test red**, then were
reverted (they are not in the tree):

| Probe | Result |
|---|---|
| claim `id` is a boolean (the exact shape of #235's D1 defect) | **FAILED** — `panicked at …: FALSIFICATION PROBE: claim id is a boolean` |
| claim the naive timestamp carries an offset | **FAILED** — `FALSIFICATION PROBE: claim the naive timestamp carries an offset: 2024-03-15T10:20:30.123456`, i.e. the test prints the value the server actually returned |

After restoring, the same test against the same server passes (`1 passed / 0 failed`), so the two
failures were the probes and not flakiness.

### 4.2 Live results

| Run | Result |
|---|---|
| `DATABASE_URL=postgres://…@127.0.0.1:55432/… cargo test -p db-pro-infrastructure --test provider_contract_conformance -- --test-threads=1` | **11 passed / 0 failed / 0 ignored**, exit 0 — both new tests ran live (no skip line) |
| `DATABASE_URL=mysql://…@127.0.0.1:33306/… cargo test -p db-pro-infrastructure --test provider_contract_conformance -- --test-threads=1` | **11 passed / 0 failed**, exit 0 — the decoder test ran live against MySQL; only `factory_dispatch_drives_postgres_against_the_live_fixture` skipped with its reason |
| `… --test mysql_integration --test mysql_fixture_matrix -- --include-ignored --test-threads=1` | **8 passed / 0 failed** (5 + 3), exit 0 |
| `DATABASE_URL=postgres://… cargo test --all -- --include-ignored` (PostgreSQL fixture live) | **987 passed / 0 failed / 0 ignored**, exit 0 |

MySQL fixture load, verified before the runs: `SELECT VERSION()` → **8.4.11**, `decoder_matrix` count
**2**, and **0** `ERROR` lines in the entrypoint log (the `SET NAMES utf8mb4` fix from `640eaf4c`
holds). The fixture was created for this pass and removed afterwards.

## 5. Gates at the change head

| Gate | Result | Raw |
|---|---|---|
| `cargo fmt --all -- --check` | PASS, exit 0 | no output |
| `cargo check --workspace` | PASS, exit 0 | `Finished dev profile [unoptimized + debuginfo] target(s) in 0.28s` |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 | `Finished dev profile in 0.26s`; **0** lines matching `warning` |
| `cargo test --workspace` | PASS, exit 0 | **952 passed / 0 failed / 35 ignored** (baseline **942 / 0 / 35** → **+10**) |
| `cargo build --release --locked -p db-pro-native` | PASS, exit 0 | `target/release/db-pro-native`, **24,960,720 bytes** |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial), exit 0 | 4 passed / 0 warnings / 0 failed / 4 not executed |
| `cargo test --all -- --include-ignored` (PostgreSQL fixture live) | PASS, exit 0 | **987 passed / 0 failed / 0 ignored** (baseline **977 / 0 / 0** → **+10**) |
| MySQL live targets, `--include-ignored` (MySQL `DATABASE_URL`) | PASS, exit 0 | 8 passed / 0 failed (5 + 3) |

The +10 on the plain workspace run: 3 `capabilities.rs` tests, 1 conformance advertisement test, 2
conformance live tests (skip path without `DATABASE_URL`), and 4 UI tests. The CI-mirroring run adds
the same ten, with two of them exercising the live path instead of skipping.

Raw outputs this pass are session-local under `/tmp/pass8/` (`gates-fast.txt`, `check.txt`,
`clippy.txt`, `workspace.txt`, `release-build.txt`, `perf-scan.txt`, `include-ignored.txt`,
`mysql-live.txt`, `mysql-conformance.txt`); every assertion they record is carried by the committed
tests.

## 6. #234's acceptance, criterion by criterion (disposition: stay open)

| # | Criterion | State after this pass |
|---|---|---|
| 1 | PostgreSQL and SQLite implement the same explicit contract, no regressions | **MET** — all three providers implement `ProviderFactory`/`DbConnector`; `factory_dispatch_drives_postgres_against_the_live_fixture` now drives PostgreSQL **through** the composite (capabilities, connect, query, introspect) against the live fixture, so "no regressions" is demonstrated for the new layer; `cargo test --workspace` 952/0/35 and the CI-mirror 987/0/0. Remainder, stated: `crates/runtime/src/api.rs` still reaches PostgreSQL directly for object dependencies/partitions/tablespaces/rename, which is path-level, not contract-level |
| 2 | UI/application code does not branch on provider name where capability dispatch suffices | **PARTIAL, smaller** — MySQL now resolves to its own capability set through a named state, an unknown driver is explicit rather than `None`, and no lookup silently answers with another driver's set. **Still unmet:** the UI branches on provider name for formatting/parsing dialect, schema defaults and connection/user service gates, and `CapabilityLookup` is a driver-label mapping rather than a query to the registered factories |
| 3 | Unsupported features expose a reason instead of generic failure | **PARTIAL** — operation-level reasons exist and improved (`DbError::Unsupported`, `unavailable_reason()` in the UI, the corrected `explain_query` message, and now honest `false` flags for the four PostgreSQL-only features so the UI cannot offer them). **Still unmet:** there is no capability-*level* reason channel — `DatabaseCapabilities` is booleans only, and the UI's disabled states/limitations are documented rather than carried per flag |
| 4 | Conformance test suite for query/value/schema/core lifecycle | **PARTIAL, materially closer** — the suite now has a **value/decoder case** that runs against the provider `DATABASE_URL` names, asserts the class-aware contract, and was falsified twice against the live MySQL fixture; query/transaction/introspection/cancel/factory-registration cases already existed for SQLite. **Still unmet:** PostgreSQL and MySQL have no *schema* or *lifecycle* conformance case in this file (their live coverage sits in `pg_integration.rs` / `mysql_fixture_matrix.rs`), and MySQL still has no dialect/parameter path to conform against |
| 5 | Adding a third provider does not require a second UI stack | **PARTIAL** — structurally met (single egui stack; MySQL reuses it, and now has a capability path). **Still unmet:** MySQL remains unusable from the UI because its driver card is disabled (`crates/ui/src/connection_view.rs`), which is #235's criterion 2 and is deliberate while there is no binder or dialect |
| 6 | Architecture docs + service/provider tests | **MET** — `docs/architecture/provider-contract.md` records the contract, and the two stale release documents (LIM-002, the capability matrix) are corrected; provider tests exist and now include the live decoder conformance case |

**Disposition: the issue stays open.** Criteria 1 and 6 are met; 2, 3, 4 and 5 are partial with the
remainder named above. Nothing in this pass was closed on partial credit.

## 7. What could not be verified here

- **No GUI evidence.** The MySQL driver card is still disabled, so no user-facing MySQL path exists;
  criterion 5's remaining half and #235's criterion 2 are untouched by design.
- **No MySQL parameter/dialect path**, so `parameters`/`positional_parameters`/`data_diff` are `false`
  by construction rather than by test — the flags are pinned, the features do not exist.
- **MySQL `CHECK` constraints** are still not introspected (recorded in the matrix as `PARTIAL`).
- **`cargo test --workspace` without `DATABASE_URL`** exercises the two new live tests only through
  their skip path; the live path is proven by the runs in §4.2 and is not part of the plain gate.
- **CI is unchanged**: `.github/workflows/*` was not touched, so CI still does not run a MySQL service
  — the recommendation in §9 of `providers/60-…` stands untouched.
