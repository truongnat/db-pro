# Plan — Table Data Mutation Failure Index Mapping

## Baseline

- Target branch: `fix/table-data-mutation-index-mapping`
- Issue severity: P1 (Wrong mutation failure attribution)
- Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Objective

Fix `TableDataService::apply_mutations_detailed` so that when a batch transaction statement fails, `TransactionFailure::statement_index` is mapped back to the caller's original `mutations` slice index rather than the internal reordered statement index.

## Scope

1. Update `TableDataService::apply_mutations_detailed` in `crates/core/src/application/table_data_service.rs` to track original indices during mutation sorting.
2. Remap `failure.statement_index` in the `Err(failure)` path when `execute_parameterized_transaction` returns a `TransactionFailure`.
3. Add a unit test verifying that `statement_index` correctly references the original input mutation position when mutations are reordered.
