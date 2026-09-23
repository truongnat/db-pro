# Plan — Table Data Mutation Failure Index Mapping

## Baseline

- Target branch: `fix/table-data-mutation-index-mapping`
- Issue severity: P1 (Wrong mutation failure attribution / out-of-bounds index access)
- Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Objective

Fix `TableMutationExecution` in `crates/core/src/application/table_mutation_execution.rs` so that:
1. When a batch transaction statement fails during execution (`phase == Statement`), `TransactionFailure::statement_index` is mapped back to the caller's original `mutations` slice index rather than the internal reordered statement index.
2. Pre-execution failures (`phase == Begin` or `phase == Validation`) retain `statement_index = 0` (reflecting zero statements executed) and do not falsely attribute pre-execution failures to reordered statement indices.
3. `validation_failure` sets `statement_index = 0` instead of `self.mutations.len()`, preventing out-of-bounds array access.

## Scope

1. Update `TableMutationExecution` in `crates/core/src/application/table_mutation_execution.rs`:
   - Set `statement_index: 0` in `validation_failure`.
   - Gate index remapping on `failure.phase == TransactionFailurePhase::Statement`.
2. Add unit tests in `crates/core/src/application/table_data_service.rs` verifying:
   - `Validation` phase failures return `statement_index: 0`.
   - `Begin` phase failures retain `statement_index: 0` without remapping.
   - `Statement` phase failures correctly remap `statement_index` to original input mutation indices.
