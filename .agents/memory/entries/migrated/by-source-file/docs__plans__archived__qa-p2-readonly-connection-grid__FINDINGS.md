---
schema_version: 1
id: "MEM-SRC-f80c1eb9e5379b56"
title: "Source mirror: docs/plans/archived/qa-p2-readonly-connection-grid/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p2-readonly-connection-grid/FINDINGS.md"
source_sha256: "f80c1eb9e5379b5657fbc12be06f828c2e9190d8d4c1c34ab752c350837d44df"
mirror_body_sha256: "f80c1eb9e5379b5657fbc12be06f828c2e9190d8d4c1c34ab752c350837d44df"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# FINDINGS — QA-P2-08 Read-Only Connection Data Grid Visual Affordance

## Severity: P2

## Finding
`DataSection` and `DataGrid` in the frontend checked `pkColumns.length > 0` to decide if cell editing / row deletion is allowed, but ignored `connection.readonly`.

## Impact
Users connected to a read-only database connection could double-click cells, stage cell edits, stage row deletes, and open row edit dialogs. Applying these staged changes called the backend API which failed with "Connection is read-only".

## Remediation
1. `DataSection` reads `readonly` from `useConnectionStore`.
2. Guarded staged mutation callbacks in `DataSection` if `isReadonlyConnection` is true.
3. Updated `DataGrid` to calculate `canEdit = pkColumns.length > 0 && !isReadonlyConnection`.
4. Rendered connection read-only banner (`dataGrid.readOnlyConnection`).
5. Added unit tests and verified all quality gates.
