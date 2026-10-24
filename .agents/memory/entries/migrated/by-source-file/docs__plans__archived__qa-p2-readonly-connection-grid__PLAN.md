---
schema_version: 1
id: "MEM-SRC-496266c28d9c4910"
title: "Source mirror: docs/plans/archived/qa-p2-readonly-connection-grid/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-readonly-connection-grid/PLAN.md"
source_sha256: "496266c28d9c4910480699e95eaf30d01c51dd0e904ef436f61bfa57a76b569d"
mirror_body_sha256: "496266c28d9c4910480699e95eaf30d01c51dd0e904ef436f61bfa57a76b569d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# PLAN — QA-P2-08 Read-Only Connection Data Grid Visual Affordance

## Severity
P2 (UX & Mutation Mismatch)

## State
State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets `frontend/src/`. Readonly connection handling for data grid in native egui is implemented in `crates/ui/`.

## Context
When a connection is marked as `readonly: true`, the backend `TableDataService` correctly rejects mutation requests (updates and deletes). However, `DataSection` and `DataGrid` in the frontend determine grid editability solely based on whether primary keys exist (`pkColumns.length > 0`).

As a result:
1. The read-only warning banner is not displayed for readonly connections when primary keys exist.
2. The user is allowed to stage cell edits, row deletes, batch deletes, or edit via row edit dialog.
3. Upon clicking "Apply", the backend returns an error (`Connection is read-only`), resulting in a confusing user experience where the UI allowed staging mutations that were guaranteed to fail.

## Proposed Changes
1. Look up connection `readonly` status in `DataSection` from `useConnectionStore`.
2. Guard staged mutation handlers (`handleCellSave`, `handleDeleteRow`, `handleBatchDelete`, `handleRowSave`) when `isReadonlyConnection` is true.
3. Pass `isReadonlyConnection` to `DataGrid` and compute `canEdit = pkColumns.length > 0 && !isReadonlyConnection`.
4. Render a read-only connection warning banner in `DataGrid` when `isReadonlyConnection` is true.
5. Add unit tests for `DataGrid` testing `isReadonlyConnection`.

## Verification Strategy
- Run frontend unit tests (`npm test`).
- Run frontend quality gates (`npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`).
- Run backend Rust quality gates (`cargo test -p db-pro-core -p db-pro-infrastructure`).
