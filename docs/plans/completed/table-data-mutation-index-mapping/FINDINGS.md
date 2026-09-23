# Findings — Table Data Mutation Failure Index Mapping

## Severity: P1 (Incorrect Data Mutation Error Attribution)

## Evidence

In `crates/core/src/application/table_mutation_execution.rs`:

```rust
let mut indexed_mutations: Vec<_> = self.mutations.iter().enumerate().collect();
indexed_mutations.sort_by_key(|(_, mutation)| match mutation {
    TableDataMutation::Delete { .. } => 0,
    TableDataMutation::Update { .. } => 1,
    TableDataMutation::Insert { .. } => 2,
});
```

`TableMutationExecution` reorders `mutations` so that `Delete` statements execute before `Update` statements, which execute before `Insert` statements.

When `execute_parameterized_transaction` returns a `TransactionFailure` with `phase == TransactionFailurePhase::Statement`, `failure.statement_index` reflects the 0-based index in the `indexed_mutations` vector (the execution order), NOT the index in the original `mutations` slice passed by the caller (`RuntimeCommand::ApplyTableChanges` / native UI staged changes).

Furthermore, when `execute_parameterized_transaction` returns a `TransactionFailure` with `phase == TransactionFailurePhase::Begin` or `phase == TransactionFailurePhase::Validation`, `failure.statement_index` is `0` (indicating zero statements executed). Without gating remapping on `phase == TransactionFailurePhase::Statement`, `failure.statement_index` was incorrectly remapped to `indexed_mutations[0].0` (e.g., original index of a reordered Delete mutation), falsely attributing a pre-execution failure to a specific mutation.

Lastly, `validation_failure` previously set `statement_index = self.mutations.len()`, which produced an out-of-bounds index for an N-element mutations slice.

In `crates/runtime/src/worker.rs`:
`RuntimeEvent::TableChangesFailed` emits `statement_index: failure.statement_index`, which the UI uses to highlight or handle the specific staged change that failed.

## Failure Scenarios

1. **Statement Failure Remapping**:
   - User stages an `Insert` change at index 0 (e.g. inserting a row with a duplicate key).
   - User stages a `Delete` change at index 1 (deleting an old row).
   - UI sends `mutations = [Insert, Delete]`.
   - `TableMutationExecution` reorders them to `[Delete (pos 0), Insert (pos 1)]`.
   - Statement at execution index 1 (`Insert`) fails with a unique constraint error.
   - `apply_mutations_detailed` returns `TransactionFailure` with `statement_index = 1`.
   - UI receives `statement_index = 1` and marks the `Delete` change (original index 1) as the failing mutation instead of the `Insert` change (original index 0).

2. **Validation / Begin Failure Non-Remapping**:
   - Connector returns `TransactionFailure` with `phase: Begin`, `statement_index: 0`.
   - Without phase gating, `statement_index` was mapped to `indexed_mutations[0].0` (e.g. index 1), falsely attributing the Begin failure to the first reordered mutation statement.

3. **Validation Failure Out-of-Bounds Index**:
   - Read-only safety policy failure produces `TransactionFailure` with `phase: Validation` and `statement_index = self.mutations.len()` (e.g. 2 for 2 mutations).
   - Callers indexing `mutations[statement_index]` hit an out-of-bounds panic/error.

## Scope of Fix

1. Gate `statement_index` remapping on `failure.phase == TransactionFailurePhase::Statement`.
2. Map `statement_index` back to original input mutation index when `phase == Statement`.
3. Set `statement_index = 0` for `validation_failure` (reflecting zero statements executed).
