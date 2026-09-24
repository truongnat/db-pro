# Query Editor và Data Result Panel — Feature Baseline

**Source baseline:** `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`  
**Scope:** native egui Query Editor và vùng hiển thị query result/data grid.

> Lưu ý về cách gọi: trong source hiện tại, vùng result được triển khai dưới dạng **output dock phía dưới Query Editor**, không phải right-side dock cố định. Tuy nhiên, phần `Record` inspector/value inspector là panel inspect dữ liệu nằm trong result surface. Report này gom cả hai dưới tên “data panel”.

## 1. Kiến trúc tổng thể

```text
Query workspace
├── Context chrome
│   ├── File path breadcrumb
│   ├── Connection/schema picker
│   ├── Run / Stop
│   ├── Explain
│   ├── Format
│   └── More query actions
├── Transaction controls (optional)
├── Visual query builder (optional)
├── SQL editor
├── Optional panels
│   ├── Find in SQL overlay
│   ├── SQL snippets
│   └── Bind parameters
├── Output dock (optional)
│   ├── Results
│   ├── Chart
│   ├── Messages
│   ├── Explain
│   └── History
└── Query status bar
```

Entry points:

- `DbProApp::draw_query` → query workspace frame.
- `DbProApp::draw_query_content` → orchestrates diagnostics, chrome, editor, output, status bar, and dialogs.
- `DbProApp::draw_query_editor` → renders SQL editor and applies editor effects.
- `DbProApp::draw_query_output_dock` → renders output dock chrome and active output pane.
- `DbProApp::draw_output_pane` → selects Results/Chart/Messages/Explain/History.
- `DbProApp::draw_result_grid` → renders result toolbar, record inspector, header, virtualized rows, selection and editing.

## 2. Query Editor — current features

### 2.1 Context chrome

File: `query_context_view.rs`

Function: `draw_context_strip`

Features:

- Shows file path breadcrumb for file-backed query documents.
- Shows active connection and schema chip.
- Shows `No connection` state when disconnected.
- Shows production environment warning styling.
- Opens connection/schema picker when chip is clicked.
- Provides Run/Stop control.
- Provides Explain action.
- Provides Format SQL action.
- Provides More query actions menu.

`QueryChromeAction` variants:

| Action | Meaning | Root effect |
|---|---|---|
| `RunControl(...)` | Run or stop the active query. | `apply_run_control_action`. |
| `Explain` | Request query plan. | `explain_query`. |
| `Format` | Format active SQL document. | `format_active_query`; cancels prediction request if needed. |

### 2.2 Connection/schema context picker

File: `query_context_picker_view.rs`

Function: `draw_picker`

Features:

- Lists saved connections.
- Shows connection environment beside connection name.
- Shows available schemas.
- Highlights current connection/schema.
- Closes when clicking outside.

`QueryContextPickerAction`:

| Action | Effect |
|---|---|
| `SelectConnection(id)` | Binds active query document to connection id. |
| `SelectSchema(schema)` | Binds active query document to schema. |
| `Close` | Closes picker. |

The query context is document-scoped, so changing connection/schema changes the active document context rather than only changing a global label.

### 2.3 Run/Stop/Cancel

File: `query_run_control_view.rs`

Function: `draw_run_control`

States:

- No active request + connected → `Run`.
- No active request + disconnected → run control reports disconnected.
- Active request + provider supports cancel → `Stop`.
- Active request + provider does not support cancel → `Running…` with capability reason.

`QueryRunControlAction`:

| Action | Effect |
|---|---|
| `Run` | Calls `dispatch_query`. |
| `Cancel(request_id)` | Calls `cancel_query`. |
| `ReportUnsupportedCancel(reason)` | Shows provider limitation in feedback. |
| `ReportDisconnected` | Shows connect-before-run feedback. |

Shortcut shown by UI: primary modifier + Enter to run; Escape is shown for Stop tooltip.

### 2.4 SQL text editing

File: `query_editor_surface_view.rs`, `editor/renderer.rs`, `editor/buffer.rs`, `editor/document.rs`

`QueryEditorSurfaceContext::draw_query_editor` performs:

