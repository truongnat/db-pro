# History Activity Baseline

> Status: source analysis; no implementation or runtime verification performed.
>
> Source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (HEAD, 2026-09-24).

## Scope

This document traces the native **History** activity and adjacent query-history surfaces. DB Pro currently has multiple distinct representations of query history; the rail activity, query output dock, Quick Open, local UI storage, and meta-store API do not all consume the same records. This is source analysis, not runtime evidence or an implementation commitment.

Line anchors refer to the exact source SHA above. `Source-observed` means the behavior is visible in source; it does not prove that the UI or database path was exercised.

## 1. Surface and dispatch

- The activity rail labels `Activity::History` as **History** in the tools/management group (`crates/ui/src/activity_bar_view.rs:85-96`). The sidebar routes that activity to `draw_history` (`crates/ui/src/sidebar_view.rs:64-75`), which renders the same saved-query and local-history library sections used by the Queries activity (`crates/ui/src/sidebar_activities_view.rs:140-157`; `crates/ui/src/sidebar_queries_surface_view.rs:26-68,83-108`).
- The History activity is therefore a query-library shortcut, not a general execution-log or audit view. It shows saved queries plus a compact list of local SQL strings; it does not render the structured execution records described below.
- The goal documents place Query History inside the Queries activity, not as a separate top-level History rail entry (`docs/goals/goal-phase-g-productivity.md:190-202`; `docs/goals/goal-full-product.md:221-258`). The present activity also overlaps the Saved Queries and History sections already visible in the Queries activity.

## 2. Distinct history stores and views

### A. Sidebar local-history strings

- `QueryEditorState::query_history` is `Vec<String>`. `commit_dispatched` records the prepared SQL before the runtime result arrives; exact-string duplicates are ignored, and the list is capped at 20 by removing index 0 (`crates/ui/src/query_editor_state.rs:31-35`; `crates/ui/src/query_execution_actions.rs:128-150`). Re-running an existing SQL string does not move it to the newest position.
- The sidebar renders the last 15 entries in reverse vector order. Each row displays only the first line, with full SQL in a hover tooltip (`crates/ui/src/sidebar_query_library_view.rs:48-67`). No timestamp, connection, schema, status, or outcome is available on this path.
- Clicking a local-history row emits `OpenHistory(sql)`; the action consumer calls `set_active_query_text(sql)` and activates the Query workspace (`crates/ui/src/sidebar_activities_view.rs:160-166`). This replaces text in the active query document instead of opening a new document, keeps its current connection/schema, marks it dirty, and does not request dirty-buffer confirmation (`query_documents.rs:78-83,239-246`; `query_state.rs:41-46`). The underlying `TextBuffer::set_text` replacement is undo-recorded (`crates/ui/src/editor/buffer.rs:454-517`), so this is an unprompted destructive-to-current-buffer action, but not proven irreversible loss.
- Lifecycle persistence writes `query_history_entries`, not this `query_history` string vector (`crates/ui/src/app_lifecycle.rs:53-65`). The sidebar list is therefore not persisted by this storage hook.

### B. Structured native execution history

- `UiQueryHistoryEntry` stores SQL, optional connection/schema, execution start time, duration, success/failed/cancelled status, row and affected-row counts, and error fields (`crates/ui/src/runtime_query_types.rs:55-75`). Completion, failure, cancellation and multi-result reducers create these records; the shared reducer trims the front above 500 entries (`crates/ui/src/query_result_events.rs:63-78`; `query_failure_events.rs:74-89`; `query_execution_events.rs:68-83`; `query_multi_result_events.rs:95-130`; `query_history_events.rs:6-29`).
- The native output dock has a separate **Recent Executions** history pane. It filters by SQL text, displays the latest 50 matching entries, and offers Replay. Replay creates a new query document with the recorded connection/schema context and immediately dispatches it for execution (`crates/ui/src/query_output_actions_view.rs:117-154,173-230`; `crates/ui/src/query_documents.rs:36-43,348-355`). This is distinct from clicking a string in the History sidebar.
- The output pane maps only `Failed` to FAIL; both `Success` and `Cancelled` display as OK (`crates/ui/src/query_output_actions_view.rs:181-191`), although the data model records `Cancelled` explicitly.
- The output pane’s preview truncation slices at byte offset 117 after checking byte length (`query_output_actions_view.rs:224-227`). A multibyte UTF-8 character crossing that byte boundary can make the string slice panic; this is a source-derived edge-case risk, not a reproduced runtime failure.
- Eframe storage serializes/restores the structured collection under `dbpro.native.query-history-v1` (`crates/ui/src/app_lifecycle.rs:53-59`; `crates/ui/src/app_storage.rs:142-154`). Source inspection does not establish the host storage’s crash/flush durability.

