---
schema_version: 1
id: "MEM-SRC-63ccc5380aa2d6db"
title: "Source mirror: docs/plans/active/table-mutation-failure-index-bounds-safety/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/table-mutation-failure-index-bounds-safety/PLAN.md"
source_sha256: "63ccc5380aa2d6db414eb2685d3fd3dfd49b64d3a11bc3165e371a1e388843b7"
mirror_body_sha256: "63ccc5380aa2d6db414eb2685d3fd3dfd49b64d3a11bc3165e371a1e388843b7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
