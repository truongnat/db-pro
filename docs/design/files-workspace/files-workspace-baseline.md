# Files Workspace Baseline

This document records the source-observed feature set of the **Files** activity in the native DB Pro UI. It is an analysis baseline for later Files-workspace feature design, not UI runtime evidence and not a commitment to implement every stored workspace capability.

The distinction used throughout:

- **Function**: a rendering, mapping, or workspace operation.
- **Action**: a typed UI intent returned by a view.
- **Effect**: a filesystem, Git, process, or workspace-state change after an action is consumed.
- **Source-observed**: visible in Rust at the exact source SHA below; it does not prove the user-facing path was exercised at runtime.

## 1. Source basis

**Source SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (`main`, 2026-09-24).

Primary implementation files:

- `crates/ui/src/files_activity_view.rs` — Files activity entry and sub-tab dispatch.
- `crates/ui/src/files_surface_view.rs` — workspace header, empty state, roots, trust/environment indicators, sub-tabs.
- `crates/ui/src/files_activity_tabs.rs` — action consumption and Files sub-tab orchestration.
- `crates/ui/src/files_tree_view.rs` — workspace file tree and file actions.
- `crates/ui/src/files_search_view.rs` — search/replace/refactor controls and result presentation.
- `crates/ui/src/files_secondary_tabs_view.rs` — migration list and dependency list presentations.
- `crates/ui/src/files_tasks_view.rs` — shell task input and last-run output.
- `crates/ui/src/files_git_view.rs` — Git status, staging, diff, commit, and external-file-change presentation.
- `crates/ui/src/files_agent_context_view.rs` — agent context items and workspace utilities.
- `crates/ui/src/workspace_files_state.rs` — Files activity state and operation mapping.
- `crates/ui/src/ide_workspace.rs`, `ide_workspace_types.rs`, `ide_workspace_scan.rs` — local workspace model, filesystem operations, search, detection, diagnostics, and scan limits.
- `crates/ui/src/git_workspace.rs` — optional command-line Git adapter.

All source line anchors in this document refer to the Source SHA above.

## 2. Activity and action pipeline

Files is rendered as a sidebar activity. The activity draws the workspace shell first; if there is an open root, it dispatches the selected sub-tab and then draws the agent-context strip.

```text
egui interaction
  → local typed action (FilesSurfaceAction / FilesTreeAction / FilesSearchAction / ...)
  → DbProApp Files action mapper
  → WorkspaceFilesState / IdeWorkspaceState / Git adapter / local process
  → feedback message or refreshed local Files state
```

Source: `files_activity_view.rs:8-29,31-66`; `files_activity_tabs.rs:62-237`.

The workspace operations are local filesystem/Git/process operations. The Files activity path does not use the database `UiCommand → worker → provider → UiEvent` pipeline for these actions.

## 3. Workspace shell and roots

`FilesSurfaceContext::draw` renders a WORKSPACE header, an empty state when there are no roots, or a workspace card and six sub-tabs when a root is open (`files_surface_view.rs:28-37,40-68,70-110,244-321`).

| Capability | Source-observed behavior | Source |
|---|---|---|
| Open folder | Opens the folder picker; recent roots can be selected from the empty state. The recent list displays up to 8 paths. | `files_surface_view.rs:70-104`; `files_activity_view.rs:34-39` |
| Multiple roots | Displays a root switcher when more than one root exists; the active root drives the tree. | `files_surface_view.rs:213-230`; `ide_workspace.rs:25-30,39-59` |
| Recent folders | Keeps up to 12 paths in workspace state; this UI shows at most 8. | `ide_workspace.rs:149-154`; `files_surface_view.rs:93-104` |
| Trust | Shows Trusted/Untrusted badge and lets the user toggle trust. The model also has `Restricted`, but the UI toggle is boolean. | `files_surface_view.rs:174-192`; `ide_workspace_types.rs:19-25` |
| Environments | Shows selectable environment labels. `open_root` seeds Development/Staging/Production with `connection_id`, `database`, and `schema` unset if the list is empty. These labels are not evidence of connected DB profiles. | `ide_workspace.rs:60-80`; `ide_workspace_types.rs:43-49` |
| Refresh | Re-scans roots and SQL diagnostics, then reports indexed count or failure. | `workspace_files_state.rs:106-117` |
| Close/remove root | Closes the workspace or removes the active root; closing clears roots, expansion state, diagnostics, and agent context items. | `files_surface_view.rs:134-154`; `workspace_files_state.rs:97-104`; `ide_workspace.rs:84-106` |

The source contains no persisted workspace manifest in `WorkspaceFilesState`; this baseline does not assert that open roots, selected tab, or expanded folders survive application restart (`workspace_files_state.rs:8-26`).

## 4. File tree

The tree supports New SQL, New folder, directory expand/collapse, opening SQL files, creating SQL under a directory, deleting files/folders, adding a file to agent context, and searching references by file stem (`files_tree_view.rs:5-15,25-62,66-83,85-143,146-230`; action mapping: `files_activity_tabs.rs:84-129`).

The scanner indexes only `sql`, `md`, `txt`, `toml`, `json`, `yml`, `yaml`, and `dbpro-nb`; it skips selected generated/dependency directories, hidden entries except `.env.example`, `.dbpro`, and `.sqlfluff`, and stops at depth 8 or 2,000 entries (`ide_workspace_scan.rs:1-19,21-86`; limits: `ide_workspace_types.rs:15-17`).

Observed mutation/error boundaries:

