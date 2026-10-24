---
schema_version: 1
id: "MEM-SRC-948d9332be1decf7"
title: "Source mirror: docs/plans/archived/connection-list-reconnect-sideeffect/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/connection-list-reconnect-sideeffect/PLAN.md"
source_sha256: "948d9332be1decf71ac4c4f884fc852148fa759e2a2ebc4499ee7b668195b571"
mirror_body_sha256: "948d9332be1decf71ac4c4f884fc852148fa759e2a2ebc4499ee7b668195b571"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — QA-P1-09 Connection-list query reconnect side-effect decoupling

State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets the archived React frontend (`_archive/frontend/`). Native connection lifecycle is handled in `crates/ui/`.

## Goal
Decouple session restoration side effects from `useConnectionList` fetching logic to ensure fetching connection lists remains pure and side-effect free on generic refetches.

## Architecture & Principles
- React Query `queryFn` should be a pure query operation without hidden reconnect side effects.
- Startup session restoration must be explicitly controlled via a dedicated one-shot coordinator or startup component / hook.
- Both PostgreSQL and SQLite connections must be handled consistently during restoration without unnecessary reconnect attempts.

## Tasks
- [ ] Create active plan documentation.
- [ ] Refactor `restoreSession` and `useConnectionList` in `frontend/src/modules/connection/queries/connection.queries.ts`.
- [ ] Add/update tests in `frontend/src/modules/connection/__tests__/`.
- [ ] Execute quality gates.
- [ ] Publish PR.
