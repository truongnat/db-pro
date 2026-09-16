# Plan — Table Data Mutation Failure Index Mapping

## Baseline

- Target branch: `fix/table-data-mutation-index-mapping`
- Issue severity: P1 (Wrong mutation failure attribution)
- Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Objective

Ensure `TableDataService::apply_mutations_detailed` maps `TransactionFailure::statement_index` back to the caller's original `mutations` slice index rather than the internal reordered statement index (Delete -> Update -> Insert).

## Scope

1. Verify `TableDataService::apply_mutations_detailed` in `crates/core/src/application/table_data_service.rs` tracks original indices during mutation sorting (`indexed_mutations`) and remaps `failure.statement_index` back to original caller mutation index.
2. Add comprehensive unit tests verifying complex reordering scenarios (e.g. `[Insert 1, Update 1, Delete 1, Insert 2]`) to prove error index attribution correctness.
3. Verify all Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, native release build, clean code scan).
