---
schema_version: 1
id: "MEM-PLAN-schema-columns-runtime-aacecdc553db"
title: "Plan archive: Schema Columns Runtime"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-columns-runtime]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "aacecdc553dbc24a4433c25167ca203776d12c8fd3090301dabf8e0abe3a9daa"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-columns-runtime/PLAN.md"
  - "docs/plans/active/schema-columns-runtime/FINDINGS.md"
  - "docs/plans/active/schema-columns-runtime/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-columns-runtime/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S1 — Schema Columns Runtime Verification

State: RUNTIME_VERIFY
Baseline: `fb91c4e`
Implementation commit: `741a18d`

## Goal

Column mutation must satisfy:

```text
UI draft
→ deterministic SQL preview
→ risk classification
→ atomic execution
→ database truth
→ fresh introspection
→ all UI surfaces synchronized
```

## Scope

- rename
- data type change (widening/narrowing)
- nullable toggle
- default value (literal/expression)
- combined mutations
- rollback on failure
- cache invalidation: introspect + tableInfo + tableDdl + dependencies + catalog
- PostgreSQL/SQLite capability behavior

## Non-goals

- indexes
- relations
- triggers
- ER diagram

## Implemented source contract

- `execute_batch()` exists on `DbConnector`
- PostgreSQL batch path uses a transaction
- SQLite batch path uses a transaction
- frontend uses one batch mutation for combined column changes
- schema caches are invalidated after DDL

These are source/implementation claims, not live runtime evidence.

## Provider matrix

| Provider | Declared support | Automated evidence | Live/runtime evidence |
|---|---|---|---|
| PostgreSQL | partial/provider-dependent ALTER support | source coverage | NOT VERIFIED |
| SQLite | limited ALTER support; capability-gating required | source coverage | NOT VERIFIED |

## Completion criteria

- [ ] rollback regression test proves no partial persistence after a middle-statement failure
- [ ] PostgreSQL combined mutation verified on a live provider
- [ ] PostgreSQL failing/narrowing mutation proves rollback
- [ ] SQLite operation matrix verifies supported vs capability-gated operations
- [ ] DDL/tableInfo/dependency surfaces refresh after mutation
- [ ] P0 = 0 and P1 = 0

S1 remains `RUNTIME_VERIFY` until these items are proven.


## Source: `FINDINGS.md`

# S1 — Schema Columns Runtime Findings

## Resolved source findings

### P1-001 — Multi-statement schema mutation was non-atomic
Status: FIXED in `741a18d`.

Combined column mutations previously executed statements sequentially. The implementation now routes the batch through connector transaction support.

### P1-002 — Schema cache invalidation was incomplete
Status: FIXED in `741a18d`.

The implementation now invalidates introspection, table info, table DDL, dependencies and the frontend schema catalog.

## Open evidence gaps

### P2-001 — Atomic rollback regression execution is pending
Status: TEST RESTORED; EXECUTION NOT YET VERIFIED.

`crates/infrastructure/tests/schema_columns_atomicity_regression.rs` now explicitly verifies that a valid first DDL statement does not persist when the second statement fails, and that later statements never execute. This becomes automated evidence only after the test command actually passes.

### P2-002 — Provider runtime matrix is incomplete
Status: OPEN.

PostgreSQL and SQLite live behavior has not been independently recorded for the required mutation matrix. Source support is not sufficient evidence.

### P2-003 — Cache/UI runtime proof is incomplete
Status: OPEN.

The invalidation paths exist in source, but the DDL/tableInfo/dependency surfaces have not been recorded as runtime evidence after a real mutation.

## P0/P1 gate

No currently accepted open P0/P1 finding is recorded in this plan. Completion is still blocked by runtime evidence gaps above.


## Source: `VERIFICATION.md`

# S1 — Schema Columns Runtime Verification

State: RUNTIME_VERIFY

## Existing evidence

Source inspection records that the column mutation path is designed as:

```text
UI edit
→ classify mutation
→ execute DDL batch
→ connector transaction
→ cache invalidation
→ query refetch
```

Historical test evidence recorded in the previous plan:
- `cargo check --workspace` — PASS at the time recorded
- frontend Vitest — 1291 tests / 103 files PASS at the time recorded

These historical results are retained as historical evidence only; they are not a claim about the current branch.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | PARTIAL/source | NOT VERIFIED | combined mutation + failure rollback still required |
| SQLite | PARTIAL/source | NOT VERIFIED | supported vs capability-gated ALTER paths still required |

## Required live/runtime evidence

- [ ] PostgreSQL: rename + type + nullable combined → all requested changes applied
- [ ] PostgreSQL: failing/narrowing change → no partial mutation persisted
- [ ] SQLite: supported mutation operations succeed
- [ ] SQLite: unsupported operations are blocked by capability rules rather than invalid SQL
- [ ] DDL viewer reflects changed table definition
- [ ] tableInfo and dependencies refresh after mutation

## Completion decision

Not completed. The previous completed classification was inconsistent with the still-pending runtime matrix, so S1 is reopened as `RUNTIME_VERIFY`.

