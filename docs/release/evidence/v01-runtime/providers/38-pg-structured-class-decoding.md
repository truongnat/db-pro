# PostgreSQL structured/binary value-class decoding (#57)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ acc142c` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#57** ([Gate 5][B3] Implement JSON/JSONB/UUID/BYTEA/network decoder paths) —
  parent workstream **#23**, parent gate **#21**
- Provider: live fixture container `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432,
  `PostgreSQL 16.15`), `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. What existed before

All five classes have explicit decode branches in `decode_cell`
(`crates/infrastructure/src/postgres/query_mapper.rs:329`): `UUID` → `CellValue::Uuid`,
`JSON`/`JSONB` → `CellValue::Json`, `BYTEA` → `CellValue::Bytes`, `INET`/`CIDR` → `decode_inet`
(`:452`), with `decode_numeric` also handling both wire formats. What was missing was the proof:

- `decode_cell` had **no unit test** (a `PgRow` cannot be constructed without a live server, so the
  live suite is the only place this can be pinned).
- The 20 live cases asserted `Decimal`, `Text` (an enum label), `Int64`, `Time`, `Interval`, `Inet`,
  `Bool`, temporal formats and NULL — **JSON, JSONB, UUID and BYTEA were asserted nowhere** and no
  query in the file selected such a column.

## 2. The change

One live test over the real fixture columns plus the inline borderline classes
(`crates/infrastructure/tests/pg_integration.rs`):

| Test | Assertions |
|---|---|
| `pg_query_decodes_structured_and_binary_value_classes` | fixture read of `products.id` (UUID), `products.metadata` (JSONB) and all three `documents.binary_data` rows (BYTEA) plus an inline class query |

Expected-vs-actual matrix, measured on `PostgreSQL 16.15`:

| Class | Source | Expected (asserted) | Contract point pinned |
|---|---|---|---|
| UUID | `products.id` = `a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11` | `CellValue::Uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11")` | canonical lowercase text, not bytes |
| JSONB | `products.metadata` = `{"brand": "TechCo", "weight_kg": 1.8}` | `CellValue::Json` with `["brand"] == "TechCo"`, `["weight_kg"] == 1.8` | structured, keyed access survives |
| JSON | `'{"a":1,"b":[true,null]}'::json` | `CellValue::Json` with `["a"] == 1`, `["b"][0] == true` | JSON and JSONB agree |
| JSONB | the same literal cast to `jsonb` | `CellValue::Json` with `["b"][1].is_null()` | nested nulls preserved |
| BYTEA | `documents.binary_data` = `E'\\xDEADBEEF'` | `CellValue::Bytes([0xde, 0xad, 0xbe, 0xef])` | non-UTF8 bytes stay byte-exact (no mojibake) |
| BYTEA | `E'\\x48656c6c6f'` | `CellValue::Bytes(b"Hello")` | ASCII bytes exact |
| BYTEA | `NULL` | `CellValue::Null` | NULL is not empty bytes |
| CIDR | `'192.0.2.0/24'::cidr` | `CellValue::Inet("192.0.2.0/24")` | address **and** prefix preserved |
| NULL handling | `NULL::jsonb` vs `'null'::jsonb` | `CellValue::Null` vs `CellValue::Json(Value::Null)` | a SQL NULL is distinguishable from a JSON literal null |

The first run of the new test was **green**, which is the honest result here: this issue's remainder was
missing *coverage*, not a defect. The assertions are exact (value equality, not "is not an error"), so
they fail if a class is re-tagged or coerced.

## 3. Verification

- `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored --test-threads=1`
  → **21 passed / 0 failed / 0 ignored** (18 at the `07-pg-integration-live.txt` baseline, 20 after the
  #56 temporal cases; the new case is the 21st).
- Gate line for this change: `cargo fmt --all -- --check` exit 0, `cargo check --workspace` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace`
  **862 passed / 0 failed / 22 ignored** (baseline 862/0/21 — the new case is `#[ignore]`d and only runs
  with `DATABASE_URL`), `cargo build --release --locked -p db-pro-native` exit 0,
  `bash .skills/perf-audit/scripts/perf-scan.sh` `Status: PASS` (4/0/0).

## 4. Not claimed here

- A `decode_cell` **unit** test: impossible without a live server (`PgRow` is not constructible from a
  fixture), which is why these assertions live in the integration suite.
- Array element parsing, enum/domain arms and the custom-type fallback: #58 (and the contract in #53).
- JSON numbers beyond `i64`: they route through `serde_json`'s number model
  (`CellValue::Json`), and the canonical text contract for precision-sensitive *columns* is A1's, not
  this path's.
