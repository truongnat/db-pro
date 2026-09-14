# Findings — Table Data Mutation Failure Index Mapping

## Severity: P1 (Incorrect Data Mutation Error Attribution)

## Evidence

In `crates/core/src/application/table_data_service.rs`:

```rust
let mut ordered = mutations.to_vec();
ordered.sort_by_key(|mutation| match mutation {
    TableDataMutation::Delete { .. } => 0,
    TableDataMutation::Update { .. } => 1,
    TableDataMutation::Insert { .. } => 2,
});
```

`TableDataService::apply_mutations_detailed` reorders `mutations` so that `Delete` statements execute before `Update` statements, which execute before `Insert` statements.

However, when `execute_parameterized_transaction` returns a `TransactionFailure`, `failure.statement_index` reflects the 0-based index in the `ordered` vector (the execution order), NOT the index in the original `mutations` slice passed by the caller (`RuntimeCommand::ApplyTableChanges` / native UI staged changes).

In `crates/runtime/src/worker.rs`:
`RuntimeEvent::TableChangesFailed` emits `statement_index: failure.statement_index`, which the UI uses to highlight or handle the specific staged change that failed.

## Failure Scenario

1. User stages an `Insert` change at index 0 (e.g. inserting a row with a duplicate key).
2. User stages a `Delete` change at index 1 (deleting an old row).
3. UI sends `mutations = [Insert, Delete]`.
4. `TableDataService::apply_mutations_detailed` reorders them to `[Delete (pos 0), Insert (pos 1)]`.
5. Statement at execution index 1 (`Insert`) fails with a unique constraint error.
6. `apply_mutations_detailed` returns `TransactionFailure` with `statement_index = 1`.
7. UI receives `statement_index = 1` and marks the `Delete` change (original index 1) as the failing mutation instead of the `Insert` change (original index 0).

## Scope of Fix

Map `statement_index` in `apply_mutations_detailed` back to the index in the original input slice.
