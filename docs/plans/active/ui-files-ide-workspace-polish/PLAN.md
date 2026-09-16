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
