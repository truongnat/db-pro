# Query Activity Baseline

> Status: source analysis; no implementation or runtime verification performed.
>
> Source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`  
> Research date: 2026-09-24

## Scope

This document analyzes the **Queries activity/sidebar**: open query documents, saved queries, the sidebar's local recent-query list, and its SQL shortcuts. It does not re-specify the central SQL editor, execution controls, result grid, or output/history dock. Those surfaces are covered by [`../query-editor-data-panel/`](../query-editor-data-panel/).

The Query activity is a workspace navigator and launcher, not a standalone query execution implementation. Its actions route to the existing query-document state, query library commands, and the central `WorkspaceTab::Query` surface.

## Evidence boundary

The findings below are source-derived from the exact source baseline above. They do not establish runtime behavior, provider behavior, database persistence, or visual acceptance. Line references are source anchors at that baseline.

## Surface and dispatch

- The activity bar labels `Activity::Queries` as “Queries”; the sidebar dispatcher routes that activity to `draw_queries` (`crates/ui/src/activity_bar_view.rs:66-68`; `crates/ui/src/sidebar_view.rs:65-68`).
- `DbProApp::draw_queries` builds a `SidebarQueriesSurfaceContext` from query documents, active document/tab, saved-query summaries/folders, `query.editor.query_history`, and the pending delete-confirmation id. It collects typed surface actions, then applies them (`crates/ui/src/sidebar_activities_view.rs:8-29`).
- The surface combines open query documents, saved-query library, local history, and shortcut controls (`crates/ui/src/sidebar_queries_surface_view.rs:26-32`).

### Action pipeline

```text
Queries sidebar view
  → SidebarQueriesSurfaceAction
      → OpenQuery(SidebarQueriesAction)
      → Library(SidebarQueryLibraryAction)
      → Shortcut(SidebarQueryShortcutAction)
  → DbProApp::draw_queries
  → apply_sidebar_query_action / apply_sidebar_query_library_action
    / apply_sidebar_query_shortcut_action
  → query-document state, workspace state, clipboard, overlay,
    or runtime command dispatch
```

`SidebarQueriesSurfaceContext::draw` draws open query documents, saved-query/history sections, then shortcuts (`sidebar_queries_surface_view.rs:26-32,83-118`). The separate `draw_history` surface reuses the saved-query and local-history renderers for the History activity (`sidebar_queries_surface_view.rs:34-68`); the Queries activity and History activity therefore share rendering/actions for this library path.

| Action group | UI intent | Effect in the app |
|---|---|---|
| Open query documents | `NewQuery`, `NewScratch`, `Select(index)`, `Duplicate(index)`, `Rename(index)`, `Close(index)` (`sidebar_queries_view.rs:6-14`) | Create/activate a query document; switch to its index and Query workspace; duplicate/rename; or request close (`sidebar_activities_view.rs:32-44`). |
| Saved query/history | `NewQuery`, `OpenQuery(sql)`, `OpenHistory(sql)`, `CopySql`, `Rename`, delete request/confirm/cancel, folder-delete request (`sidebar_query_library_view.rs:5-16`) | Create a document; replace active document text and activate Query; copy SQL; dispatch rename/delete commands; or update confirmation overlay (`sidebar_activities_view.rs:160-202`). |
| Shortcuts | `InsertSnippet(sql)`, `NewScratch` (`sidebar_query_shortcuts_view.rs:5-9`) | Insert SQL into the active document and activate Query/Queries, or create and activate a scratch document (`sidebar_activities_view.rs:46-59`; `query_documents.rs:23-34`). |

The key behavioral distinction: open-document selection switches document identity, while saved query/history actions currently assign SQL text to the active document. See [Source-observed interaction risks](#source-observed-interaction-risks).

### Main Query editor boundary

The Queries activity creates or activates `WorkspaceTab::Query`, but it does not own SQL execution, result display, or transaction UI. The current Query editor context menu has save/save-as, visual builder, find, transaction, font, prediction, snippets, and folder actions (`query_actions_surface_view.rs`). Keep those central editor concerns in the existing [`query-editor-data-panel`](../query-editor-data-panel/) documentation instead of duplicating them here.

### Source files

- Activity dispatch and action reduction: `activity_bar_view.rs`, `sidebar_view.rs`, `sidebar_activities_view.rs`.
- Queries activity composition: `sidebar_queries_surface_view.rs`.
- Open query documents: `sidebar_queries_view.rs`, `query_documents.rs`, `query_state.rs`.
- Saved-query library and local-history rows: `sidebar_query_library_view.rs`, `query_library_state.rs`, `query_save_commands.rs`, `query_save_actions.rs`.
- SQL snippets and scratch affordance: `sidebar_query_shortcuts_view.rs`, `query_snippets.rs`.
- Execution and structured-history state: `query_execution_actions.rs`, `query_history_events.rs`.

## Shell actions and sections

The activity bar routes `Activity::Queries` to the Queries sidebar and provides the Queries label (`activity_bar_view.rs:66-68`; `sidebar_view.rs:65-68`). `draw_queries` supplies the query documents, active selection, saved library, local history strings, and delete-confirmation state, then applies the typed actions returned by the surface (`sidebar_activities_view.rs:8-29`).

Sections are ordered as **Open Queries → Saved Queries → History → Snippets → Scratch** in the main Queries activity (`sidebar_queries_surface_view.rs:26-32,83-118`). The open list shows the active document only when `WorkspaceTab::Query` is active and marks dirty documents with a bullet (`sidebar_queries_view.rs:55-68`).

The Scratch section describes scratch tabs as “disposable” and calls for throwaway SQL (`sidebar_query_shortcuts_view.rs:19-28`). That is user-facing copy; this baseline does not establish restart persistence behavior.

## Action and effect reference

| UI action | Typed intent | Effect |
|---|---|---|
| New query / New scratch | `SidebarQueriesAction::NewQuery` / `NewScratch` | Create a document; new-document helper copies active connection/schema, scratch emits feedback, and query surface is activated (`query_documents.rs:23-34`). |
| Select an open query row | `Select(index)` | Switch query document and set `WorkspaceTab::Query` (`sidebar_activities_view.rs:37-39`). |
| Duplicate / Rename tab / Close query | `Duplicate(index)` / `Rename(index)` / `Close(index)` | Call duplicate, generated title rename, or close-request helper (`sidebar_activities_view.rs:40-42`). Close is requested through lifecycle handling rather than direct list mutation. |
| Click saved-query row | `OpenQuery(sql)` | Replace active query text and activate `WorkspaceTab::Query` (`sidebar_query_library_view.rs:150-157`; `sidebar_activities_view.rs:163-166`). |
| Click local-history row | `OpenHistory(sql)` | Same active-buffer text replacement path as saved-query open (`sidebar_query_library_view.rs:54-63`; `sidebar_activities_view.rs:163-166`). |
| Copy SQL | `CopySql { name, sql }` | Write SQL to clipboard and show feedback (`sidebar_activities_view.rs:167-170`). |
| Rename saved query | `Rename(query)` | Synthesize a title from folder draft or `\" (renamed)\"`, dispatch rename command (`sidebar_activities_view.rs:190-202`). |
| Delete saved query | Request → confirm/cancel | Request stores a confirmation id; confirm dispatches delete command, cancel clears the overlay (`sidebar_activities_view.rs:175-183`). |
| Delete folder | Request delete folder | Stores pending folder-delete confirmation (`sidebar_activities_view.rs:184-186`). |
| Insert built-in snippet | `InsertSnippet(sql)` | Inserts into active query document and activates Query workspace/activity (`sidebar_activities_view.rs:51-55`). |

