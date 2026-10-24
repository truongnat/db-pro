---
schema_version: 1
id: "MEM-SRC-848eb37cd7ee9403"
title: "Source mirror: docs/plans/completed/empty-schema-qualification/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/empty-schema-qualification/VERIFICATION.md"
source_sha256: "848eb37cd7ee9403d7c820b803b9500009d05d566cd0585c1cee65084621291d"
mirror_body_sha256: "848eb37cd7ee9403d7c820b803b9500009d05d566cd0585c1cee65084621291d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
