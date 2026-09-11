# PLAN — PostgreSQL Query Mapper Fallback Binary Decoding Correctness

## Overview
Fix unsafe binary payload UTF-8 string conversion during fallback decoding in `crates/infrastructure/src/postgres/query_mapper.rs`.

## Lifecycle State
`IMPLEMENTING`

## Problem Statement
In `crates/infrastructure/src/postgres/query_mapper.rs`, `decode_cell()` decodes result cells based on column type name. If typed decoding fails or if the data type is unrecognized, it executes fallback logic:

```rust
row.try_get_raw(i)
    .ok()
    .and_then(|raw| raw.as_bytes().ok())
    .and_then(|bytes| std::str::from_utf8(bytes).ok())
```

When sqlx communicates with PostgreSQL using binary protocol format (`PgValueFormat::Binary`), raw column value bytes represent binary-encoded data (e.g. big-endian integers, IEEE floats, binary timestamps, UUID bytes), not UTF-8 text. Attempting `std::str::from_utf8` on raw binary bytes can interpret raw byte sequences as text or corrupt strings.

## Proposed Fix
In `decode_cell()` fallback logic:
1. Inspect `raw.format()`.
2. If `raw.format() == PgValueFormat::Text`, safely convert text bytes to string.
3. If `raw.format() == PgValueFormat::Binary`, do NOT attempt raw UTF-8 parsing on binary bytes; return `<unsupported binary value: {data_type}>`.

## Tasks
- [ ] Implement `raw.format()` check in fallback path of `decode_cell()`
- [ ] Add unit test for fallback logic under text and binary format scenarios
- [ ] Run Rust unit tests (`cargo test -p db-pro-core -p db-pro-infrastructure`)
- [ ] Run Frontend quality gates (`pnpm run typecheck`, `pnpm run test`)
