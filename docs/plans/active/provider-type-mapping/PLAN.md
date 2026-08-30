# Provider Type Mapping (QA-P1-14) Plan

Canonical lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`.

## Objective
Remediate PostgreSQL query result decoding (QA-P1-14) by expanding the row mapping type matrix to cover non-basic types (`NUMERIC`, `DECIMAL`, `MONEY`, `TIMETZ`, `INTERVAL`, `INET`, `CIDR`, `BIT`, `VARBIT`, `TSVECTOR`, `TSQUERY`, and array types) and adding safe fallback checking (`PgValueFormat::Text`) to prevent binary payload decoding errors.

## Implementation Steps
1. Update `crates/infrastructure/src/postgres/query_mapper.rs` with additional type handlers and safe text fallback.
2. Ensure array types decode into `Vec<Option<T>>` to support nullable array elements.
3. Add unit tests for parameter binding, array JSON handling, and fallback invariants.
4. Execute quality gates: `cargo check`, `cargo clippy`, and `cargo test`.
