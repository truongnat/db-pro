# Feature Plan: Table Mutation Transaction Failure Index Bounds Safety

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `TableMutationExecution::execute` (`crates/core/src/application/table_mutation_execution.rs`), batch transaction statement errors remap failure `statement_index` values back to original input mutation positions.

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, error propagation, out-of-bounds index mapping prevention).
- **Evidence**: `crates/core/src/application/table_mutation_execution.rs` lines 67–74:
```rust
if failure.phase == TransactionFailurePhase::Statement
    && failure.statement_index < indexed_mutations.len()
{
    failure.statement_index = indexed_mutations[failure.statement_index].0;
}
```
If `failure.phase` is `Commit` (`statement_index = statements.len()`), `Begin` (`statement_index = 0`), or `Validation` (`statement_index = 0`), statement index remapping must be skipped so out-of-bounds indexing or false mutation index attribution does not occur.

## Scope & Implementation
1. Ensure `TableMutationExecution::execute` guards statement index remapping with `failure.phase == TransactionFailurePhase::Statement && failure.statement_index < indexed_mutations.len()`.
2. Add regression test `apply_mutations_detailed_retains_commit_statement_index_without_out_of_bounds_remapping` in `crates/core/src/application/tests/table_data_service_tests.rs`.

## Verification
- Core unit test suite `cargo test -p db-pro-core` passes with 508 passing tests.