The source-level “Open in Editor” context-menu item is intentionally listed separately under [Source-observed interaction risks](#source-observed-interaction-risks): it closes the menu without producing an action (`sidebar_query_library_view.rs:167-180`).

## Library state and runtime commands

The sidebar groups saved query summaries by folder and resolves folder identity for a folder context menu (`sidebar_query_library_view.rs:31-45,108-148`). Query save commands list queries/folders by connection id and provide create-folder/save/rename/delete operations (`query_save_commands.rs`). Connection events load those lists, and query/folder operation events refresh them for the active connection (`app.rs:120-149`; `operation_events.rs:259-367`). This identifies the dataflow boundary; it does not assert a successful PostgreSQL or SQLite runtime call.

The sidebar history is not the structured history collection. It displays the most recent 15 rows from the string list and truncates labels to the first line; a tooltip exposes the SQL (`sidebar_query_library_view.rs:48-67`). The query-execution path deduplicates exact SQL strings and trims that list to 20 (`query_execution_actions.rs:143-150`). Separately, the structured record reducer stores connection/schema and execution outcome metadata up to 500 entries (`query_history_events.rs:6-29`).

## Query editor cross-reference

The Query editor and output/history docks are intentionally out of this activity baseline's detailed scope. Refer to [`query-editor-data-panel`](../query-editor-data-panel/) for those surfaces; this document records only how the Queries sidebar enters them and which state/action boundaries it uses.

## Current feature inventory

| Area | Source-observed behavior | Boundary / limitation |
|---|---|---|
| Open query documents | Lists current query documents; selects/switches, duplicates, renames, and closes them. Dirty documents are marked in the list. New Query and New Scratch actions create documents. (`sidebar_queries_view.rs:5-68`; `sidebar_activities_view.rs:32-44`) | This is a live-document list, not a project/file tree. “Rename tab” changes the generated tab title; it is not a user-entered filesystem rename. |
| Saved Queries | Groups saved query summaries by folder, displaying an “Unfiled” group where no folder is assigned. Folder groups are collapsible; query context actions include Open in Editor, Copy SQL, Rename query, and Delete query. Delete has confirm/cancel UI. (`sidebar_query_library_view.rs:26-45,69-85,108-148,150-223`) | The sidebar does not expose search/filter or a move-between-folders action. Folder grouping is sourced from summaries; query persistence is performed through runtime commands, not local-only view state. |
| Saved-query loading | Connection events request saved-query and folder lists; query/folder operation events refresh the library for the active connection (`app.rs:120-149`; `operation_events.rs:259-367`). Save and rename/delete are runtime command paths (`query_save_commands.rs`; `query_save_actions.rs`). | Source evidence does not establish database-provider runtime success. Query library records are distinct from query documents and from filesystem SQL files. |
| Sidebar history | Displays up to the last 15 distinct SQL strings in reverse insertion order, using the first line as the row label and full SQL as hover text. Click emits `OpenHistory(sql)`. (`sidebar_query_library_view.rs:48-67`) | This surface receives `query.editor.query_history: Vec<String>`. Execution adds a unique SQL string and caps this list at 20 entries (`query_execution_actions.rs:143-150`). It has no per-row timestamp, status, duration, row count, connection, schema, search, or filter. |
| Structured execution history | A separate editor-owned `query_history_entries` collection stores SQL, connection/schema, start time, duration, status, row/affected-row counts, and error details; the reducer retention cap is 500 (`query_history_events.rs:6-29`). | The Queries sidebar does not receive this structured collection; the sidebar's compact list and the editor/output History surface are different state paths. See the existing query-editor/data-panel design docs for the latter. |
| SQL shortcuts | Offers six built-in SQL snippets and an Open Scratch SQL action. Snippet selection inserts SQL into the active query editor and activates the Query workspace (`sidebar_query_shortcuts_view.rs`; `query_snippets.rs`). | Snippets are a fixed built-in set; no user-authored snippet catalog or search is represented in this sidebar path. |
| Scratch documents | New Scratch creates a query document using the active connection/schema context and gives scratch-specific feedback (`query_documents.rs:23-34`). | Source inspection does not establish durable scratch-file persistence or project-level file lifecycle. |

## Source-observed interaction risks

### Opening a saved query or history entry replaces the active document text

The saved-query row emits `OpenQuery(query.sql)` on row click (`sidebar_query_library_view.rs:150-157`). The context-menu “Open in Editor” branch closes its menu but emits no action (`sidebar_query_library_view.rs:167-180`). The reducer handles both `OpenQuery(sql)` and `OpenHistory(sql)` by calling `set_active_query_text(sql)` and activating the Query workspace (`sidebar_activities_view.rs:160-166`). `set_active_query_text` delegates to `QuerySessionState::set_active_text`, which writes directly to the active document with `document.set_text`; it does not create a separate document (`query_documents.rs:78-83`; `query_state.rs:41-46`).

Consequently, these actions replace the active buffer rather than opening a new query document. The source path does not perform a dirty-document confirmation first. If the active document contains unsaved SQL, this is a source-evidenced data-loss risk; runtime reproduction was not performed. The replacement also keeps the existing document's context unless another path changes it, so the saved/history SQL can remain bound to a different connection or schema from its origin.

A separate `QueryDocumentContext::open_history_entry` helper accepts a structured history entry and creates a context-preserving document (`query_documents.rs:36-43`), but this sidebar's `OpenHistory(String)` path does not call it.

### Saved-query rename does not collect a new name

The context menu emits `Rename(UiSavedQuerySummary)` (`sidebar_query_library_view.rs:197-208`). The reducer generates the new name from the current free-form query-folder draft, or appends `" (renamed)"` to the old name when that draft is empty; it then dispatches the rename command (`sidebar_activities_view.rs:190-202`). The sidebar action itself does not ask for or receive a replacement query title.

### “Open in Editor” context-menu item has no action

The menu item closes the menu but does not append `OpenQuery`, `OpenHistory`, or another action (`sidebar_query_library_view.rs:167-180`). This is a source-level missing-action path, not a runtime-tested report.

## State and responsibility boundaries

- Query documents and active-document selection belong to query-session state (`query_state.rs`; `query_documents.rs`).
- Saved query summaries and folder summaries are loaded/refreshed through query-save commands and runtime operation events (`query_save_commands.rs`; `app.rs:120-149`; `operation_events.rs:259-367`).
- Sidebar history consumes the editor's short `Vec<String>`; structured execution history lives in `query_history_entries` (`sidebar_activities_view.rs:11-20`; `query_history_events.rs:8-29`).
- The sidebar does not implement SQL parsing, execution, result rendering, or transaction controls. Keep those editor/output concerns in the existing [`query-editor-data-panel`](../query-editor-data-panel/) design set.

## Assessment

The Queries activity already provides useful entry points for live query documents, saved-query folders, a compact run-history list, and built-in snippets. Its principal correctness gap is the open path: saved-query and history selection overwrite the active document instead of preserving it as an independent query, with no source-visible dirty guard or origin-context transfer. The sidebar also exposes actions whose current implementation is incomplete: saved-query rename has no user-supplied title, and “Open in Editor” emits no action.

The next design should treat opening/replaying SQL as a document-lifecycle operation with explicit source context, not as a string assignment. History and saved-query library enhancements should reuse their existing state/command boundaries and avoid duplicating the central editor's execution/output design.
