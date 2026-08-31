# CHECKLIST — PostgreSQL Non-Basic Provider Type Mapping (QA-P1-14)

- [x] Create active plan documentation under `docs/plans/active/provider-type-mapping/`
- [x] Implement NUMERIC/DECIMAL, MONEY, TIMETZ, INTERVAL, INET/CIDR, BIT/VARBIT, TSVECTOR/TSQUERY decoding in `crates/infrastructure/src/postgres/query_mapper.rs`
- [x] Implement safe PostgreSQL array type decoding using `Vec<Option<T>>`
- [x] Guard `PgValueRef::as_str()` invocation with `PgValueFormat::Text` check
- [x] Add unit tests in `query_mapper.rs` for parameter binding and array type integrity
- [x] Execute Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
- [x] Complete pre-commit verification steps
