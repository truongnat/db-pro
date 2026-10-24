---
schema_version: 1
id: "MEM-SRC-c5eb7f80bd0179a6"
title: "Source mirror: docs/plans/active/schema-columns-runtime/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/schema-columns-runtime/PLAN.md"
source_sha256: "c5eb7f80bd0179a6c76494a1b0c233d07a538021833389c25817b1fb5be8240f"
mirror_body_sha256: "c5eb7f80bd0179a6c76494a1b0c233d07a538021833389c25817b1fb5be8240f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
