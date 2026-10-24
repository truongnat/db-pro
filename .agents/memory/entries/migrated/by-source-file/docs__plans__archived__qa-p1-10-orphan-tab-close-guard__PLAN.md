---
schema_version: 1
id: "MEM-SRC-94e1578d90b5f3b9"
title: "Source mirror: docs/plans/archived/qa-p1-10-orphan-tab-close-guard/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p1-10-orphan-tab-close-guard/PLAN.md"
source_sha256: "94e1578d90b5f3b90f02c387f7ff71ec345d49488d588b591120c80cc2828ea7"
mirror_body_sha256: "94e1578d90b5f3b90f02c387f7ff71ec345d49488d588b591120c80cc2828ea7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — QA-P1-10 Orphan Tab Dirty Close Guard

## Goal
Ensure closing an orphaned tab (a tab referencing a missing connection) routes through the shared dirty close guard (`requestCloseTab`), preserving unsaved query edits or staged Data Grid mutations instead of bypassing confirmation.

## Lifecycle
State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets `frontend/src/`. Native egui tab close guards and orphan recovery are implemented in `crates/ui/`.

## Scope
1. Update `OrphanedTabView` in `frontend/src/commons/components/workspace-content.tsx` to call `requestCloseTab(tabId)`.
2. Add regression tests in `frontend/src/commons/__tests__/request-close-tab.test.ts`.
3. Verify all frontend quality gates pass.
