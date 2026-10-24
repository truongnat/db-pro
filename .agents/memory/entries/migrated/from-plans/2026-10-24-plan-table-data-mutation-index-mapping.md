---
schema_version: 1
id: "MEM-PLAN-table-data-mutation-index-mappin-4773348114fd"
title: "Plan archive: Table Data Mutation Index Mapping"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, table-data-mutation-index-mapping]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4773348114fd926d5f1a379e4495c6e72585c350bcdafb7be41bcb2ec6331480"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/table-data-mutation-index-mapping/PLAN.md"
  - "docs/plans/completed/table-data-mutation-index-mapping/FINDINGS.md"
  - "docs/plans/completed/table-data-mutation-index-mapping/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/table-data-mutation-index-mapping/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — Table Data Mutation Failure Index Mapping

## Baseline

- Target branch: `fix/table-data-mutation-index-mapping`
- Issue severity: P1 (Wrong mutation failure attribution / out-of-bounds index access)
- Canonical Lifecycle: `COMPLETED`

## Objective

Fix `TableMutationExecution` in `crates/core/src/application/table_mutation_execution.rs` so that:
1. When a batch transaction statement fails during execution (`phase == Statement`), `TransactionFailure::statement_index` is mapped back to the caller's original `mutations` slice index rather than the internal reordered statement index (Delete -> Update -> Insert).
2. Pre-execution failures (`phase == Begin` or `phase == Validation`) retain `statement_index = 0` (reflecting zero statements executed) and do not falsely attribute pre-execution failures to reordered statement indices.
3. `validation_failure` sets `statement_index = 0` instead of `self.mutations.len()`, preventing out-of-bounds array access.

## Scope

1. Update `TableMutationExecution` in `crates/core/src/application/table_mutation_execution.rs`:
   - Set `statement_index: 0` in `validation_failure`.
   - Gate index remapping on `failure.phase == TransactionFailurePhase::Statement`.
2. Add unit tests in `crates/core/src/application/table_data_service.rs` verifying:
   - `Validation` phase failures return `statement_index: 0`.
   - `Begin` phase failures retain `statement_index: 0` without remapping.
   - `Statement` phase failures correctly remap `statement_index` to original input mutation indices, including the two-mutation `[Insert, Delete]` case and the four-mutation `[Insert, Update, Delete, Insert]` reordering.
3. Verify Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

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

