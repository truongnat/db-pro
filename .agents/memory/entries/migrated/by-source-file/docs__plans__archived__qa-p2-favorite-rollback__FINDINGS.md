---
schema_version: 1
id: "MEM-SRC-d428fd56995e2468"
title: "Source mirror: docs/plans/archived/qa-p2-favorite-rollback/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-favorite-rollback/FINDINGS.md"
source_sha256: "d428fd56995e2468f26d39f7b01014e2568bed3b76d2b9b405616d9aa4a70a7b"
mirror_body_sha256: "d428fd56995e2468f26d39f7b01014e2568bed3b76d2b9b405616d9aa4a70a7b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

## Problem Description
In `frontend/src/modules/connection/queries/connection.queries.ts`, `useToggleFavorite` performs an optimistic UI update (`toggleFavoriteLocal(id)`) in `onMutate`. However, if the server mutation fails, there is no `onError` callback, leaving the UI state out of sync with backend persistence.

## Severity
P2 — UI state desynchronization on mutation failure.