### C. Quick Open history results

- Quick Open builds History results from `query_history_entries.iter().take(30)` and labels each result with status/connection; opening uses the selected vector index and creates a new document without replay (`crates/ui/src/palette_search_view.rs:266-299`; `crates/ui/src/palette_actions.rs:62-69`; `crates/ui/src/query_documents.rs:348-355`).
- The structured collection is appended chronologically and trims old entries from the front (`query_history_events.rs:8-29`). Since Quick Open takes the first 30 without reversing, it surfaces the oldest retained records, not the newest records. This competes directly with the “recent query” expectation.

### D. Meta-store query history

- `QueryService::execute` saves a record after a successful result; a failed save is logged but does not fail query execution (`crates/core/src/application/query_service.rs:143-180`). `execute_multi` returns an error-bearing result before its history save; only the success path reaches repository save (`query_service.rs:225-260`). The native runtime worker dispatches UI query commands through `QueryApi::execute_with_params` / `execute_multi` (`crates/runtime/src/worker.rs:1518-1617`; `crates/runtime/src/api.rs:304-337`); for the single-query UI command it passes `None` for database and schema (`worker.rs:1545-1551`), despite the service API accepting both context values.
- The SQLite meta-store repository stores SQL, connection id, execution timestamp, duration, row count, database and schema; its list method is per connection, newest-first, with caller-provided limit (`crates/infrastructure/src/meta/query_history_repo.rs:11-65`; `crates/infrastructure/src/meta/schema.rs:15-27`). The repository contract has `save` and `list`, not clear/delete or retention methods (`crates/core/src/ports/query_history_repository.rs:10-21`).
- `QueryApi::history` exposes the meta-store read path (`crates/runtime/src/api.rs:352-358`), but repository search found no UI call site for that method. The native UI instead keeps and renders its eframe-backed `UiQueryHistoryEntry` collection. Thus source shows a meta-store history path and a separate native UI history path; source does not show them reconciled into one displayed log.

## 3. Action/data-flow summary

```text
Query dispatch
  ├─ before result: append unique SQL string to query_history (cap 20)
  │    └─ History sidebar shows last 15; click replaces active document text
  ├─ runtime QueryService successful execution: save QueryHistory to SQLite meta.db
  │    └─ QueryApi.history exposes per-connection reads; no UI caller found
  └─ UI completion/failure/cancellation event: append UiQueryHistoryEntry (cap 500)
       ├─ eframe storage key dbpro.native.query-history-v1
       ├─ Query output dock: SQL filter, latest 50 matches, Replay executes
       └─ Quick Open: first 30 entries, open-only; source order is oldest-first
```

## 4. Source-observed risks and limits

- **P2 — History activity does not expose structured execution records.** It renders an ephemeral distinct-SQL list while structured UI records and meta-store rows are maintained elsewhere. Users can see different histories depending on which surface they open.
- **P2 — Sidebar history click replaces the active SQL without a prompt or context restore.** It writes a string into the current document and retains that document’s connection/schema; the replacement is undo-recorded but not opened in a separate, source-context document (`sidebar_activities_view.rs:160-166`; `query_documents.rs:78-83,239-246`; `editor/buffer.rs:454-517`).
- **P2 — Quick Open history omits the newest records.** The UI caps results at the first 30 of a chronological collection rather than the latest 30 (`palette_search_view.rs:266-299`; `query_history_events.rs:8-29`).
- **P2 — Cancelled executions are presented as successful in the output history pane** (`query_execution_events.rs:68-83`; `query_output_actions_view.rs:181-191`).
- **P1 edge-case risk — Unicode preview truncation can panic** when a multibyte character straddles byte 117 (`query_output_actions_view.rs:224-227`). This finding is source-derived; no runtime reproduction was performed.
- **P2 — Retention and clear semantics diverge.** Structured UI records cap at 500, sidebar strings at 20, while the meta-store insert path has no pruning/clear method. The UI surface does not expose clear/export or retention settings (`query_history_events.rs:6-29`; `query_execution_actions.rs:143-150`; `query_history_repository.rs:10-21`; `query_history_repo.rs:11-47`).
- **P2 — History placement and library contents are ambiguous.** A dedicated History rail activity repeats saved queries and local history while the product goals place Query History under Queries (`sidebar_queries_surface_view.rs:34-68,83-108`; product goal anchors above).

## 5. Evidence limits

The report is source-only at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. No build/test command, visual UI traversal, eframe persistence/restart check, query replay, PostgreSQL run, or SQLite run was performed. Source-visible tests are not runtime evidence.
