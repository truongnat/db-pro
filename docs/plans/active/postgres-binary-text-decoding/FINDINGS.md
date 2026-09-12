# Findings: Fix PostgreSQL Binary-Protocol Textual Value Decoding

## Problem Statement
In `crates/infrastructure/src/postgres/query_mapper.rs`, `decode_textual_value` receives a `sqlx::postgres::PgRow` and column index, retrieves `raw = row.try_get_raw(i)`, and calls `raw.as_str()`.

When sqlx executes parameterized queries against PostgreSQL in binary protocol mode (`PgValueFormat::Binary`), calling `raw.as_str()` on `PgValueRef` fails with `ColumnDecode("expected text format")` because sqlx's `ValueRef::as_str` implementation checks `self.format() == PgValueFormat::Text`.

## Severity
- Severity: P1
- Area: PostgreSQL provider correctness / query result decoding

## Evidence
`crates/infrastructure/src/postgres/query_mapper.rs`:
```rust
fn decode_textual_value(row: &sqlx::postgres::PgRow, i: usize, data_type: &str) -> Result<CellValue, DbError> {
    let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;
    let value = raw
        .as_str()
        .map(str::to_owned)
        .map_err(|error| DbError::QueryFailed(format!("cannot decode PostgreSQL {data_type} as text: {error}")))?;
    Ok(CellValue::Text(value))
}
```

## Failure Scenario
1. A PostgreSQL query returns custom types, ENUMs, CITEXT, or domain types.
2. PostgreSQL returns the result in binary format (`PgValueFormat::Binary`).
3. `decode_cell` falls through to `decode_textual_value`.
4. `raw.as_str()` fails because `raw.format()` is `PgValueFormat::Binary`.
5. Query execution fails completely with `cannot decode PostgreSQL <data_type> as text: expected text format`.

## Proposed Solution
Inspect `raw.format()`:
- For `PgValueFormat::Text`, call `raw.as_str()`.
- For `PgValueFormat::Binary`, fetch `raw.as_bytes()` and decode via `std::str::from_utf8(bytes)`.
