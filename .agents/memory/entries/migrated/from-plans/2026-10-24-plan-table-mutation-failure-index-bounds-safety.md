---
schema_version: 1
id: "MEM-PLAN-table-mutation-failure-index-bou-af45c4e04909"
title: "Plan archive: Table Mutation Failure Index Bounds Safety"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, table-mutation-failure-index-bounds-safe]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "af45c4e04909fa0f0a546f7170617ad9ad2f1475820ecdf9385b96bde5fc8ace"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/table-mutation-failure-index-bounds-safety/PLAN.md"
  - "docs/plans/active/table-mutation-failure-index-bounds-safety/FINDINGS.md"
  - "docs/plans/active/table-mutation-failure-index-bounds-safety/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/table-mutation-failure-index-bounds-safety/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings: Table Mutation Transaction Failure Index Bounds Safety

1. Pre-execution failures (`Validation`, `Begin`) and post-execution failures (`Commit`) report statement indices `0` or `statements.len()`.
2. Remapping `statement_index` to original mutation input index is only valid during `TransactionFailurePhase::Statement` when `statement_index < indexed_mutations.len()`.
3. `TableMutationExecution::execute` properly enforces both conditions, preventing out-of-bounds indexing or misattribution.


## Source: `VERIFICATION.md`

# Verification: Table Mutation Transaction Failure Index Bounds Safety

## Executed Gates
- `cargo check -p db-pro-core`: PASS
- `cargo test -p db-pro-core`: PASS (508 tests passed)

