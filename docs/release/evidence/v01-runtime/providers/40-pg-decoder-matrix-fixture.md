# PostgreSQL decoder matrix fixture (#59)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 8a53fdf` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#59** ([Gate 5][B5] Add PostgreSQL integration fixture covering full decoder matrix) —
  parent workstream **#23**, parent gate **#21**
- Provider: live fixture container `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432,
  `PostgreSQL 16.15`), `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. What existed before

`fixtures/postgres/001_schema.sql` covered bool/int4/int8/numeric/uuid/text/jsonb/bytea/timestamptz/date,
an enum, an array column and a generated column, but **no domain, no interval column, no time/timetz
column, no int2, no float4/float8, no inet/cidr column and no range or composite type**. The decoder
cases asserted those classes only through inline SQL literals inside the test, so "the fixture covers
the class" was not true for them, and the domain path was untested anywhere
(`grep -rn "CREATE DOMAIN" fixtures/` → no matches). The `tags TEXT[]` column was never selected by any
test.

## 2. The fixture

Added to the load path CI already uses (`ci.yml:69-71` loads `001_schema.sql`, `002_seed.sql`,
`003_verify.sql` in order):

- **`001_schema.sql`**: `CREATE DOMAIN positive_quantity AS INTEGER CHECK (VALUE > 0)`,
  `CREATE DOMAIN postal_code AS TEXT`, `CREATE TYPE decoder_pair AS (left_value INTEGER, right_value TEXT)`
  and the table `decoder_matrix` with **one column per value class**:

  | Column | Class | Column | Class |
  |---|---|---|---|
  | `id SMALLINT` | int2 | `token UUID` | uuid |
  | `flag BOOLEAN` | bool | `doc JSON` / `payload JSONB` | json / jsonb |
  | `count INTEGER` | int4 | `blob BYTEA` | bytea |
  | `big_count BIGINT` | int8 | `address INET` / `network CIDR` | inet / cidr |
  | `ratio REAL` | float4 | `status order_status` | enum |
  | `precise_ratio DOUBLE PRECISION` | float8 | `quantity positive_quantity` | domain over int4 |
  | `amount NUMERIC(24,4)` | numeric, high precision | `postal postal_code` | domain over text |
  | `calendar_date` / `wall_time TIME(6)` | date / time | `labels TEXT[]` | array |
  | `zoned_time TIMETZ` | timetz | `slot INT4RANGE` | range (no decoder arm) |
  | `local_stamp TIMESTAMP(6)` | timestamp | `pair decoder_pair` | composite (no flat form) |
  | `instant TIMESTAMPTZ` | timestamptz | `missing TEXT` | nulls |
  | `span INTERVAL` | interval | | |

- **`002_seed.sql`**: one fully populated row (`id = 1`, int4/int8 at their maxima, `NUMERIC` with 24
  significant digits, the five temporal classes with microseconds and a non-UTC `TIMETZ` offset, a range,
  a composite and a two-element array) and one row (`id = 2`) that is NULL in every nullable column.
- **`003_verify.sql`**: two new assertions so the fixture stays deterministic — `decoder_matrix` has
  exactly 2 rows, and the populated row keeps `flag`/`quantity`.

## 3. The test

`pg_decoder_matrix_covers_every_value_class` (`crates/infrastructure/tests/pg_integration.rs`) reads the
table with `SELECT *` and asserts the expected-vs-actual matrix in column order:

| Column | Asserted value |
|---|---|
| `id` / `count` / `big_count` | `Int64(1)` / `Int64(2147483647)` / `Int64(9223372036854775807)` |
| `flag` | `Bool(true)` |
| `ratio` / `precise_ratio` | `Float64(1.5)` / `Float64(0.1)` |
| `amount` | `Decimal("12345678901234567890.1234")` — every digit kept, no float hop |
| `calendar_date` | `Date("2024-03-15")` |
| `wall_time` / `zoned_time` | `Time("10:20:30.123456")` / `Time("10:20:30.123456+07:00")` |
| `local_stamp` | `DateTime("2024-03-15T10:20:30.123456")` — **no invented offset** |
| `instant` | `DateTime("2024-03-15T10:20:30.123456Z")` — explicit UTC marker |
| `span` | `Interval("1 mons 2 days 03:04:05.000006")` — microsecond precision |
| `token` | `Uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11")` |
| `doc` / `payload` | `Json` with `["a"] == 1` / `["brand"] == "TechCo"` |
| `blob` | `Bytes([0xde, 0xad, 0xbe, 0xef])` |
| `address` / `network` | `Inet("192.0.2.1…")` / `Inet("192.0.2.0/24")` |
| `status` | `Text("shipped")` — canonical enum label |
| `quantity` / `postal` | `Int64(7)` / `Text("SW1A 1AA")` — domains through their base type |
| `labels` / `slot` / `pair` | `Bytes(…)` — the documented byte-exact fallback (#58, A3 contract) |
| `missing` | `Null` |

It also asserts that the **declared provider type reaches the caller** for the classes the policy layer
depends on (`INT2`, `NUMERIC`, `TIMETZ`, `TEXT[]`, `INT4RANGE`, `decoder_pair` — note a domain column
resolves to its base type and a named composite keeps its own name), and that all 24 nullable columns of
the second row are `Null` cells (the NOT NULL `id`/`flag` keep `2`/`false`).

The first run of the test against the reloaded fixture was green; the two adjustments made while writing
it were both measurement-driven (the composite column's declared name is `decoder_pair`, not `RECORD`;
the NULL row's `id` is 2 because it is the primary key) and are recorded here rather than silently fixed.

## 4. Verification

- Fixture reload, exactly the CI steps (`psql -v ON_ERROR_STOP=1 -f …` in order, after
  `DROP SCHEMA public CASCADE`): `001` → OK, `002` → OK, `003` → both `DO` assertions pass
  (`decoder_matrix` = 2 rows; populated row intact).
- Live suite: `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored
  --test-threads=1` → **24 passed / 0 failed / 0 ignored** (23 before; the matrix case is the 24th).
- Gate line for this change: `cargo fmt --all -- --check` exit 0, `cargo check --workspace` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace`
  **863 passed / 0 failed / 25 ignored** (baseline 863/0/24 — the matrix case is `#[ignore]`d behind
  `DATABASE_URL`), `cargo build --release --locked -p db-pro-native` exit 0,
  `bash .skills/perf-audit/scripts/perf-scan.sh` `Status: PASS` (4/0/0).

## 5. Not claimed here

- Array **element** decoding: the matrix column is asserted as the byte-exact fallback, which is the
  v0.1 contract (A3, #53). Element parsing is post-v0.1.
- The fixture is reusable by #25's verification pass (the issue's own acceptance): it is loaded by
  `ci.yml` and read by one test with no test-local DDL.
- The `fixtures/smoke/**` datasets are a separate, larger surface and are untouched.