1. Render active document through `SqlEditor`.
2. Apply editor response to document state.
3. Schedule/cancel AI prediction when eligible.
4. Resolve completion requests.
5. Append terminal actions such as run statement/all and save.
6. Draw signature help.
7. Draw hover popup.

Editor capabilities visible in the source:

- Text buffer editing.
- Cursor and selection state.
- Undo/redo buffer state.
- Syntax highlighting using selected SQL dialect.
- Cached SQL tokens.
- Search match decorations.
- Diagnostic decorations.
- Executing-range decoration while query runs.
- Completion popup.
- AI prediction/ghost text.
- Signature help.
- Rich hover popup.
- Manual run statement/all actions.
- Save query action.
- Configurable editor font size.
- Auto-focus when query editor opens.

`QueryEditorAction`:

| Action | Meaning | Effect |
|---|---|---|
| `CancelPrediction { request_id }` | Cancel pending AI SQL prediction. | Dispatches `UiCommand::CancelSqlPrediction`. |
| `RequestPrediction(request)` | Request contextual SQL prediction. | Dispatches `UiCommand::RequestSqlPrediction`; commits request if dispatch succeeds. |
| `DispatchStatement` | Run current statement. | Calls `dispatch_query`. |
| `DispatchAll` | Run all statements. | Calls `dispatch_query_all`. |
| `SaveQuery` | Save current query document. | Calls `save_query_document`. |

### 2.5 SQL completion

Files: `editor/completion.rs`, `query_completion_popup_view.rs`, `query_editor_surface_view.rs`

Completion item kinds:

- Keyword.
- Table.
- View.
- Column.
- Function.
- Schema.
- CTE.
- Snippet.

Completion behavior:

- Manual and automatic triggers are distinguished.
- Completion is refreshed after editing while popup is open.
- Completion closes when cursor context changes.
- Items include label, insert text, detail, documentation, replacement range, and ranking score.
- Items are applied only when document version still matches the version used to compute them.
- Popup supports selected-item navigation, page movement, and replacement of declared prefix.
- Completion uses schema symbol index and provider-aware SQL dialect.

`EditorInteractionPolicy::completion_intent` returns:

```text
None
Open(Manual | Automatic)
Refresh
Close
```

### 2.6 Signature help and hover

Functions:

- `draw_signature_help` obtains function signature from `schema_symbol_index.signature_help`.
- `draw_hover_popup` tracks hovered token, debounces repaint, resolves rich metadata through `schema_symbol_index.rich_hover`, and draws a rich popup.

Behavior:

- Signature help is hidden while completion popup is open.
- Hover popup is hidden while completion/signature help has priority.
- Hover uses confirmed token state rather than immediately showing on every pointer move.
- Schema context and dialect are passed to symbol lookup.

### 2.7 SQL formatting

Function: `query_diagnostics_view::format_active_query`

Behavior:

- Invalidates prediction for the active document.
- Chooses PostgreSQL-like or SQLite dialect based on capabilities.
- Formats the active document.
- Returns prediction request id for cancellation when applicable.

Entry points:

- `Format` button in context chrome.
- `Format SQL` in query actions menu.

### 2.8 Diagnostics and lint

Files: `query_diagnostics_view.rs`, `editor/diagnostics.rs`

Diagnostics sources:

- Parser.
- Delimiter.
- Database/execution.
- Lint.

Current checks include:

- SQL parser errors.
- Unmatched/mismatched delimiters.
- Unclosed string literals.
- `SELECT *` lint.
- Comparison with `NULL` using `=`/`!=`/`<>`.
- `DELETE` without `WHERE`.
- `UPDATE` without `WHERE`.
- Positional `ORDER BY` ordinal.
- Comma join/cartesian-product risk.
- Duplicate projection aliases.
- Provider mismatch for `ILIKE`.
- Provider mismatch for `GLOB`.
- Invalid subquery shape handled by current parser-specific checks.

Diagnostics are debounced while typing. Cached diagnostics are reused by document index, buffer version, driver, and execution-diagnostic identity.

Some diagnostics carry a deterministic replacement fix, for example `NULL` comparison correction.

Status-bar action:

- Clicking diagnostics count opens output dock and selects Messages.

### 2.9 Search in SQL

File: `query_search_view.rs`

Function: `draw_editor_search_overlay`

