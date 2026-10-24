---
schema_version: 1
id: "MEM-SRC-e99a65779f89b1a2"
title: "Source mirror: docs/plans/archived/qa-p2-favorite-rollback/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-favorite-rollback/PLAN.md"
source_sha256: "e99a65779f89b1a223290d5b26f41ae286a733fe3cd0efa7a1b3c1f75689b04a"
mirror_body_sha256: "e99a65779f89b1a223290d5b26f41ae286a733fe3cd0efa7a1b3c1f75689b04a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

## Lifecycle State
State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets `frontend/src/`. Native connection favorite persistence is handled in `crates/ui/`.

## Goal
Ensure optimistic favorite toggle in `useToggleFavorite` is properly rolled back via `onError` callback if backend persistence fails.

## Tasks
1. Implement `onError` handler in `useToggleFavorite` in `frontend/src/modules/connection/queries/connection.queries.ts`.
2. Add regression unit test in `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`.
3. Verify all frontend and Rust quality gates pass.
