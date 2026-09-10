# Empty Schema Qualification Remediation — CHECKLIST

- [ ] Create plan documentation under `docs/plans/active/empty-schema-qualification/`
- [ ] Implement `qualify` helper and update query builders in `crates/core/src/application/sql_builder.rs`
- [ ] Update `crates/core/src/application/schema_service.rs` DDL builders for empty schema
- [ ] Update `crates/core/src/application/data_diff.rs` for empty schema
- [ ] Update `crates/core/src/application/schema_diff.rs` for empty schema
- [ ] Update `crates/infrastructure/src/postgres/cross_connection.rs` for empty schema
- [ ] Add unit tests for `schema = ""` across `sql_builder`, `schema_service`, `data_diff`, `schema_diff`
- [ ] Run Rust quality gates (`cargo test -p db-pro-core -p db-pro-infrastructure`)
- [ ] Complete pre-commit steps
- [ ] Publish feature branch and Pull Request