Features:

- Floating find overlay positioned inside editor bounds.
- Search query and match count.
- Previous/next match.
- Current match selection and cursor movement.
- `Enter` next match.
- `Shift+Enter` previous match.
- `Escape` closes and clears search.
- Search decorations are passed into the editor renderer.

### 2.10 AI SQL prediction

Files: `editor/prediction.rs`, `query_editor_surface_view.rs`, `query_actions_surface_view.rs`

Modes:

- `Off`.
- `Subtle`.
- `Eager`.

Actions:

- Generate SQL Prediction.
- Set prediction mode.
- Cancel prediction request.
- Apply/replace prediction through the editor flow.

Safety/data disclosure note shown in UI:

```text
Sends the SQL around your cursor and its schema context to your configured AI provider.
```

The source documents that no configured provider results in a local “AI provider is not configured” response.

### 2.11 Query actions menu

File: `query_actions_surface_view.rs`

`QueryActionsSurfaceAction` variants:

| Group | Action | Behavior |
|---|---|---|
| Run | `Run` | Run query or selected SQL. |
| Run | `Format` | Format SQL. |
| Run | `Explain` | Request query plan. |
| Run | `AskAgent` | Open Agent prompt about current/selected SQL. |
| Run | `Save` | Save query. |
| Run | `SaveAs` | Open Save As dialog. |
| Run | `ToggleVisualBuilder` | Open/close visual query builder. |
| Editor | `ToggleSearch` | Open/close Find in SQL. |
| Editor | `ToggleTransaction` | Open/close transaction controls. |
| Editor | `DecreaseFont` | Decrease font, minimum 10 px. |
| Editor | `IncreaseFont` | Increase font, maximum 24 px. |
| AI | `GeneratePrediction` | Request prediction when mode is not Off. |
| AI | `SetPredictionMode(mode)` | Set Off/Subtle/Eager. |
| Library | `ToggleSnippets` | Open/close SQL snippets. |
| Library | `CreateFolder` | Create query library folder for active connection. |
| Menu | `Close` | Close query actions menu. |

### 2.12 Visual query builder

File: `query_shell_surface_view.rs`, `visual_query_builder_surface_view.rs`, `visual_query_builder_view.rs`

- Optional collapsible `Visual query builder` surface.
- Toggled from Query actions menu.
- Drawn above the editor.
- Current report should treat it as a separate sub-feature and inspect its own action/state model before compare implementation.

### 2.13 SQL snippets

Files: `query_snippets_surface_view.rs`, `query_snippets.rs`

- Optional SQL snippets panel.
- Uses built-in SQL snippets.
- `QuerySnippetsAction::Insert(&'static str)` inserts snippet at cursor.
- Cancels pending prediction before insertion.
- Updates cursor/selection and marks document dirty.
- Re-runs diagnostics after insertion.

### 2.14 Bind parameters

File: `query_parameters_view.rs`

Features:

- Discovers numbered, named, and positional SQL parameters.
- Shows parameter count in status bar.
- Editable in-memory parameter values per document.
- Secret checkbox masks values.
- Secret values are not persisted with drafts.
- Provider capability warning when bindings are not advertised.

### 2.15 Transaction controls

File: `query_transaction_surface_view.rs`

Features:

- Optional transaction bar.
- Commit/rollback/transaction actions through shared `TransactionBar`.
- Auto-commit/manual state.
- Pending transaction count.
- Disconnect guard when open transaction blocks disconnect.
- Dismiss guard action.

`QueryTransactionAction`:

- `Transaction(TransactionAction)`.
- `DismissDisconnectGuard`.

Transaction SQL reuses normal query execution path so execution state, cancellation, and history remain consistent.

### 2.16 Query document lifecycle

Files: `query_documents.rs`, `query_session.rs`

Current document/session concerns:

- Multiple query documents.
- Active document index.
- New query document.
- Close query document.
- Duplicate query document.
- Dirty state and dirty-close confirmation.
- File path/document breadcrumb.
- Per-document connection/schema binding.
- Per-document output tab state.
- Per-document prediction state.
- Per-document parameter values.
- Open query from history.
- Save and Save As flows.

## 3. Data/result panel — current features

### 3.1 Output dock geometry

