---
schema_version: 1
id: "MEM-SRC-c0b2a27376a284dc"
title: "Source mirror: docs/plans/active/ui-files-ide-workspace-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-files-ide-workspace-polish/PLAN.md"
source_sha256: "c0b2a27376a284dc8c9c7a01345d9113f347fc2928349014fbbc9e350f346c64"
mirror_body_sha256: "c0b2a27376a284dc8c9c7a01345d9113f347fc2928349014fbbc9e350f346c64"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — UI09 Files/IDE Workspace Tree, Editor Tabs, Problems, Tasks, and Local-Workspace Polish (#295)

## Objective
Standardize all Files/IDE activity surfaces, workspace folder management, multi-root actions, search/replace, migrations, snapshot diffs, sandbox runners, git staging actions, and task manager action buttons using canonical `Button` primitives and semantic design tokens.

## Scope
1. **Files Activity View (`files_activity_view.rs`)**:
   - Workspace open / refresh / multi-root add & remove buttons.
   - Active tab close and context actions (delete, diff, snapshot, split).
   - Scratch buffer actions (+ File, + Selection, + Table, New SQL, New Folder).
   - Search & replace / refactor action buttons.
   - Sandbox runner & test suite runner action buttons.
   - Git staging, unstaging, diff, open, and commit action buttons.
   - External file change reload and dismiss actions.
2. **Tasks View (`tasks_view.rs`)**:
   - New SQL task and New backup task buttons.
3. **Non-Regression Coverage (`crates/ui/src/components/mod.rs`)**:
   - Add `files_activity_view.rs` to `primary_native_surfaces_do_not_reintroduce_raw_buttons`.
