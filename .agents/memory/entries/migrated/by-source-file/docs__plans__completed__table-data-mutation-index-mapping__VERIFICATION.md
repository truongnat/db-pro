---
schema_version: 1
id: "MEM-SRC-8eae1e40ddcdcec3"
title: "Source mirror: docs/plans/completed/table-data-mutation-index-mapping/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/table-data-mutation-index-mapping/VERIFICATION.md"
source_sha256: "8eae1e40ddcdcec31e8f3a054c54e63c0b07367c0ba76d814dc94d0cffd43098"
mirror_body_sha256: "8eae1e40ddcdcec31e8f3a054c54e63c0b07367c0ba76d814dc94d0cffd43098"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — Table Data Mutation Failure Index Mapping

## Unit Test Coverage

Added unit tests in `crates/core/src/application/table_data_service.rs`:

1. `apply_mutations_detailed_maps_statement_index_to_original_input_mutation`:
   - Verifies that when execution fails on reordered statement at index 1 (`Insert`), `statement_index` is mapped back to original input index 0 (`Insert`).

2. `apply_mutations_detailed_maps_statement_index_complex_reordering`:
   - Verifies `[Insert, Update, Delete, Insert]` reordering. Execution order is `[Delete, Update, Insert, Insert]`. A failure at exec index 0 (`Delete`) maps back to original input index 2.

3. `apply_mutations_detailed_retains_zero_statement_index_on_begin_failure`:
   - Verifies that when `execute_parameterized_transaction` fails with `phase: Begin`, `statement_index` remains `0` and is NOT remapped to `indexed_mutations[0].0`.

4. `apply_mutations_detailed_retains_zero_statement_index_on_validation_failure`:
   - Verifies that when `apply_mutations_detailed` fails with `phase: Validation` (e.g. read-only connection), `statement_index` is `0` (not `self.mutations.len()`).

## Automated Execution

Commands executed after merging `origin/main`:

1. `cargo test -p db-pro-core --lib application::table_data_service`
   - Result: 25 passed / 0 failed (includes all four index-mapping tests above).

2. `cargo test -p db-pro-core`
   - Result: 407 passed / 0 failed.

3. `cargo clippy -p db-pro-core --all-targets -- -D warnings`
   - Result: Pass (exit 0).

4. `rustfmt --check crates/core/src/application/table_data_service.rs crates/core/src/application/table_mutation_execution.rs`
   - Result: Pass (exit 0). Workspace `cargo fmt --all -- --check` still reports pre-existing UI formatting drift from `main`; this merge does not reformat those files.