Files: `query_layout_surface_view.rs`, `query_output_dock_surface_view.rs`

Functions:

- `calculate` computes editor height and output dock height.
- `draw_chrome` renders resize grip and output tabs.
- `draw_resize_grip` supports vertical resizing.
- Output can be maximized or restored.
- Output can be closed.
- Editor keeps a minimum height when output is open.
- Maximized output leaves a compact editor strip.

Output dock controls:

- Resize vertically.
- Maximize output.
- Restore output.
- Close output.

### 3.2 Output tabs

File: `query_output_tabs_view.rs`

Tabs:

1. Results.
2. Chart.
3. Messages.
4. Explain.
5. History.

Per-document selected output tab is persisted in query output state.

Results tab shows result row-count badge. Messages tab shows message-count badge when non-empty.

### 3.3 Results surface

File: `query_results_surface_view.rs`

`QueryResultsSurfaceAction`:

- `SelectResult(index)` switches active result for multi-result execution.
- `OpenExport` opens export dialog.

Results surface displays:

- Multi-result selector (`Result 1`, `Result 2`, etc.).
- Row count.
- Execution duration in milliseconds.
- Export button.
- Empty state `No results yet` before execution.
- Statement-completed state when query returns no rows.
- Result grid for row-returning statements.

### 3.4 Result grid layout and performance

Files: `result_grid_view.rs`, `result_grid_body_view.rs`, `result_grid_projection.rs`

Features:

- Horizontal scrolling for wide result sets.
- Vertical virtualized row rendering using `show_rows`.
- Row number gutter.
- Stable original row identity after filter/sort projection.
- Projection cache keyed by result epoch/filter/sort/layout state.
- Selection lookup maps visible/sorted rows back to original result rows.
- Separate table-data mode and query-result mode.
- Query results are read-only unless the active workspace is a mutable table data view.

### 3.5 Result-grid toolbar

File: `result_grid_toolbar_view.rs`

Controls:

- Filter visible rows.
- Clear row filter.
- Matching row count badge.
- Copy selected cell.
- Copy selected row as TSV.
- Copy visible rows as CSV.
- Copy visible rows as JSON.
- Copy visible rows as Markdown.
- Copy visible rows as SQL INSERT statements.
- Toggle Record inspector.
- Inspect selected cell.
- Copy status feedback badge.

`ResultGridToolbarAction`:

| Action | Effect |
|---|---|
| `CopySelectedCell` | Copy selected cell value. |
| `CopySelectedRow` | Copy selected row as tab-separated text. |
| `CopyVisibleCsv` | Copy filtered/visible rows as CSV. |
| `CopyVisibleJson` | Copy filtered/visible rows as JSON array. |
| `CopyVisibleMarkdown` | Copy filtered/visible rows as Markdown table. |
| `CopyVisibleInsert` | Copy filtered/visible rows as SQL INSERT. |
| `InspectSelectedCell` | Open advanced value inspector. |

### 3.6 Column headers

Files: `result_grid_header_surface_view.rs`, `result_grid_header_menu_view.rs`, `result_grid_header.rs`

Visible behavior:

- Column name.
- Data type.
- Primary-key indicator.
- Foreign-key indicator.
- Sort marker and sort priority.
- Horizontal column resize divider.
- Click-to-sort behavior.
- Shift-click table sort behavior in table-data mode.
- Header context menu.

`GridHeaderAction`:

| Action | Meaning |
|---|---|
| `Sort { column_index, descending }` | Ascending, descending, or clear sort. |
| `CycleTableSort` | Add/cycle table-data sort, optionally with Shift. |
| `StartResize` | Begin column resize. |
| `Resize` | Apply column width delta. |
| `CopyColumnName` | Copy header name. |
| `CopyColumnValues` | Copy values for one column. |
| `MoveLeft` / `MoveRight` | Reorder visible column. |
| `ResetOrder` | Restore column order. |
| `ResetWidths` | Restore widths. |
| `HideColumn` | Hide selected column. |
| `AddFilter` | Add filter for selected column in table-data mode. |
| `ShowColumns` | Show hidden columns. |
| `ResetLayout` | Reset column order/width/hidden state. |
| `AutoSize` | Auto-size selected column. |

