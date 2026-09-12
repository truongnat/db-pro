# Fix PostgreSQL Binary-Protocol Textual Value Decoding

## Summary
In `crates/infrastructure/src/postgres/query_mapper.rs`, `decode_textual_value` unconditionally calls `raw.as_str()` on `PgValueRef`. When sqlx receives row values in binary format (`PgValueFormat::Binary`), calling `as_str()` on a binary `PgValueRef` returns a `ColumnDecode("expected text format")` error. This causes queries returning fallback textual types (such as custom ENUMs, domain types, CITEXT, BPCHAR, or NAME columns) under binary protocol to fail completely.

This plan addresses this P1 correctness issue by inspecting `raw.format()` in `decode_textual_value` and decoding binary payloads via `std::str::from_utf8(bytes)`.

## Scope
1. Update `decode_textual_value` in `crates/infrastructure/src/postgres/query_mapper.rs`.
2. Add regression tests in `crates/infrastructure/src/postgres/query_mapper.rs`.
3. Verify test suite and quality gates.
