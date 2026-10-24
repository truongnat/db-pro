---
schema_version: 1
id: "MEM-SRC-acf1a26646cc669f"
title: "Source mirror: docs/plans/archived/qa-p1-10-orphan-tab-close-guard/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/qa-p1-10-orphan-tab-close-guard/FINDINGS.md"
source_sha256: "acf1a26646cc669f8389ca64b5f6dcf521e9608051f642659c58b8a9b80d1f67"
mirror_body_sha256: "acf1a26646cc669f8389ca64b5f6dcf521e9608051f642659c58b8a9b80d1f67"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — QA-P1-10 Orphan Tab Dirty Close Guard

## Problem Statement
In `frontend/src/commons/components/workspace-content.tsx`, `OrphanedTabView` directly called `useWorkspaceStore.getState().closeTab(tabId)`. This bypassed `requestCloseTab(tabId)` and its shared dirty check guard (`hasUnsavedWork(tabId)`), which inspects both `tab.dirty` and staged Data Grid revisions (`useStagedChangesStore`).

## Failure Scenario
1. User opens a query tab or table tab, edits SQL or stages Data Grid edits.
2. Connection becomes invalid/removed or context changes, causing the tab to display `OrphanedTabView`.
3. User clicks "Close".
4. Tab closes immediately without warning, discarding unsaved SQL or staged mutations.

## Severity
P1: Unsaved user work loss and state guard bypass.
