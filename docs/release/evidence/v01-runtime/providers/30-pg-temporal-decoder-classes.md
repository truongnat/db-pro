# PostgreSQL temporal decoders no longer invent a timezone (#56)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 5749d8e` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#56** ([Gate 5][B2] Implement PostgreSQL temporal decoder paths without timezone invention) —
  parent workstream **#23**, parent gate **#21**, depends on **#52**/**#54**
- Contract source: #52 (Gate 5 A2) "Canonical temporal semantics to lock" —
  TIMESTAMP = `YYYY-MM-DDTHH:MM:SS.ffffff` (**no** timezone),
  TIMESTAMPTZ = `YYYY-MM-DDTHH:MM:SS.ffffffZ`

## 1. Measured behaviour before the change

Reproduction on the live fixture (`dbpro-v01-pg-fixture`, PostgreSQL 16), session timezone set to
`America/New_York` through the connector itself so the connection cannot be mistaken for a UTC session:

```
SELECT TIMESTAMP '2024-03-15 10:20:30.123456' AS naive_stamp
```

| Path | Observed cell | Verdict |
|---|---|---|
| `query_mapper::decode_cell` `"TIMESTAMP"` arm | `DateTime("2024-03-15T10:20:30.123456+00:00")` | **DEFECT** — a naive `timestamp` was stamped with `+00:00`, i.e. reinterpreted as an instant |

Raw failing assertion, before the fix (`crates/infrastructure/tests/pg_integration.rs`):

```
assertion `left == right` failed: a naive timestamp must keep its wall-clock reading
  left: "DateTime(\"2024-03-15T10:20:30.123456+00:00\")"
 right: "DateTime(\"2024-03-15T10:20:30.123456\")"
```

The marker came from `NaiveDateTime::and_utc()` (`crates/infrastructure/src/postgres/query_mapper.rs:371`
before the change): the decoder *supplied* the missing zone instead of reporting that the value class
has none. The stored value class is unrepresentable once the marker exists, so nothing downstream
(#60-#63) could recover the distinction between a `timestamp` and a `timestamptz`.

While pinning the matrix, the `"TIMESTAMPTZ"` arm was measured as well: `DateTime::to_rfc3339()` renders
a zero offset as `+00:00`, not the `Z` that #52's locked contract spells
(`crates/infrastructure/src/postgres/query_mapper.rs:367` before the change).

## 2. The fix

Two named formatters next to the other temporal formatters in
`crates/infrastructure/src/postgres/query_mapper.rs`; each decoder arm now names the value class it
decodes instead of reusing an instant rendering:

| Type | Decoder | Rendering |
|---|---|---|
| `timestamp without time zone` | `try_get::<NaiveDateTime>` → `format_naive_timestamp` | `%Y-%m-%dT%H:%M:%S%.6f` — no marker at all |
| `timestamp with time zone` | `try_get::<DateTime<Utc>>` → `format_utc_instant` | `to_rfc3339_opts(SecondsFormat::Micros, true)` — normalized UTC with `Z` |

`DATE`, `TIME`, `TIMETZ`, `INTERVAL`, `INET`/`CIDR`, `NUMERIC` and the null path are unchanged; the
session/OS timezone never reaches them because SQLx decodes them from the wire value, not from a
session-formatted string.

Fractional policy: microseconds are spelled out to six digits (`10:20:30` → `10:20:30.000000`), which
is the "preserve PostgreSQL microsecond precision" row of #52's contract and keeps the string
width-stable for consumers.

**Representation note (deliberately not changed here).** #52's representation plan lists dedicated
domain/IPC variants (`timestamp`, `timetz`, `timestamptz`) and keeps `CellValue::DateTime` as the
pre-Gate-5 legacy variant "until decoder migration #56 removes the ambiguous provider mapping". This
change removes the ambiguous mapping — after it, `CellValue::DateTime` holds *either* a
microsecond-precise naive wall-clock reading *or* a `Z`-marked UTC instant, and the two are no longer
confusable. Adding the dedicated variants is an IPC/DTO change that belongs to #52/#54; it is recorded
here as the explicit remainder rather than smuggled into the decoder fix.

## 3. Verification

Focused unit tests — `cargo test -p db-pro-infrastructure --lib postgres::query_mapper`
→ **13 passed / 0 failed / 0 ignored** (4 new):

- `naive_timestamp_carries_no_timezone_marker`
- `naive_timestamp_spells_out_whole_second_fraction`
- `utc_instant_keeps_explicit_utc_marker`
- `utc_instant_is_normalized_to_utc`

Provider tests against the live fixture, session timezone `America/New_York` —
`DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored`
→ **20 passed / 0 failed / 0 ignored** (2 new; the 18 pre-existing PG tests stay green):

| Type | Input | Decoded cell |
|---|---|---|
| DATE | `DATE '2024-03-15'` | `Date("2024-03-15")` |
| TIME | `TIME '10:20:30.123456'` | `Time("10:20:30.123456")` |
| TIMETZ | `TIMETZ '10:20:30.123456+05:30'` | `Time("10:20:30.123456+05:30")` |
| TIMESTAMP | `TIMESTAMP '2024-03-15 10:20:30.123456'` | `DateTime("2024-03-15T10:20:30.123456")` |
| TIMESTAMPTZ | `TIMESTAMPTZ '2024-03-15 10:20:30.123456+00'` | `DateTime("2024-03-15T10:20:30.123456Z")` |
| TIMESTAMP, whole second | `TIMESTAMP '2024-03-15 10:20:30'` | `DateTime("2024-03-15T10:20:30.000000")` |
| NULL | `NULL::TIMESTAMP` | `Null` |

- `pg_timestamp_without_time_zone_keeps_wall_clock_value` — fails before the fix, passes after
- `pg_temporal_classes_decode_to_canonical_strings` — the full class matrix above in one query

Gate line for this change (see `LEDGER.md` for the recorded numbers): `cargo fmt --all -- --check`,
`cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo build --release --locked -p db-pro-native`,
`bash .skills/perf-audit/scripts/perf-scan.sh`.

## 4. Left open / handed to other rows

- Dedicated `timestamp`/`timetz`/`timestamptz` domain variants and their DTO mirrors (#52/#54) — this
  fix deliberately stays on the existing `DateTime` variant.
- Frontend copy/export of these canonical strings (#61) and type-aware editability (#62).