- `create_file`, `create_folder`, `rename_path`, and `delete_path` operate on local filesystem paths (`ide_workspace.rs:157-207`).
- `apply_replace`, `run_task`, and `rename_symbol_across_sql` require Trusted state (`ide_workspace.rs:268-293,365-394,407-415`).
- The Files tree action mapper discards the result from delete and create-under-directory (`files_activity_tabs.rs:103-120`); the context menu invokes Delete without a confirmation step (`files_tree_view.rs:112-143,180-229`).
- New SQL at the workspace root reports create errors to feedback, unlike create-under-directory (`files_activity_tabs.rs:84-121`).

## 5. Search, replace, and refactor

The Search sub-tab exposes a text query, replacement string, Find, Preview, Replace all, and a separate Rename from/to plus Refactor form (`files_search_view.rs:22-105`). Search results show up to 40 file/line entries and open matching SQL paths; replacement previews show up to 30 files and only a per-file hit count (`files_search_view.rs:108-147`).

Source behavior:

- Search is literal, case-insensitive, reads indexed files, and stops at the requested limit; the Files state requests at most 100 hits (`ide_workspace.rs:210-241`; `workspace_files_state.rs:119-127`).
- Replace preview counts literal matches per file; it does not produce a before/after diff (`ide_workspace.rs:243-266`).
- Replace-all is trust-gated, writes each matching file sequentially, then refreshes preview and search state (`ide_workspace.rs:268-293`; `workspace_files_state.rs:129-147`).
- “Refactor” calls the same text-replacement operation; no SQL symbol binding is shown in this path (`workspace_files_state.rs:186-193`; `ide_workspace.rs:407-415`).

## 6. Migrations, tasks, graph, and Git

### Migrate

The Migrate sub-tab detects SQL files whose path contains `migration`, contains `/migrate`, or starts with `migrations/`; it labels each entry with a filename-derived version and status (`ide_workspace.rs:341-362`). The UI only lists and opens files (`files_secondary_tabs_view.rs:12-35`; `files_activity_tabs.rs:153-158`). `MigrationStatus` has `Pending` and `Unknown`, and detection currently assigns `Unknown` (`ide_workspace_types.rs:51-66`; `ide_workspace.rs:353-358`). No database history lookup or migration execution is wired from this tab.

### Tasks

The Tasks sub-tab accepts one free-form command, runs it via `sh -lc` (or `cmd /C` on Windows) in the active workspace root, and stores the last result (`files_tasks_view.rs:16-42`; `ide_workspace.rs:365-394`). Execution is gated by workspace trust. The UI displays exit code, elapsed time, up to 800 stdout characters and 400 stderr characters; it has no task catalog or live output/cancel action (`files_tasks_view.rs:44-74`). The “sample benchmark” is local timing of small CPU work, not a database/provider benchmark (`files_activity_tabs.rs:161-190`; `ide_workspace.rs:603-628`).

### Graph and diagnostics

The Graph sub-tab lists at most 60 `file → object` entries (`files_secondary_tabs_view.rs:38-60`). Edges are extracted from SQL text by scanning for `FROM`, `JOIN`, `UPDATE`, `INTO`, and `TABLE`, not by resolving parsed SQL or database metadata (`ide_workspace.rs:439-455,553-573`). The diagnostics scan recognizes two string patterns, `SELECT *` and `DROP TABLE` without `IF EXISTS`; refresh/open invokes the scan, but the Files sub-tab dispatch has no diagnostics view (`ide_workspace.rs:307-339`; `workspace_files_state.rs:106-113`; `files_activity_view.rs:19-28`).

### Git

The Git sub-tab is backed by the installed `git` executable. It can refresh status, stage/unstage paths, show a diff against HEAD, open a file, reload/dismiss an external-change notice, and commit staged files (`git_workspace.rs:31-60,63-114,116-163`; `files_git_view.rs:25-43,108-145,162-229`; action mapping: `files_activity_tabs.rs:203-237`). Git is optional. Status is capped at 200 entries in the adapter; the view displays at most 80 and diff text at most 8,000 characters (`git_workspace.rs:92-105`; `files_git_view.rs:142-145,213-227`). Branch/remote management, history, hunk staging, push/pull, and conflict resolution are not wired in this view.

## 7. Agent context and database-linked utilities

The strip can attach the active file, selected SQL text, and selected table; it also lists removable context chips and exposes clear, schema-drift check, schema snapshot export, and split-editor actions (`files_agent_context_view.rs:6-14,21-27,29-159`; effect mapping: `files_activity_tabs.rs:13-59`).

The workspace context implementation currently fingerprints supplied table names for drift comparison (`workspace_files_state.rs:195-200`; `ide_workspace.rs:593-601`). Snapshot export writes a generated SQL file whose content is table-name/column-count comments, not executable table DDL (`workspace_files_state.rs:160-173`; `files_activity_tabs.rs:47-50`).

## 8. Current capability inventory

Files is currently a **local SQL-project workspace shell** that combines:

1. multi-root browsing and local SQL document opening;
2. literal cross-file search and replacement;
3. heuristic migration-file listing and SQL-token dependency listing;
4. trust-gated shell tasks and text refactor;
5. optional local Git status/staging/diff/commit;
6. agent context attachment and a small set of live-schema helper actions.

Several types and helpers in `ide_workspace.rs` are explicitly described as incrementally integrated; their presence in the model is not evidence that Files exposes them (`ide_workspace_types.rs:1-9,167-185`).

## 9. Evidence boundary

This is source analysis at `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. It establishes that the listed call paths and guards exist in source. It does not establish that all UI interactions were exercised, that file mutations are atomic, that the task runner is safe for every workspace, or that migration/dependency results match a live database. No runtime capture or live database verification is included in this baseline.
