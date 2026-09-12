# Verification: Fix PostgreSQL Binary-Protocol Textual Value Decoding

## Verification Steps
1. Unit tests in `crates/infrastructure/src/postgres/query_mapper.rs` verifying textual decoding for both text and binary formats.
2. `cargo test -p db-pro-core -p db-pro-infrastructure` passes with 0 failures.
3. `cargo check --workspace` passes cleanly.
