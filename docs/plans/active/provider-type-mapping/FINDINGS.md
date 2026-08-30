# Provider Type Mapping (QA-P1-14) Findings

## Finding Summary

### QA-P1-14 — PostgreSQL row mapper type matrix and decoding fallback
- **Severity:** P1
- **Area:** PostgreSQL query execution / row mapping
- **File:** `crates/infrastructure/src/postgres/query_mapper.rs`
- **Evidence:** Query execution using binary protocol fell through to `row.try_get::<String, _>(i)` for non-basic PostgreSQL types (such as `NUMERIC`, `MONEY`, `TIMETZ`, `INTERVAL`, `INET`, array types), causing binary decode errors and rendering cells as `<unsupported value: ...>`.
- **Fix:** Expanded `decode_cell` with handlers for `NUMERIC`, `DECIMAL`, `MONEY`, `TIMETZ`, `INTERVAL`, `INET`, `CIDR`, `BIT`, `VARBIT`, `TSVECTOR`, `TSQUERY`, and array types using `Vec<Option<T>>`. Added explicit `PgValueFormat::Text` format check before attempting string decoding in fallback handler.
