# Empty Schema Qualification Remediation — CHECKLIST

- [x] Create plan documentation under `docs/plans/active/empty-schema-qualification/`
- [x] Implement `qualify` helper and update query builders in `crates/core/src/application/sql_builder.rs`
- [x] Update `crates/core/src/application/schema_service.rs` DDL builders for empty schema
- [x] Update `crates/core/src/application/data_diff.rs` for empty schema
- [x] Update `crates/core/src/application/schema_diff.rs` for empty schema
- [x] Update `crates/infrastructure/src/postgres/cross_connection.rs` for empty schema
- [x] Add unit tests for `schema = ""` across the changed qualification paths, including dotted schema-diff names
- [x] Run Rust quality gates (`cargo test -p db-pro-core -p db-pro-infrastructure`)
- [x] Complete pre-commit steps
- [x] Publish feature branch and Pull Request