Header context-menu labels include:

- Sort Ascending.
- Sort Descending.
- Clear Sort.
- Add Filter.
- Copy Column Name.
- Copy Column Values.
- Move Column Left/Right.
- Reset Column Order.
- Reset Column Widths.
- Hide Column.
- Show Columns.
- Reset Layout.
- Auto Size.

### 3.7 Cell selection and keyboard interaction

Files: `result_grid_interaction_surface_view.rs`, `result_grid_keyboard_view.rs`, `result_grid_selection.rs`

Features:

- Select cell.
- Select row.
- Select all visible cells.
- Clear selection.
- Copy selected cell/rows.
- Keyboard navigation.
- Pasted text into selected editable cell.
- Enter/F2 to begin editing.
- Commit edit and navigate.
- Delete selected rows when editable.
- Apply staged changes.
- Discard staged changes.
- Confirmation when discarding multiple staged changes.

`ResultGridInteractionAction`:

- `CopySelectedRows`.
- `CopySelectedCell`.
- `ApplyStagedChanges`.
- `DiscardStagedChanges`.
- `DeleteSelectedRows`.
- `SubmitCellEdit { row_index, column_index }`.
- `BeginCellEdit { row_index, column_index }`.
- `CommitEditAndNavigate`.
- `Navigate`.

### 3.8 Cell context menu

File: `result_grid_cell_menu_view.rs`

Copy actions:

- Copy Cell Value.
- Copy Row (TSV).
- Copy Selected Rows.
- Copy Selected Rows with Headers.
- Copy Selected Rows as JSON.
- Copy Selected Rows as Markdown.
- Copy Selected Rows as INSERT SQL.
- Copy Row as JSON.
- Copy Row as CSV.

Editable actions when write policy allows:

- Edit Cell.
- Set to NULL.
- Revert Cell.
- Revert Row.
- Undo Delete.
- Duplicate Row.
- Delete Row.

Always available data actions:

- Filter by this value.
- Sort Ascending.
- Sort Descending.

Write-block behavior:

- Read-only columns show a lock item and a reason.
- Staged cell/row state controls whether revert actions appear.
- Table edit capability is distinct from read-only query result display.

### 3.9 Inline cell editor

File: `result_grid_cell_editor_surface_view.rs`

Features:

- Single-line editor for normal values.
- Checkbox editor for boolean values.
- Enter commits.
- Escape cancels.
- Focus is requested when editor opens.
- Edit errors clear when value changes.

`CellEditorAction`:

- `Commit`.
- `Cancel`.

### 3.10 Record inspector panel

Files: `result_grid_record_surface_view.rs`, `result_grid_edit.rs`

The `Record` toolbar button toggles a full-record inspection surface.

Features:

- Shows selected row index.
- Lists every column and compact value preview.
- Scrolls vertically for wide records.
- `Inspect` per column opens advanced cell inspector.
- `Close` hides record inspector.
- If no row is selected, shows `Select a row to inspect the full record.`

`RecordInspectorAction`:

- `Close`.
- `Inspect(column_index)`.

### 3.11 Advanced value inspector

File: `result_grid_inspector_surface_view.rs`

`ValueInspectorAction`:

- `CopyRaw`.
- `ExportBytes`.
- `Apply`.
- `Close`.

Modes by value type:

| Value kind | Modes |
|---|---|
| JSON | Raw, Pretty, Tree |
| Bytes | Raw, Hex, Base64 |
| Other | Raw, Pretty |

Features:

- Shows data type and NULL/value state.
- Shows write policy (`editable via ChangeSet`, read-only, or block reason).
- Copy raw value.
- Export decoded bytes to temporary `.bin` file.
- Pretty JSON preview.
- JSON tree preview.
- Hex/Base64 bytes preview.
- Apply edited value to ChangeSet when writable.
- Close with button or Escape.

### 3.12 Export

Files: `query_dialog_surface_view.rs`, `result_grid_export.rs`, `result_grid_view.rs`

Current output formats/actions:

- CSV.
- JSON array.
- Markdown table.
- SQL INSERT statements.
- PostgreSQL-style `COPY` formatting helpers.
- Clipboard copy for visible/selected values.
- Export dialog from Results tab.
- Byte export from advanced value inspector.

