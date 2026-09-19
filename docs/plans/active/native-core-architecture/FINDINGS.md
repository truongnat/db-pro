# Native Core Architecture — Findings

## F1 — `DbProApp` owns multiple feature states

Evidence: `crates/ui/src/app.rs` contains shell, query, grid, schema,
connection, agent, monitoring, security, workspace and task state in one
struct. Views are split into files, but still mutate the same aggregate.

Impact: feature changes have a wide compile and regression surface; state
transitions cannot be tested independently from the application composition
root.

Severity: P1 architectural correctness/maintainability risk.

## F2 — Connection dialog state was embedded in the composition root

Evidence: the dialog draft, validation result, diagnostics and modal lifecycle
were individual `DbProApp` fields used by connection views and unrelated shell
code.

Fix in progress: these fields now live in `ConnectionDialogState`, with explicit
open/edit/duplicate/new/test/close transitions.

Severity: P1 boundary leak.

## F3 — Runtime event dispatch was centralized before feature migration

Evidence at discovery: `events.rs` drained all `UiEvent` values and the
central router mutated feature state inline.

Fix in `01d7b547`: the router is now a pure event-to-handler table; feature
reducers live in their owning event modules. The architecture check rejects
direct state access in `event_router.rs`.

Severity: resolved.

## F4 — Connection lifecycle was coupled to every consumer

Evidence: active connection identity, pending request identity, failure
markers and request flags were individual `DbProApp` fields read by explorer,
query, navigation, task, schema and event code.

Fix in `13dedb61`: those lifecycle values now have one owner,
`ConnectionLifecycleState`, with explicit clearing behavior for pending
requests and connection errors. Fix in `d9ff6b00`: the saved-connection read
model now has one owner, `ConnectionCatalogState`, including replacement and
lookup behavior.

Severity: P1 boundary leak, partially resolved.

## F5 — Shell layout and navigation state was coupled to the composition root

Evidence: activity selection, workspace tab, panel visibility/geometry,
welcome lifecycle and pending navigation were individual `DbProApp` fields
consumed by the shell, sidebar, query, explorer and workspace actions.

Fix in `a483000a`: those values now have one owner, `WorkspaceShellState`;
panel geometry is clamped through state setters and shell tests cover defaults
and boundary clamping.

Severity: P1 boundary leak, resolved for the shell slice.

## F6 — Query document lifecycle was coupled to the composition root

Evidence: open documents, active document selection, selected text, save/close
request maps and Save As/dirty-close state were individual `DbProApp` fields
read by query, agent, workspace and runtime event code.

Fix in `0bde3a5f`: document lifecycle values now have one owner,
`QuerySessionState`. Fix in `24f8692a`: output-tab selection and per-document
output-tab overrides now have one owner, `QueryOutputState`.

Severity: P1 boundary leak, resolved for document lifecycle.

## F7 — Table/data interaction state was coupled to the composition root

Evidence: grid projection/layout, filtering/sorting, selection, cell editor,
record inspector and insert-row drafts were individual `DbProApp` fields
consumed by table, result-grid, explorer and query event code.

Fix in `77a27f0c`: those interaction values now have one owner,
`TableDataState`, with its own default-state test. Table metadata requests and
mutation orchestration remain a separate follow-up.

Severity: P1 boundary leak, resolved for interaction state.

## F8 — Table metadata and request state was coupled to the composition root

Evidence: table introspection, DDL/data requests, paging, table-data filters,
metadata searches, table details, table view selection and row reload tracking
were individual `DbProApp` fields consumed across table views and event
handling.

Fix in `0fc757d8`: those values now have one owner, `TableState`, with a
default-state test. Mutation/change-set effects remain intentionally separate
for the next migration slice.

Severity: P1 boundary leak, resolved for metadata/request state.

## F9 — Query editor state was coupled to the composition root

Evidence: `DbProApp` held editor overlays, visual query builder drafts,
diagnostics caches, problem filters and query history as unrelated fields.

Fix in `941db901`: these values now have one owner, `QueryEditorState`, with a
default-state test. The editor view and query event paths address the aggregate.

Severity: P1 boundary leak, resolved for the query-editor slice.

## F10 — Schema explorer state was coupled to the composition root

Evidence: schema loading, selection, navigation cache, pinned/recent tables and
object-view state were spread across `DbProApp` and explorer consumers.

Fix in `f96dfce1`: these values now have one owner, `SchemaExplorerState`, with
a default-state test.

Severity: P1 boundary leak, resolved for the explorer slice.

## F11 — The remaining composition root is still too broad

Evidence: `DbProApp` still owns palette state, IDE/Git state, routine/transfer
state, monitoring/audit/admin/security state, schema workbench/migration state,
transaction state, diagram state and saved-task state in addition to runtime
orchestration (`crates/ui/src/app.rs`).

Impact: the extracted aggregates reduce coupling, but new feature work can still
reach unrelated state through the composition root and the centralized event
dispatcher.

Severity: P1 architectural follow-up.

Current status: the root now contains only an allowlisted set of feature
aggregates, shell composition state, presentation context and the task bridge;
the allowlist is enforced in CI. The deeper privacy boundary between sibling
feature modules (private aggregate fields plus reducer-only APIs) remains the
last architectural hardening slice.

## F12 — Legacy agent command/event path bypassed the workflow boundary

Evidence at discovery: `RunAgent` and `ExecuteAgentTool` remained in the UI and
runtime command enums, while the UI router ignored the corresponding tool
completion events. The active agent surface already used `AgentWorkflow`, so the
legacy path could emit runtime events with no state transition consumer.

Fix in `a096664e`: removed the legacy commands, runtime events, translation
branches and no-op router arm. Agent execution now has one runtime boundary:
`StartAgentWorkflow` / `ContinueAgentWorkflow` / `CancelAgentWorkflow` and
`AgentWorkflow` events.

Severity: P1 event-contract correctness, resolved.
