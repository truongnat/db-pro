---
schema_version: 1
id: "MEM-PLAN-ui-files-ide-workspace-polish-06e043e7d33f"
title: "Plan archive: Ui Files Ide Workspace Polish"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-files-ide-workspace-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "06e043e7d33fce74fbedb9c02321e76422080407feef1e3e33238fd5e422bf0b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-files-ide-workspace-polish/PLAN.md"
  - "docs/plans/active/ui-files-ide-workspace-polish/FINDINGS.md"
  - "docs/plans/active/ui-files-ide-workspace-polish/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-files-ide-workspace-polish/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — UI09 Files/IDE Workspace Polish

1. `files_activity_view.rs` used numerous legacy button helpers (`compact_button`, `compact_icon_button`, `secondary_button_with_icon`, `ghost_button_with_icon`, `danger_button`).
2. Standardizing all of these to `Button` with consistent variants (`Default`, `Secondary`, `Ghost`, `Destructive`), sizes (`Sm`, `IconSm`), and tooltips creates visual harmony across the entire IDE lane.


## Source: `VERIFICATION.md`

# Verification — UI09 Files/IDE Workspace Polish

## Test Matrix

- `cargo fmt --all -- --check` (PASSED)
- `cargo check --workspace` (PASSED)
- `cargo clippy --workspace --all-targets -- -D warnings` (PASSED)
- `cargo test --workspace` (PASSED: 483 tests passed in db-pro-ui, all infrastructure and core unit/integration tests passed)
- `cargo build --release --locked -p db-pro-native` (PASSED)
- Primary surface raw button check passes for `files_activity_view.rs` and `tasks_view.rs`.

