# Empty Schema Qualification — VERIFICATION

## Test Strategy

1. **Unit Verification (`crates/core/src/application/sql_builder.rs`)**:
   - Test `build_select`, `build_count`, `build_insert`, `build_update`, `build_delete` with `schema = ""`.
   - Assert generated SQL contains `"users"` (or `` `users` ``) without `""."users"`.

2. **Unit Verification (`crates/core/src/application/schema_service.rs`)**:
   - Test `build_create_table_ddl` with `schema = ""`.
   - Assert generated CREATE TABLE SQL does not contain `""."tablename"`.

3. **Unit Verification (`crates/core/src/application/data_diff.rs` & `schema_diff.rs`)**:
   - Test data diff and schema diff when `schema` is `""`.

4. **Integration Gate Execution**:
   - `cargo test -p db-pro-core -p db-pro-infrastructure`
