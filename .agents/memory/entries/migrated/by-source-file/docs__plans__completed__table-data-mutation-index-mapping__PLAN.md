---
schema_version: 1
id: "MEM-SRC-166f7c2bc17b6587"
title: "Source mirror: docs/plans/completed/table-data-mutation-index-mapping/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/table-data-mutation-index-mapping/PLAN.md"
source_sha256: "166f7c2bc17b658755337995314a3c146e0bc0d3ddea5f70cd2cce4cb498d822"
mirror_body_sha256: "166f7c2bc17b658755337995314a3c146e0bc0d3ddea5f70cd2cce4cb498d822"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
