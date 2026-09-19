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

## F11 — Sibling feature modules can still reach the composition root

Evidence: `DbProApp` remains the composition root and feature view/reducer
modules are still implemented as `impl DbProApp`, so sibling feature code can
reach other aggregates through the root (`crates/ui/src/app.rs`).

Impact: state ownership is explicit, but the Rust module boundary is not yet a
feature boundary. A new feature can still reach unrelated state by adding a
method to `DbProApp` instead of going through a typed feature facade.

Severity: P1 architectural follow-up.

Current status: `1a69b98d` split the former database catch-all into named
feature aggregates (`database_feature_states.rs`), and `02ab0cc1` plus
`ee6a1247` scoped aggregate fields to the app boundary. The connection dialog,
catalog and lifecycle storage are now private behind feature APIs; `a418764f`
also hides pending-request, pending-target and failure storage behind lifecycle
methods. `ConnectionDialogView<'a>` now owns the connection dialog renderer's
explicit dependencies, and the renderer guard rejects `DbProApp` from the
dialog view/form/advanced-panel modules. The root field allowlist and
visibility guard are enforced in CI. `connection_status.rs` now follows the
same rule: active-connection, schema and status helpers are pure functions
over explicit state, with only root wrappers retained for orchestration. The
connection lifecycle event reducer now follows the same explicit-state shape;
`connection_events.rs` no longer implements methods on `DbProApp`, and the
root only performs the follow-up connect/schema/query orchestration. The
connection deletion confirmation is also isolated in `connection/delete_dialog.rs`
with explicit overlay/catalog/lifecycle/runtime dependencies. The
query-folder confirmation is now isolated in `query_folder_delete_dialog.rs`,
and the mixed-responsibility `connection/confirm_dialogs.rs` module is deleted.
The root now owns one `ConnectionFeatureState` aggregate instead of three
independent connection fields; its catalog, lifecycle and dialog remain
separate sub-states behind that feature boundary.
remaining architectural slice is to move the other view and reducer APIs from
`impl DbProApp` onto feature-owned contexts, so sibling features cannot use
the composition root as a shared mutable facade.

## F13 — Database management state was grouped behind a catch-all aggregate

Evidence at discovery: routine, transfer, monitoring, audit, settings, FDW,
replication, event-trigger, masking, synthetic-data and security state lived
under `DatabaseOperationsState`, even though the features have different
lifecycles and safety boundaries.

Fix in `1a69b98d`: replaced the catch-all with named aggregates in
`database_feature_states.rs` and moved schema comparison state into its own
`SchemaCompareState`. `scripts/check-ui-architecture.sh` rejects the old
aggregate and file name.

Severity: P1 state-boundary risk, resolved for the database-management slice.

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
