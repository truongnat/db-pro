---
schema_version: 1
id: "MEM-PLAN-schema-indexes-runtime-f46caff92b06"
title: "Plan archive: Schema Indexes Runtime"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, schema-indexes-runtime]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f46caff92b06bd810e26a50d498a6fe0ecf430776c66655233bde4e9f36ca1db"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/schema-indexes-runtime/PLAN.md"
  - "docs/plans/active/schema-indexes-runtime/FINDINGS.md"
  - "docs/plans/active/schema-indexes-runtime/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/schema-indexes-runtime/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# S2 — Schema Indexes Runtime Verification

State: RUNTIME_VERIFY
Merged implementation: PR #2 / `aa77ece`

## Goal

Index management must preserve database truth across create/list/drop operations and provider boundaries.

```text
UI index action
→ dialect-aware SQL
→ DDL command/service
→ database mutation
→ introspection
→ refreshed index UI
```

## Scope

- list indexes
- create normal index
- create unique index
- create composite index
- drop index
- provider-aware identifier quoting
- metadata/cache refresh
- PostgreSQL and SQLite runtime accounting

## Non-goals

- relations/foreign keys
- triggers
- ER diagram
- expression/partial index editor UX unless explicitly added later

## Current evidence

The merged S2 work added a SQLite integration test proving:

- `CREATE UNIQUE INDEX`
- composite `CREATE INDEX`
- `DROP INDEX`
- SQLite introspection via `PRAGMA index_list` / `PRAGMA index_info`

This is SQLite automated evidence only. It does not prove PostgreSQL runtime behavior or the UI refresh lifecycle.

## Provider matrix

| Provider | Declared support | Automated evidence | Live/UI runtime evidence |
|---|---|---|---|
| PostgreSQL | yes | source only | NOT VERIFIED |
| SQLite | yes | PASS: integration test | NOT VERIFIED (UI round-trip outstanding) |

## Completion criteria

- [x] SQLite create/unique/composite/drop introspection regression test
- [ ] PostgreSQL create/unique/composite/drop verified against a real PostgreSQL provider
- [ ] UI create/drop → refreshed index list verified
- [ ] cache invalidation verified through the user-facing lifecycle
- [ ] P0 = 0 and P1 = 0

S2 remains `RUNTIME_VERIFY` until the pending provider/UI evidence is recorded.


## Source: `FINDINGS.md`

# S2 — Schema Indexes Runtime Findings

## Confirmed implementation

- Index listing/creation/drop surfaces exist in the frontend.
- PostgreSQL and SQLite introspection paths exist.
- SQLite automated verification covers unique, composite and drop index behavior.
- DDL success invalidates schema metadata surfaces in source.

These are implementation/source claims except where the SQLite integration test is explicitly cited.

## Open evidence gaps

### P2-001 — PostgreSQL runtime evidence missing
Status: OPEN.

The merged S2 verification exercised SQLite only. PostgreSQL index introspection and DDL support are present in source, but no actual PostgreSQL runtime result is recorded.

### P2-002 — UI refresh lifecycle not runtime-verified
Status: OPEN.

Source wiring exists for query invalidation, but no recorded runtime evidence proves create/drop from the UI refreshes the visible index list without stale state.

## Rejected wording from previous plan

Claims such as "perfectly supports" and "passed flawlessly" were removed because they exceeded the evidence actually collected.

## P0/P1 gate

No accepted open P0/P1 finding is currently recorded for S2. The feature remains `RUNTIME_VERIFY` because evidence is incomplete, not because an accepted P0/P1 defect is open.


## Source: `VERIFICATION.md`

# S2 — Schema Indexes Runtime Verification

State: RUNTIME_VERIFY

## Automated evidence recorded by merged S2 work

Command recorded in the S2 implementation:

```text
cargo test --test schema_indexes_runtime_verification
```

Recorded result:

```text
running 1 test
test verify_create_and_drop_index ... ok

test result: ok. 1 passed; 0 failed
```

The test uses SQLite and proves:

1. `CREATE UNIQUE INDEX` is visible through SQLite introspection and marked unique.
2. a composite `CREATE INDEX` preserves ordered columns.
3. `DROP INDEX` removes the target index while leaving the other index intact.

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | NOT VERIFIED | NOT VERIFIED | source implementation exists; no provider runtime result recorded |
| SQLite | PASS | PARTIAL | backend integration proven; UI round-trip still pending |

## UI lifecycle

```text
UI create/drop
→ DDL service
→ database
→ cache invalidation
→ introspection
→ refreshed index list
```

Status: no runtime evidence recorded.

## Completion decision

Not completed. S2 has valid SQLite automated evidence but still requires PostgreSQL provider evidence and the user-facing refresh lifecycle before moving to `docs/plans/completed/`.

