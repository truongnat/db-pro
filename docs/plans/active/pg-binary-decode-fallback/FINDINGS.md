# FINDINGS — PostgreSQL Query Mapper Fallback Binary Decoding Correctness

## Finding: Unsafe UTF-8 Conversion on Binary Protocol Payloads in Fallback Mapper

**Severity:** P1 (PostgreSQL correctness & data representation risk)
**Area:** PostgreSQL Query Mapper / Protocol Format Handling
**Files:** `crates/infrastructure/src/postgres/query_mapper.rs`

### Evidence
In `decode_cell()`:
```rust
    res.unwrap_or_else(|_| {
        row.try_get_raw(i)
            .ok()
            .and_then(|raw| raw.as_bytes().ok())
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .map(|value| CellValue::Text(value.to_owned()))
            .unwrap_or_else(|| CellValue::Text(format!("<unsupported value: {data_type}>")))
    })
```
`raw.as_bytes()` returns raw payload bytes regardless of `raw.format()`. In binary protocol mode, `bytes` are binary representations, not ASCII/UTF-8 string text.

### Failure Scenario
1. PostgreSQL query includes an unmapped data type or custom domain where `try_get::<String, _>(i)` fails.
2. `row.try_get_raw(i)` returns a binary-formatted value (`PgValueFormat::Binary`).
3. Fallback code calls `raw.as_bytes().ok()` and `from_utf8(bytes)`.
4. If binary bytes happen to form valid UTF-8 sequences, raw binary bytes are presented to users as garbage text.
5. If binary bytes do not form valid UTF-8, it falls back to `<unsupported value: {data_type}>`.

### Required Fix
Check `raw.format()`. Only attempt string decoding if `raw.format() == PgValueFormat::Text`. For binary payloads, return `<unsupported binary value: {data_type}>`.
