# PLAN — PostgreSQL Non-Basic Provider Type Mapping (QA-P1-14)

## Problem Statement

In `crates/infrastructure/src/postgres/query_mapper.rs`, PostgreSQL query row decoding handled explicit basic types (`BOOL`, `INT2/4/8`, `FLOAT4/8`, `UUID`, `TIMESTAMPTZ`, `TIMESTAMP`, `DATE`, `TIME`, `JSON/JSONB`, `BYTEA`). However:
1. `NUMERIC` and `DECIMAL` types fell back to `row.try_get::<String>()`, or produced decoding errors when converted to cell values.
2. `MONEY`, `TIMETZ`, `INTERVAL`, `INET`/`CIDR`, `BIT`/`VARBIT`, `TSVECTOR`/`TSQUERY` were unhandled, triggering error fallbacks or unformatted string conversions.
3. PostgreSQL Array types (e.g., `_text`, `_int4`, `text[]`) were unhandled or failed decoding when elements contained `NULL` values if attempted via standard `Vec<T>`.
4. Binary protocol raw decoding fallback indiscriminately called `raw.as_str()` on binary format payloads, causing potential UTF-8 decoding panic or corruption.

This defect is classified as **P1 Severity** (provider-specific query failure and data mapping risk).

## Proposed Changes

1. **PostgreSQL Row Mapper Expansion (`crates/infrastructure/src/postgres/query_mapper.rs`)**:
   - Add explicit decoding for `NUMERIC` / `DECIMAL` (try f64 or fallback to String text representation).
   - Add decoding for `MONEY` formatted as string.
   - Add decoding for `TIMETZ` and `INTERVAL`.
   - Add decoding for network/bit/textsearch types (`INET`, `CIDR`, `BIT`, `VARBIT`, `TSVECTOR`, `TSQUERY`).
   - Add safe decoding for array types (`dt_upper.starts_with('_') || dt_upper.ends_with("[]")`) using `Vec<Option<T>>` to handle nullable array elements safely without decoding failure.
   - Guard raw binary format decoding by checking `raw.format() == PgValueFormat::Text` before calling `raw.as_str()`.

2. **Unit Tests**:
   - Add unit tests for `bind_params_all_variants_succeed` and `query_param_array_serialization_integrity`.
   - Verify PostgreSQL query mapper compiles and passes unit tests.

## Plan Directories
Created under `docs/plans/active/provider-type-mapping/`:
- `PLAN.md`
- `CHECKLIST.md`
- `FINDINGS.md`
- `VERIFICATION.md`