Formatting details:

- SQL identifiers are quoted and embedded quotes escaped.
- Text/JSON/bytes values are SQL-literal escaped.
- CSV/delimited values quote delimiter, quote, and newline-containing fields.
- JSON number handling avoids unsafe precision coercion for integers beyond exact JSON-safe range.

### 3.13 Chart pane

File: `query_output_panes_view.rs`

Chart controls:

- Chart type: Bar, Line, Area, Scatter, Pie.
- X column selector.
- Y numeric column selector.
- Aggregation: None, Count, Sum, Average, Min, Max.
- Series column selector.
- Projection max-points handling.
- Null/non-numeric Y skip count.
- X fallback handling for nulls/indexes.

Empty states:

- `No result to chart`.
- `No data to chart`.

### 3.14 Messages pane

Features:

- Displays query notices and execution details.
- Shows latest messages first, capped to 20 visible messages.
- Empty state: `No messages yet`.
- Diagnostics status-bar action opens Messages tab.

### 3.15 Explain pane

File: `query_output_actions_view.rs`

Features:

- Explain.
- Explain ANALYZE with explicit confirmation.
- Raw JSON toggle.
- Copy plan.
- Parsed plan tree.
- Planning time and total runtime when available.
- Findings/advisor messages with severity.
- Raw fallback when plan cannot be parsed.

Safety behavior:

- UI warns that `EXPLAIN ANALYZE` executes the statement, including writes.
- User must check `I understand this will execute the query` before running.

`QueryOutputAction`:

- `Explain`.
- `ExplainAnalyze`.
- `OpenHistory(entry, run)`.

### 3.16 Query history pane

Features:

- Recent executions list.
- Search/filter history.
- Clear history search.
- Total entry count.
- Status indicator: OK/FAIL.
- Execution duration and metadata.
- Preview of SQL.
- One-click replay/open into editor.
- Empty state and no-match state.

## 4. Current action/data flow

### Query editor flow

```text
Editor response
  → QueryEditorAction
  → DbProApp::apply_query_editor_effects
  → UiCommand / dispatch_query / save_query_document
```

### Query chrome flow

```text
QueryChromeAction
  → apply_query_chrome_action
  → run/cancel/explain/format
```

### Output flow

```text
Output tab selection
  → draw_output_pane
  → Results / Chart / Messages / Explain / History
```

### Grid flow

```text
Toolbar/header/cell/keyboard intent
  → grid action enum
  → TableDataState / TableEditingState / ChangeSet
  → clipboard, export, edit, query command, or confirmation
```

## 5. Feature boundary and current limitations visible from source

- Output is implemented as a bottom dock; a persistent right-side result panel is not visible in this source path.
- Query result grid has rich copy/export/edit/inspect behavior, but query-result editability is intentionally separate from table-data editability.
- Result export supports several formats, but the Results surface still routes through an export dialog whose exact UX should be documented separately.
- Explain parsing is strongest for PostgreSQL-shaped plans; raw fallback exists when parsing fails.
- Query context picker selects one connection and schema per document; there is no multi-target execution action in the inspected path.
- AI prediction has explicit mode and egress disclosure, but it depends on configured provider and capability state.
- Lint/diagnostic fixes are currently limited to deterministic cases; there is no general fix-all flow visible in the inspected path.
- Data grid staged mutations are guarded by write policies, ChangeSet state, and transaction/navigation guards.

## 6. Suggested next compare/documentation slices

If this feature is later compared against DBeaver/DataGrip/pgAdmin, compare these slices independently:

1. SQL editing and completion.
2. Query execution/cancellation/transactions.
3. Result grid filtering/sorting/column layout.
4. Data editing and ChangeSet/apply/discard.
5. Record/value inspector.
6. Export and clipboard formats.
7. Explain/plan visualization.
8. History/replay.
9. Charting.
10. Right-side versus bottom-dock layout behavior.

## 7. Evidence boundary

This is a source inventory based on commit `a21504ebd2a0d37e83c89e2a57ed5d731b99df1b`. It documents current functions, action variants, and visible effects. It does not establish runtime behavior against PostgreSQL or SQLite and does not claim that every listed path has been manually exercised.
