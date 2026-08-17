# Plan: Remediate PostgreSQL Provider Type Mapping & Decoding Defects (QA-P1-14)

## Overview
Address release-blocking P1 defect `QA-P1-14` (PostgreSQL row mapper type mapping matrix & decoding resilience).

### Lifecycle State
`IMPLEMENTING`

### Severity
`P1`

### Problem Statement
In `crates/infrastructure/src/postgres/query_mapper.rs`, PostgreSQL query row mapping explicitly handles only `BOOL`, `INT2`, `INT4`, `INT8`, `FLOAT4`, `FLOAT8`, `UUID`, `TIMESTAMPTZ`, `TIMESTAMP`, `JSON`/`JSONB`, and `BYTEA`. All other Postgres data types (e.g. `NUMERIC`, `DECIMAL`, `DATE`, `TIME`, `TIMETZ`, `INTERVAL`, `INET`, `CIDR`, `MACADDR`, `MONEY`, `OID`, `BIT`, `VARBIT`, `VARCHAR`, `TEXT`, `CHAR`, `BPCHAR`, `NAME`, `CITEXT`, arrays, custom ENUMs, and DOMAINs) fall through to `row.try_get::<String>(i)`.

If `sqlx` fails to decode a column into `String` (for instance, complex composite types, certain binary formats, or custom driver encodings), `map_row` returns `Err`. This aborts the **entire query execution** for the user. Furthermore, exact decimal types (`NUMERIC`/`DECIMAL`) lack explicit lossless conversion guarantees.

### Scope & Invariants
1. Expand explicit type dispatch in `crates/infrastructure/src/postgres/query_mapper.rs` for common PostgreSQL types.
2. Implement cell-level graceful fallback: if a cell cannot be decoded using its primary expected type or string representation, fall back to stringification or a safe placeholder string rather than failing the entire query row.
3. Preserve lossless transport for precision-sensitive numeric types via `CellValue::Text`.
4. Add comprehensive unit tests verifying mapping, round-trip, and error-recovery behavior.
5. Execute Rust & Frontend quality gates.
