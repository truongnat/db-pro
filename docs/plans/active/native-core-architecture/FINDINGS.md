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
output-tab overrides now have one owner, `QueryOutputState`. Query-document
lifecycle orchestration now runs through `QueryDocumentContext`, which receives
explicit aggregate dependencies, keeps switching/cursor synchronization in the
same boundary, owns active-document text/binding mutations and leaves query
execution plus cross-aggregate grid invalidation at the composition root.
Query connection/schema/capability lookup is now isolated in the read-only
`QueryConnectionContext` rather than being implemented inline by root methods.
Table-editor sample generation and typed value parsing are now pure functions
in `table_editor_values.rs`, removing another non-UI concern from the root
facade.
Table mutation capability checks and staged-change transitions now use an
explicit `TableMutationContext`; reload and runtime command effects remain at
the composition boundary.
Staged transaction grouping and retry-target filtering now belong to
`TableMutationState::build_apply_plan`, leaving `apply_staged_changes` as a
boundary adapter rather than a second mutation planner.
Composite-primary-key row reload filters are also built by the mutation state,
with missing-key metadata reported as a typed transition error.
Fix in `46da8e22`: insert/duplicate row value mapping and validation now live in
the pure `table_editor_values.rs` boundary; the table view only coordinates the
selected table, staged change and feedback transitions.
Fix in `43882edb`: synthetic-data plan construction now lives in the pure
`synthetic_data.rs` boundary; table metadata, numeric inputs and deterministic
FK seed-pool generation are passed in explicitly from the view adapter.
Fix in `91a3975e`: masking preview headers, sample rows and profile mapping now
live in the pure `masking.rs` boundary; the navigation view only commits the
preview result to `MaskingState`.
Fix in `393ae525`: RLS/table-policy mutation preview planning now lives in the
pure `security_rls.rs` boundary with one shared identifier dialect; navigation
only coordinates state, feedback and command dispatch.
Fix in `932f15d4`: keyed data-diff request validation and command planning now
live in `SchemaCompareState`; navigation only resolves the active source,
allocates request identity, dispatches the effect and reports feedback.
Fix in `c8d8a81d`: monitoring state and snapshot/workload/session-control
command planning now live in `monitoring_state.rs`; navigation only adapts
active connection/request identity and dispatches feature-owned effects.
Fix in `2f16e922`: audit filter construction and selected/bookmarked export
planning now live in `audit_state.rs`; navigation only dispatches the load
effect or reports the aggregate's export result.
Fix in `a03f7a17`: the database-management state catch-all was removed; routine,
transfer, synthetic-data, masking, PostgreSQL settings, FDW, replication,
event-trigger and security aggregates now have explicit feature modules.
Fix in `d473f970`: PostgreSQL settings, FDW, logical replication and event
trigger command payload construction now belongs to their feature state modules;
navigation and event reducers only resolve identity and dispatch the effect.
Fix in `73a11cd0`: security role, membership, privilege and RLS-inspection
command construction now belongs to `SecurityState`; navigation retains only
UI validation feedback, request identity and dispatch.
Fix in `b9c42bde`: RLS preview application now also produces its `ExecuteDdl`
effect from `SecurityState`, keeping SQL ownership and empty-preview rejection
outside the composition root.
Fix in `0e4cba46`: saved-query and query-folder refresh effects now belong to
`QueryLibraryState`; connection and operation reducers only provide identity and
dispatch the resulting effects.
Fix in `4a356721`: table metadata, table DDL, paged data, row reload and DDL
execution effect construction now belongs to `TableState`; the table view keeps
only context resolution, request lifecycle and feedback.
Fix in `b13e9d5c`: migration apply and schema-workbench DDL effects now build in
their owning state aggregates; root methods retain only capability gates,
request identity and dispatch lifecycle.
Fix in `d327635a`: query-folder creation and saved-query save/rename/delete
effects now build in `QueryLibraryState`; query and sidebar views no longer
construct remote library payloads inline.
Fix in `d83a77cd`: backup/restore and file-picker effects now build from
`OverlayState`, keeping settings UI responsible only for capability gates,
confirmation and dispatch lifecycle.
Fix in `0070d374`: palette and explorer connection switching now use the
connection lifecycle's explicit `Connect` effect builder instead of constructing
runtime commands in view code.
The architecture guard now enforces that the extracted table-editor context
and value modules cannot regress to a root dependency.

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
Fix in `dbb43442`: the visual builder draft is now an explicit
`VisualQueryBuilderState`; form transitions, validation, SQL preview generation,
import and FK-join suggestion are state-owned, while the view keeps only egui
rendering and query-document/feedback adapters.

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
The root now also owns one `WorkspaceFeatureState` aggregate for shell,
local-file and named-session state; the old `workspace_files` and
`workspace_sessions` composition-root fields are removed, while the typed
aggregate keeps the shell navigation API readable.
Schema event handling now follows the same boundary: `schema_events.rs` is an
explicit-state reducer with a typed follow-up transition, while `DbProApp`
only coordinates the request to reload selected table metadata.
Agent provider/workflow events now follow the same boundary: `agent_events.rs`
reduces explicit `AgentState` and `FeedbackState`, including request-scoped
configuration failures and provider readiness, with the root limited to event
composition.
Fix in `ce304360`: workflow event routing and stale session/run/document
validation now live in `agent_events.rs`; the root wrapper only passes the
event and Agent aggregate into that reducer.
Fix in `097c121d`: document snapshots and UI-to-core Agent context mapping now
live in the pure `agent_context.rs` mapper; `AgentState` supplies explicit
document/schema inputs instead of owning the conversion algorithm.
Table runtime events now follow the same boundary: `table_events.rs` owns
request matching and table/grid state transitions, returning typed effects for
cache invalidation and staged-change retry instead of reaching through the
composition root.
Saved-query and query-folder read-model replacement now follows the same
boundary in `query_library_events.rs`; the root retains only event-routing
adapters and the reducer tests cover replacement semantics.
SQL prediction ready/failed handling now follows the same boundary in
`query_prediction_events.rs`; stale request and document-version checks stay
with the query-session state instead of the composition root.
Saved-query completion now follows the same boundary in
`query_save_events.rs`; it returns an explicit close-document transition while
the root performs only tab orchestration.
Query-history retention now follows the same boundary in
`query_history_events.rs`; the 500-entry cap is owned by the editor-state
reducer rather than the composition root.
Query queued feedback now follows the same boundary in
`query_queue_events.rs`; the request status message no longer requires the
composition root.
Database-management event state transitions now follow the same boundary in
`management_events.rs`; monitoring, audit, settings, FDW, replication, event
trigger, security and data-compare read models no longer mutate through the
composition root. Cross-feature refresh orchestration remains at the root.
File-picker state transitions and DDL completion now follow the same boundary
in `file_picker_events.rs` and `ddl_events.rs`; the root retains only workspace
opening and schema/RLS refresh side effects.
Explain completion and query cancellation now follow the same boundary in
`query_execution_events.rs`; output-tab selection, document cleanup and
cancellation history no longer mutate through the root event handler.
Single-statement query completion now follows the same boundary in
`query_result_events.rs` through an explicit `QueryResultContext`; grid
invalidation, history and output presentation no longer live in the root.
Multi-statement query completion now follows the same boundary in
`query_multi_result_events.rs`; diagnostics, history status and result
presentation remain request-scoped outside the root.
Query-local failure handling now follows the same boundary in
`query_failure_events.rs`; the mixed `events_query.rs` module is deleted and
cross-feature routing remains explicit at the composition root.
Recent-table MRU ownership now follows the state-owner rule on
`SchemaExplorerState`; workspace, palette and explorer consumers no longer
depend on a `DbProApp` recent-table facade.
Palette open lifecycle now follows the state-owner rule on `PaletteState`;
navigation, welcome, sidebar and query shortcut consumers no longer mutate
palette state through the composition root.
New-connection dialog opening now follows the feature-owner rule on
`ConnectionFeatureState`; shell entry points no longer depend on a root
`open_new_connection` facade.
Workspace close/refresh lifecycle now follows the feature-owner rule on
`WorkspaceFilesState`; the root retains only the native folder-picker command.
Workspace search/replace, task, refactor, context, schema snapshot and drift
transitions now follow the same rule on `WorkspaceFilesState`; the root only
composes the schema input required by snapshot and drift operations.
Workspace Git status/stage/unstage/diff/commit transitions and external-file
change detection now follow the same rule on `WorkspaceFilesState`; query
documents are passed in as an explicit snapshot at the view boundary.
Schema compare snapshot, diff and migration-plan transitions now follow the
same rule on `SchemaCompareState`; only migration apply remains root
orchestration because it allocates a request and dispatches provider work.
Transaction policy transitions now follow the same rule on
`QueryExecutionPolicyState` and return explicit SQL effects; the root only
dispatches the returned effect.
The capture-only native entrypoint now uses the feature-owned new-connection
helper, keeping the capture build aligned with the dialog lifecycle migration.
Named-session store mutations and persistence now follow the same rule on
`WorkspaceSessionState`; capture/restore of cross-feature layout remains
explicit composition-root orchestration.
Query output-tab override and active-tab mutations now follow the same rule on
`QueryOutputState`; the root only resolves the active document identity.
Grid projection epoch and row-identity cache invalidation now follow the same
rule on `TableDataState`; query/result reducers and table orchestration call
that explicit state API.
Query document collection invariants now follow the same rule on
`QuerySessionState`; document lifecycle helpers no longer edit the collection
and active index independently through the composition root.
Active query text, explain state, running request, result selection/count and
message projections now follow the same rule on `QuerySessionState`; the root
only coordinates cancellation, connection lookup and grid invalidation.
Query-document connection/schema metadata changes and prediction invalidation
now follow the same rule on `QuerySessionState`; the root only dispatches the
returned prediction-cancel command.
Result-grid row/cell selection and mutation-error matching now follow the same
rule on `TableDataState` and `TableMutationState`; grid cells and views no
longer call selection helpers through `DbProApp`.
Result-grid keyboard navigation target calculation now follows the same rule
on `TableDataState`; the root handles only egui input and commit-edit effects.
Activity-bar rendering now follows the same rule: its explicit renderer owns
only workspace-shell state and returns navigation intents instead of reaching
through `DbProApp`; the guard now freezes that boundary.
Named workspace-session capture, restore, duplication and persistence now
follow the same rule through `WorkspaceSessionContext`; the session module
does not expose a root facade anymore.
Grid layout scope, persistence and restore transitions now follow the same
rule on `TableDataState`; table-opening flows pass explicit connection/schema/
table identity and the former `grid_layout.rs` root facade is deleted.
Result-grid row-identity derivation and cache rebuilding now follow the same
rule on `TableDataState`; table-editor code keeps request/mutation orchestration
but no longer owns the grid identity algorithm.
Result-grid identity lookup now follows the same rule on `TableDataState`; grid
views pass table metadata explicitly and no longer depend on a root lookup
facade.
Mutation-error clearing now follows the same rule on `TableMutationState`; the
table editor keeps feedback and request orchestration but no longer implements
target matching and state clearing.
Selected-row projection now follows the same rule on `TableDataState`; the
clipboard surface consumes state-owned indexes instead of owning selection
projection.
Table metadata primary-key and column-write-policy projections now follow the
same rule on `TableState`; connection mutability remains an explicit lifecycle
concern at the composition boundary.
Active query buffer-version projection now follows the same rule on
`QuerySessionState`; query dispatchers consume the state API instead of a root
helper.
Active query running-request projection now follows the same rule on
`QuerySessionState`; query cancellation and dispatch checks read the owning
session directly.
Explain-plan/request and result-count projections now follow the same rule on
`QuerySessionState`; output, agent and navigation surfaces read query state
directly without root projection facades.
Active query messages projection now follows the same rule on
`QuerySessionState`; output and navigation surfaces consume the session-owned
message slice directly.
Active query result projection now follows the same rule on
`QuerySessionState`; output, navigation, palette and agent surfaces consume
the session-owned result.
Active query text read projection now follows the same rule on
`QuerySessionState`; the root setter remains only for prediction cancellation
and text mutation orchestration.
Active output-tab read projection now follows the same rule on
`QueryOutputState`; output rendering resolves the tab by active document
identity while tab mutations remain explicit orchestration transitions.
Active output-tab mutation now follows the same rule on `QueryOutputState`; the
query views pass active document identity directly and the root tab setter is
deleted.
Per-document output-tab routing now follows the same rule on
`QueryOutputState`; the active-document condition is evaluated by feature state
instead of query view code.
Row-identity matching now uses the published `table_events` function directly;
the composition root no longer exposes a forwarding helper.
Toast mutations now follow the same rule on `FeedbackState`; error, success and
info notifications no longer route through `DbProApp` wrappers.
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

## F14 — Navigation view still owned synthetic/transfer harness orchestration

Evidence at discovery: synthetic seed, masking preview, and file/database
transfer harness actions were implemented in `navigation_view.rs`, mixing
navigation rendering with feature-specific transfer workflows and keeping the
navigation module above 3.5k lines.

Fix in the current refactor: moved the complete harness implementation to
`crates/ui/src/transfer_harness_view.rs`. The methods remain on `DbProApp` only
as a compatibility seam for existing navigation intents; the navigation module
no longer contains transfer implementation code. The extracted module compiles
against the same state and domain APIs, with no behavior change.

Severity: P2 maintainability and feature-boundary risk, resolved for the
synthetic/transfer harness slice. The remaining view/reducer seams are tracked
as follow-up work rather than hidden behind another facade.

## F15 — Table mutation dialogs were coupled to the data editor module

Evidence at discovery: discard confirmation, pending-change review and conflict
resolution rendering occupied the middle of `table_editor_view.rs` alongside
row editing and mutation dispatch, making the editor responsible for several
independent UI surfaces.

Fix in the current refactor: moved the complete dialog slice to
`crates/ui/src/table_mutation_dialogs_view.rs`. Cross-module mutation actions
are explicit `pub(crate)` entry points; the table editor retains only the data
editing and mutation execution boundary.

Severity: P2 maintainability and feature-boundary risk, resolved for the table
mutation-dialog slice.

## F16 — Insert-row workflow was embedded in the table editor renderer

Evidence at discovery: duplicate/open/submit insert actions and the complete
insert-row dialog lived beside the table data toolbar, row editing and reload
logic in `table_editor_view.rs`.

Fix in the current refactor: moved the insert-row workflow to
`crates/ui/src/table_insert_row_view.rs`. Existing callers now cross an
explicit app-level method boundary, while parsing and sample generation remain
in the dedicated `table_editor_values` module.

Severity: P2 maintainability and feature-boundary risk, resolved for the
insert-row workflow slice.

## F17 — Security activity mixed rendering with role/RLS orchestration

Evidence at discovery: the navigation module contained the complete security
activity surface plus role, RLS and policy request/preview/apply actions,
coupling navigation composition to the security feature boundary.

Fix in the current refactor: moved that complete slice to
`crates/ui/src/security_activity_view.rs`. Navigation now only invokes the
named security activity surface; security-specific command construction and
preview transitions stay together with the security renderer.

Severity: P1 boundary risk for security feature changes, resolved for the
security activity slice.

## F18 — Management activity made navigation a feature god-module

Evidence at discovery: monitoring, health advisor, audit, PostgreSQL settings,
FDW, logical replication and event-trigger rendering plus their request
actions occupied one continuous block in `navigation_view.rs`.

Fix in the current refactor: moved the complete management activity slice to
`crates/ui/src/database_management_view.rs`. The navigation module now keeps
shell surfaces (top bar, output, transfers, diagram and schema compare) while
management feature rendering and actions have a named boundary.

Severity: P1 maintainability and feature-boundary risk, resolved for the
management activity slice.

## F19 — Management feature views were still aggregated after the first split

Evidence at discovery: the first management extraction reduced navigation, but
left monitoring, audit, FDW, replication, event-trigger and pg-settings views
inside one 1.5k-line `database_management_view.rs` aggregate.

Fix in the current refactor: deleted that aggregate and published dedicated
feature view modules (`monitoring_activity_view`, `audit_activity_view`,
`fdw_activity_view`, `replication_activity_view`, `event_trigger_activity_view`,
`pg_settings_activity_view`, and `maintenance_activity_view`). Monitoring
snapshot rendering is further decomposed into health/local, sessions, server
stats, workload and confirmation surfaces.

Severity: P1 feature-boundary risk, resolved for the management view topology.

## F20 — Runtime event transitions still inflated the composition root

Evidence at discovery: `app.rs` contained the forwarding `on_*` and request
failure transition methods even though each transition already delegated to an
explicit feature reducer.

Fix in the current refactor: moved those transitions to
`crates/ui/src/runtime_event_handlers.rs`; the event router still calls the
same `DbProApp` API, but the composition root no longer owns the event-handler
implementation block.

Severity: P2 composition-root maintainability, resolved.

## F81 — Table workspace chrome mixed rendering and feature effects

Evidence at `1dc8f5ab`: `table_view.rs` rendered breadcrumb, action buttons and
view tabs while directly opening the agent, changing query workspace state,
refreshing table requests and mutating `table_view`.

Fix at `35776e2b`: `table_workspace_surface_view.rs` owns the chrome and returns
typed `AskAgent`, `NewQuery`, `Refresh` and `SelectView` actions; the root now
only applies those effects and routes the selected content view.

Severity: P1 feature-boundary risk, resolved for table workspace chrome.

## F82 — Result-grid keyboard acquisition was coupled to the app root

Evidence at `35776e2b`: `result_grid_view.rs` read shortcut and paste events,
decided copy/apply/discard/delete/navigation behavior, and executed those
effects in one method.

Fix at `8801fdce`: `result_grid_keyboard_view.rs` reads an explicit input
context and returns typed keyboard intents; the grid root reduces them into
existing clipboard, mutation and selection services.

Severity: P1 interaction-boundary risk, resolved for grid keyboard input.

## F83 — Schema-object surface directly mutated cross-feature state

Evidence at `8801fdce`: schema-object header and view tabs directly changed
query workspace state, schema-object view state and table-data request state.

Fix at `d3414e38`: `schema_object_surface_view.rs` owns breadcrumb/tabs/open-query
rendering and returns `OpenQuery` or `SelectView`; the workspace root applies
the cross-feature effects.

Severity: P1 feature-boundary risk, resolved for schema-object chrome.

## F84 — Result-grid row gutter painting lived in the row coordinator

Evidence at `d3414e38`: `result_grid_view.rs` combined row identity/mutation
state, row-number geometry, selection handling and cell rendering.

Fix at `db6013ee`: `result_grid_row_gutter_view.rs` owns the row-number visual
surface and returns only the egui response; selection and cell side effects
remain in the row coordinator.

Severity: P2 rendering-boundary maintainability risk, resolved for row gutter.

## F78 — Workspace tab rendering owned root mutations

Evidence at discovery: `workspace_tabs_view.rs` rendered every workspace tab,
context-menu branch and query/table navigation action inside `DbProApp`.

Fix in `6a117654` and `7b6c7d0f`: moved rendering into
`workspace_tabs_surface_view.rs`, which consumes explicit feature context and
returns `WorkspaceTabsAction`; the root now only applies navigation/query/table
effects. The architecture guard rejects `DbProApp` from the surface module.

Severity: P1 feature-boundary risk, resolved for workspace tab intent mapping.

## F79 — Query editor surface owned dispatch and prediction effects

Evidence at discovery: editor painting, completion/prediction scheduling,
hover state, document mutation and query dispatch were combined in
`query_editor_panel.rs`.

Fix in `27baf92c` and `d8b7629d`: completion popup and editor interaction now
consume explicit document/state contexts; the surface returns
`QueryEditorEffects`, while `DbProApp` remains the command executor.

Severity: P1 query-feature boundary risk, resolved for editor surface intent
mapping.

## F80 — Result-grid header mixed rendering with grid mutations

Evidence at discovery: header painting, sort/reorder/resize/filter actions and
table feedback mutation were all implemented in `result_grid_header.rs`.

Fix in `67965be2`: `result_grid_header_surface_view.rs` returns typed
`GridHeaderAction` values; the root reducer applies sort, resize, layout,
filter and autosize effects. The architecture guard rejects `DbProApp` from
the header surface module.

Severity: P1 data-grid boundary risk, resolved for header intent mapping.

## F70 — Table-data loading/error placeholder lived in the composition root

Evidence at discovery: `table_data_view.rs` rendered the loading and failed
states inline inside `DbProApp`, including the retry button and its visual
state mapping. That made a reusable table-data surface depend on the root
receiver and left the feature's retry intent implicit in a widget callback.

Fix in the current refactor: moved placeholder rendering into
`table_data_placeholder_view.rs`. The module receives an explicit theme/error
context and returns `TableDataPlaceholderAction::Retry`; the root only clears
the error and schedules the table-data request.

Severity: P2 feature-boundary maintainability risk, resolved for table-data
loading/error presentation.

## F71 — Table-data pagination controls lived inside the mutation toolbar

Evidence at discovery: `table_data_view.rs` owned the page navigation, page-size
selector and offset mutation inline with refresh, filters and staged-change
actions. The toolbar therefore mixed three independent interaction lifecycles
and scheduled data reloads from widget code.

Fix in the current refactor: moved pagination rendering into
`table_data_pagination_view.rs`. It receives explicit theme/query/paging state,
mutates only pagination-local values, and returns `RequestData` or `ResetPage`
intents for the root to execute.

Severity: P2 feature-boundary maintainability risk, resolved for table-data
pagination.

## F72 — Table-data filter editor mixed local UI and reload orchestration

Evidence at discovery: the unified toolbar rendered column scope, typed
operators, draft input, filter chips and clear/edit actions inline in
`DbProApp`, while the same block also initiated filter commits and reloads.
This made the filter lifecycle difficult to reason about and coupled a large
widget closure to the composition root.

Fix in the current refactor: moved filter rendering into
`table_data_filter_view.rs`. It receives explicit query/grid/table metadata,
keeps local draft mutations in that state, and returns typed commit, reload,
remove and clear intents for the root to execute.

Severity: P1 feature-boundary maintainability risk, resolved for table-data
filter presentation and intent routing.

## F73 — Table-data sort selector mixed state changes with reload scheduling

Evidence at discovery: the toolbar computed sort labels, mutated server-sort
state, handled staged-change guards and called the reload method in one root
closure. The selector was therefore coupled to the table orchestration object
instead of exposing a small interaction contract.

Fix in the current refactor: moved sorting into `table_data_sort_view.rs`. The
view receives explicit query/grid state and returns `ReloadFromStart` or
`BlockedByStagedChanges`; the root owns feedback and request orchestration.

Severity: P2 feature-boundary maintainability risk, resolved for table-data
sorting.

## F74 — Table-data mutation/status controls lived in the root toolbar

Evidence at discovery: refresh, read-only/primary-key status, staged-change
controls, mutation-failure recovery and selection badges were all rendered in
the same `DbProApp` toolbar closure. The root mixed visual status, button
affordances and mutation command dispatch across one large interaction block.

Fix in the current refactor: moved these controls into
`table_data_mutation_toolbar_view.rs`. The view consumes explicit mutation
status and returns typed refresh, staged-change and failure-recovery intents;
the root remains the only place that executes table mutations and requests.

Severity: P1 feature-boundary maintainability risk, resolved for table-data
mutation/status controls.

## F75 — Result-grid header menu owned command decisions in the grid root

Evidence at discovery: the result-grid header drew the complete column context
menu and directly decided sort, filter, reorder, visibility, layout and sizing
requests inside `DbProApp` code. The menu was a separate interaction surface
but had no explicit action contract.

Fix in the current refactor: moved menu rendering to
`result_grid_header_menu_view.rs`. It receives explicit column/menu state and
returns typed `GridHeaderMenuAction` values; the grid root remains responsible
for applying state changes and sort/filter orchestration.

Severity: P1 feature-boundary maintainability risk, resolved for the grid
header context menu.

## F76 — Result-grid header content painting was embedded in interaction code

Evidence at discovery: PK/FK badges, column labels, data types and sort
markers were painted inline beside resize, click and context-menu handling in
`result_grid_header.rs`. The visual header contract could not be reviewed or
changed independently from interaction orchestration.

Fix in the current refactor: moved header-content painting into
`result_grid_header_content_view.rs` with an explicit visual context. The
remaining header root coordinates input, state and typed menu actions.

Severity: P2 presentation-boundary maintainability risk, resolved for grid
header content rendering.

## F77 — Result-grid cell rendering and context menu were root-coupled

Evidence at discovery: `result_grid_cell.rs` combined cell background/error
painting, value typography, a 17-command context menu and mutation/clipboard
dispatch inside `DbProApp` methods. This made the highest-frequency grid
surface depend on the composition root for both presentation and command
selection.

Fix in the current refactor: moved surface/value painting to
`result_grid_cell_surface_view.rs` and context-menu rendering to
`result_grid_cell_menu_view.rs`. The menu returns typed `GridCellMenuAction`
values; the root retains only policy preparation, selection coordination and
effect application.

Severity: P1 feature-boundary maintainability risk, resolved for grid cell
surface and menu presentation.

## F48 — Runtime transport adapter was embedded in the protocol module

Source SHA: `b85d65da`.

Evidence: `runtime.rs` combined the typed `UiCommand`/`UiEvent` contract,
connection/schema/query DTOs and the `TaskBridge` channel adapter. The
protocol is a stable message boundary, while channel ownership, bounded
draining and request-id allocation are transport concerns.

Fix in the current refactor: moved `UiCommand` and `UiEvent` into
`runtime_protocol.rs` and `TaskBridge` plus channel limits into
`task_bridge.rs`; `runtime.rs` preserves the existing public re-exports and
keeps DTO definitions separate from transport.

Severity: P1 boundary risk and P2 maintainability, resolved.

## F49 — Runtime DTO families shared one facade implementation file

Source SHA: `4bb3d21d`.

Evidence: after protocol/transport extraction, `runtime.rs` still mixed
connection draft/driver models, schema/table metadata and query/result/history
models in one implementation file. These model families have different
consumers and invariants even though they cross the same public crate API.

Fix in the current refactor: moved connection models to
`runtime_connection_types.rs`, schema/table models to
`runtime_schema_types.rs`, and query/result/history models to
`runtime_query_types.rs`. `runtime.rs` is now a 104-line facade that wires
modules, re-exports the stable API and keeps only cross-family invariants.

Severity: P2 maintainability and protocol-boundary risk, resolved.

## F50 — ER diagram view owned cross-feature mutations through `DbProApp`

Evidence at source `3f8ec33b`: the diagram canvas, layout worker, design-mode
planner and egui panel were split into several files, but each file still
implemented methods on `DbProApp`. A canvas click could therefore reach table,
query, workspace and feedback state directly from the renderer.

Impact: the diagram surface was not a real feature boundary; rendering code
could mutate unrelated feature state and the composition root was reachable
from sibling diagram modules.

Fix: `DiagramViewContext` now owns the diagram presentation inputs,
diagram/design modules return typed `DiagramAction` intents, and the root only
applies the cross-feature `OpenTable`/`ExecuteQuery` effects. The architecture
guard freezes the four diagram modules against `DbProApp` dependencies.

Severity: P1 feature-boundary risk, resolved for the ER diagram slice.

## F51 — Static palette and SQL snippet catalogs were attached to `DbProApp`

Evidence at source `74c0f870`: palette catalog construction and the shared SQL
snippet list were pure data builders, but were exposed as associated functions
on the composition root. Views and actions consequently depended on the root
for data that has no application-state dependency.

Impact: the command catalog boundary was obscured and pure catalog behavior
was harder to test or reuse without constructing the application facade.

Fix: palette catalog builders now live as module functions, while shared SQL
snippets have their own `query_snippets.rs` module with a stability test. The
architecture guard freezes both modules against `DbProApp` dependencies.

Severity: P2 maintainability and boundary risk, resolved.

## F52 — SQL diagnostics engine was attached to the composition root

Evidence at source `94b8dbb3`: parser selection, structural delimiter checks,
SQL lint rules, provider capability diagnostics and deduplication were all
associated functions on `DbProApp`; the view module also owned the diagnostics
refresh cache and debounce orchestration.

Impact: pure query analysis depended on the application facade, making the
provider contract and lint behavior harder to test independently from egui
and root state.

Fix: analysis, lint and formatting are now module functions. Diagnostics
refresh receives the explicit `QueryFeatureState`, driver and lint settings;
formatting receives the query state, task bridge and capability lookup. The
root facade dependency was removed from `query_diagnostics_view.rs`, and the
architecture guard freezes that boundary.

Severity: P1 query-core boundary risk, resolved for diagnostics/formatting.

## F53 — Small sidebar activity renderers still implemented root methods

Evidence at source `01ec23d1`: the diagram sidebar and monitoring maintenance
controls were pure renderers with narrow state needs, but were implemented as
`DbProApp` methods and mutated workspace/monitoring state through the root.

Impact: simple activity surfaces could grow an accidental dependency on every
application aggregate and were not independently composable.

Fix: the diagram sidebar now receives theme/schema data and returns a navigation
intent; maintenance controls receive `MonitoringState` directly. The root
only applies the returned navigation effect, and the architecture guard freezes
both modules against `DbProApp` dependencies.

Severity: P2 feature-boundary maintainability risk, resolved.

## F54 — Result export/value formatting was attached to `DbProApp`

Evidence at source `f64ae90d`: CSV/TSV escaping, exact-number JSON mapping,
SQL INSERT generation, PostgreSQL COPY formatting and SQL literal conversion
were associated functions on `DbProApp` inside the clipboard renderer.

Impact: provider-value/export contracts were coupled to egui clipboard state;
query export dialogs and tests depended on the composition root for pure data
transforms.

Fix: pure export/value functions now live in `result_grid_export.rs`; the
clipboard module keeps only selection, staged-cell resolution and UI output
orchestration. Export dialogs and tests call the pure module directly, and the
architecture guard freezes it against `DbProApp` dependencies.

Severity: P1 data-contract boundary risk, resolved for result export.

## F55 — Grid keyboard navigation was attached to `DbProApp`

Evidence at source `84f94f26`: arrow/tab/home/end navigation and selection
projection were implemented as a `DbProApp` method, even though the
transition only needs table data, transient editing state and feedback.
The method also reached into the root to decide when an edit should commit.

Impact: a core data-grid interaction could silently depend on unrelated root
aggregates, and keyboard selection behavior could not be exercised through a
narrow feature context.

Fix: `GridNavigationContext` now owns the three state references required by
the transition; edit commit remains an explicit table-editor orchestration
step at the caller, while navigation itself is a root-free module function.
The architecture guard freezes the boundary.

Severity: P1 table-core boundary risk, resolved for keyboard navigation.

## F47 — Schema Workbench mixed mutation planning with view rendering

Source SHA: `08acab31`.

Evidence: `schema_workbench.rs` combined the schema form compositor/sidebar
with object-mutation request construction, provider-aware preview planning and
DDL dispatch across table, column, view, index, constraint, trigger, sequence,
type, schema, extension, comment and partition modes.

Fix in the current refactor: moved mutation request construction, preview
planning, database actions and DDL application into
`schema_workbench_actions.rs`; the Workbench view keeps navigation, dependency
and documentation surfaces, while state/form/preview boundaries remain
explicit.

Severity: P1 feature-boundary risk, resolved for mutation action ownership.

## F45 — Chart projection and egui rendering shared one module

Source SHA: `0ba94810`.

Evidence: `chart_view.rs` combined chart configuration/value projection,
aggregation/downsampling and all egui painter primitives in one module.
Those paths have different dependencies and test lifecycles: the projection
engine is data-only while rendering is egui/theme-specific.

Fix in the current refactor: moved chart types, projection, numeric parsing,
aggregation, downsampling and engine tests into `chart_engine.rs`; retained
`chart_view.rs` as the renderer facade and preserved the existing public
re-exports/API.

Severity: P2 feature-boundary maintainability risk, resolved.

## F46 — Agent panel mixed thread rendering with panel composition

Source SHA: `f0ddf324`.

Evidence: `agent_view.rs` combined side-panel composition, settings/context
controls and a long message/activity/confirmation thread renderer. The thread
has its own rendering lifecycle and confirmation/result interaction surface.

Fix in the current refactor: moved the thread into
`agent_thread_view.rs`, then split empty state, messages, activities/results,
confirmation preview/actions and retry/thinking into focused render helpers.
The panel facade still owns submission, close/settings and context actions.

Severity: P1 feature-boundary risk and P2 maintainability, resolved for the
agent thread surface.

## F38 — Palette catalog and action routing were embedded in the palette view

Evidence at discovery: `palette_view.rs` combined static command/catalog data,
palette action routing and dialog rendering. That made a search index change
or a command action change require editing the same rendering-heavy module.

Fix in the current refactor: moved static catalog construction to
`palette_catalog.rs` and action routing to `palette_actions.rs`. The palette
view now owns indexing/filtering and coordinates the dialog surface; existing
action helper methods remain explicit feature-boundary entry points.

Severity: P2 palette-boundary maintainability risk, resolved for catalog and
action ownership.

## F39 — Table editor mixed data rendering with mutation lifecycle

Evidence at discovery: `table_editor_view.rs` combined the table data grid
toolbar, DDL surface, row editing, staged mutation application, conflict
recovery and table-data request orchestration in one module of more than 1,700
lines.

Fix in the current refactor: moved data-grid rendering and paging controls to
`table_data_view.rs`, and moved row editing/staged apply/retry/failure handling
to `table_mutation_actions.rs`. The remaining `table_editor_view.rs` owns DDL
and table request/filter coordination, while the mutation failure input is an
explicit `StagedApplyFailure` value rather than an unlabelled four-argument
call.

Severity: P1 table-core boundary risk, resolved for data rendering and
mutation ownership.

## F40 — Query editor panel mixed editor interaction with popup rendering

Evidence at discovery: `query_editor_panel.rs` contained the SQL editor host,
completion application, signature-help rendering, rich hover rendering and
completion explanation helpers in one module of more than 1,000 lines.

Fix in the current refactor: moved hover, signature-help and completion
rendering/application helpers to `query_editor_support.rs`. The panel now
owns the editor interaction surface and coordinates those support renderers;
its production module is reduced below the file-size boundary without
changing the editor command flow.

Severity: P1 query-editor boundary risk, resolved for support rendering.

## F41 — Table metadata view mixed unrelated schema surfaces

Evidence at discovery: `table_metadata_view.rs` rendered structure/columns,
indexes, foreign keys, constraints and dependency graphs in one module above
1,000 lines.

Fix in the current refactor: moved structure/column rendering to
`table_structure_view.rs` and foreign-key/constraint/dependency rendering to
`table_relations_view.rs`; the metadata view now owns only index rendering.

Severity: P1 schema-metadata boundary risk, resolved.

## F42 — IDE workspace model mixed types, filesystem scan and state operations

Evidence at discovery: `ide_workspace.rs` owned the workspace data contract,
filesystem traversal/indexing and all workspace mutations in one nearly
1,000-line module, including a module-level dead-code allowance.

Fix in the current refactor: moved the persisted workspace data contract to
`ide_workspace_types.rs` and bounded filesystem discovery/index construction
to `ide_workspace_scan.rs`. The remaining `ide_workspace.rs` now owns state
operations and compatibility helpers; the allowance is retained with an
explicit reason because those operations are a persisted contract while shell
adoption is incremental.

Severity: P1 workspace-core boundary risk, resolved for types and scanning.

## F43 — Files activity compositor owned every workspace tab

Evidence at discovery: `files_activity_view.rs` combined workspace header/root
selection, agent context, tree, search, migrations, tasks, graph and Git
surfaces in one nearly 1,000-line view.

Fix in the current refactor: moved the tab/tree implementations to
`files_activity_tabs.rs`; `files_activity_view.rs` now owns only the workspace
header, tab selection and surface composition.

Severity: P2 workspace-activity boundary risk, resolved.

## F44 — Workspace shell mixed tab chrome with tab-content composition

Evidence at discovery: `workspace_view.rs` combined tab lifecycle/menu/chrome
rendering with the active workspace surface switch in one module above 800
lines.

Fix in the current refactor: moved tab lifecycle/rendering to
`workspace_tabs_view.rs` and reusable tab chrome primitives to
`workspace_tab_primitives.rs`. The workspace view is now a minimal content
compositor.

Severity: P1 shell composition-boundary risk, resolved.

## F25 — Table metadata state still owned the data-query lifecycle

Evidence at discovery: `TableState` combined table metadata/DDL with the data
result, paging, filters, sorts, request slots and row-reload identity. The
`TableEditorState` aggregate therefore hid a second responsibility boundary
instead of making it explicit.

Fix in the current refactor: introduced `TableDataQueryState` and moved the
data read model plus query lifecycle there. Table metadata reducers retain only
metadata/DDL state, while table-data reducers and views receive the dedicated
query state. `SchemaLoadedContext` also makes the cross-feature reset explicit
without expanding reducer argument lists.

Severity: P1 feature-boundary risk, resolved for table data-query ownership.

## F26 — Table data-query policy and reset behavior leaked into views

Evidence at discovery: table views and the composition-root methods directly
implemented filter-operator compatibility and repeated the same result/paging
reset sequence. That made state invariants easy to diverge when a new table
surface was added.

Fix in the current refactor: `TableDataQueryState` now owns filter-operator
policy plus `reset_for_table`, `invalidate_result` and `reset_page` transitions.
Callers keep only user-flow guards and command orchestration; the query state
owns its own lifecycle invariants.

Severity: P2 maintainability and feature-boundary risk, resolved.

## F27 — Grid layout behavior leaked through `DbProApp`

Evidence at discovery: column ordering, visibility, movement, auto-sizing and
viewport width calculation were implemented as `DbProApp` methods in
`result_grid_view.rs`, even though they only mutated `TableDataState`.

Fix in the current refactor: moved those transitions to `TableDataState` and
updated the grid/header/test callers to cross the state boundary directly.
`DbProApp` retains only behavior that coordinates workspace mode, mutation
guards, query sorting and command/reload orchestration.

Severity: P2 feature-boundary risk, resolved for grid layout state.

## F28 — Table grid state still owned transient editing UI

Evidence at discovery: `TableDataState` combined grid projection/layout/
selection/cache data with cell-edit buffers, inspector mode, discard dialogs
and insert-row form state. Dialogs and result-grid editors therefore reached
through the same state object even though their lifecycles differ.

Fix in the current refactor: introduced `TableEditingState` and moved the
transient editing, inspector, confirmation and insert-row fields there.
`TableDataState` now represents the grid data model and interaction geometry;
mutation/event/dialog callers receive the editing boundary explicitly.

Severity: P1 feature-boundary risk, resolved for table editing state ownership.

## F29 — ER renderer owned Design Mode mutation orchestration

Evidence at discovery: `diagram_view.rs` combined canvas/panel rendering with
foreign-key draft parsing, mutation-plan previewing, schema-fingerprint guards
and query-runtime dispatch. The renderer therefore owned a write-oriented
workflow in addition to drawing the schema map.

Fix in the current refactor: moved Design Mode add-FK, preview and apply
actions into `diagram_design_actions.rs`. The diagram view keeps only the
Design Mode surface and delegates workflow transitions across the explicit
action boundary.

Severity: P1 feature-boundary risk, resolved for Design Mode orchestration.

## F30 — ER view mixed canvas interaction with diagram composition

Evidence at discovery: the ER module combined schema-map composition, empty
states, canvas painting, pan handling and table-opening transitions in one
renderer file. That made changes to canvas interaction coupled to toolbar and
Design Mode UI composition.

Fix in the current refactor: moved empty-state/canvas rendering, pan handling
and diagram-table navigation into `diagram_canvas_view.rs`. The source view
now coordinates schema candidates, toolbar and Design Mode while the canvas
surface owns its interaction lifecycle.

Severity: P2 feature-boundary risk, resolved for ER canvas interaction.

## F31 — ER view still owned the Design Mode surface

Evidence at discovery: `diagram_view.rs` still rendered the full Design Mode
draft panel alongside schema search, toolbar and canvas composition. That kept
draft-table/column/FK form rendering coupled to the ER map coordinator even
after mutation actions had moved out.

Fix in the current refactor: moved the Design Mode panel surface into
`diagram_design_panel_view.rs`. The coordinator now only decides when the
panel is shown; panel rendering and its controls cross the feature-owned view
boundary.

Severity: P2 feature-boundary risk, resolved for Design Mode presentation.

## F32 — Database administration state was flat in `DbProApp`

Evidence at discovery: the composition root exposed eleven unrelated
administration states (`audit`, `fdw`, `security`, `monitoring`, replication,
transfer and others) as sibling fields. Views and reducers could therefore
couple directly to a growing flat surface instead of crossing one database
administration feature boundary.

Fix in the current refactor: introduced `DatabaseManagementState` and moved
those child states behind `DbProApp.management`. Existing child state types
and command helpers remain feature-owned; only composition and access paths
changed.

Severity: P1 composition-root boundary risk, resolved for database-management
state ownership.

## F33 — Schema workspace state was split across four root fields

Evidence at discovery: schema explorer, schema workbench, schema compare and
ER diagram state were independent `DbProApp` fields even though they form one
schema workspace lifecycle and share selection/navigation context. This kept
schema feature consumers coupled to the composition root shape.

Fix in the current refactor: introduced `SchemaWorkspaceState` with explicit
`explorer`, `workbench`, `compare` and `diagram` children, then routed views,
reducers, persistence adapters and tests through the aggregate. Narrow
contexts remain narrow and continue receiving only the child state they need.

Severity: P1 composition-root boundary risk, resolved for schema workspace
state ownership.

## F34 — Navigation view owned schema comparison UI

Evidence at discovery: `navigation_view.rs` contained the schema-compare
sidebar, full diff/migration/data-compare surface and keyed data-diff command
alongside shell navigation and activity rendering. This coupled a schema
feature lifecycle to the navigation shell.

Fix in the current refactor: moved schema-compare presentation and its keyed
data-diff action into `schema_compare_view.rs`; navigation keeps only the shell
and activity surfaces and delegates compare rendering through the same
`DbProApp` feature boundary.

Severity: P1 feature-boundary risk, resolved for schema comparison UI.

## F56 — Schema Compare view depended on the composition root

Evidence at discovery: `schema_compare_view.rs` implemented its sidebar and
workspace renderer as `impl DbProApp`, so snapshot/diff form state, schema
projection, feedback and navigation were reachable through the root and the
view could directly invoke unrelated command orchestration.

Fix in `456dc9c7`: `schema_compare_view.rs` now renders through
`SchemaCompareViewContext` and returns `SchemaCompareAction`; root code only
builds the context and applies the returned navigation/command intent. The
architecture guard now rejects `DbProApp` from this view module.

Severity: P1 feature-boundary risk, resolved for schema comparison rendering.

## F57 — Schema Workbench form reached root orchestration directly

Evidence at discovery: `schema_workbench_form.rs` rendered mutable workbench
fields and directly invoked planning, DDL apply, query-document creation and
workspace navigation methods on `DbProApp`.

Fix in `c9720057`: `schema_workbench_form.rs` now consumes
`SchemaWorkbenchFormContext` and returns `SchemaWorkbenchFormAction`; the root
adapter in `schema_workbench_actions.rs` applies those intents. The form module
is now included in the architecture guard and cannot depend on `DbProApp`.

Severity: P1 feature-boundary risk, resolved for schema-workbench form UI.

## F58 — Query search overlay depended on the composition root

Evidence at discovery: `query_search_view.rs` owned the floating search UI as
an `impl DbProApp`, reaching into query editor and document state directly
while also handling cursor/selection transitions.

Fix in `43a43503`: the overlay and legacy search bar now consume
`QuerySearchContext`; the query compositor constructs that context and the
architecture guard rejects `DbProApp` from the search module.

Severity: P1 feature-boundary risk, resolved for query search UI.

## F59 — Query output tab chrome lived in the composition root

Evidence at discovery: the output-tab strip was implemented inside
`query_output_view.rs` as an `impl DbProApp`, so tab selection and dock chrome
could reach the full application facade instead of the query output aggregate.

Fix in `fe7c554f`: moved the strip to `query_output_tabs_view.rs`, which owns
`QueryOutputTabsContext` and only publishes output-tab state transitions. The
architecture guard rejects `DbProApp` from that module.

Severity: P1 feature-boundary risk, resolved for output-tab chrome.

## F60 — Query output chart and message panes depended on the root

Evidence at discovery: chart configuration/rendering and query-message
presentation lived as `DbProApp` methods beside result-grid orchestration,
allowing a presentation-only pane to reach unrelated application state.

Fix in `9fab05c8`: chart and message panes now consume
`QueryOutputPanesContext`; the root wrapper supplies only query-session state
and the architecture guard freezes the new pane module.

Severity: P1 feature-boundary risk, resolved for chart/message panes.

## F61 — Query explain/history panes invoked root actions directly

Evidence at discovery: explain-plan rendering, destructive EXPLAIN ANALYZE
confirmation, history filtering and history-open actions were all methods on
`DbProApp` inside `query_output_view.rs`.

Fix in `5d42d5e9`: those panes now consume `QueryOutputActionsContext` and
return `QueryOutputAction`; only the composition root applies Explain and
history-document intents. The new module is guarded against `DbProApp` access.

Severity: P1 feature-boundary risk, resolved for explain/history panes.

## F62 — Query results header owned selection and export controls in the root

Evidence at discovery: result-tab selection, result metrics and export intent
were rendered directly in `query_output_view.rs` alongside result-grid and
dialog orchestration. That left a presentation header coupled to the full
application facade.

Fix in the current refactor: moved the header to
`query_results_pane_view.rs`, which consumes `QueryResultsPaneContext` and
returns `QueryResultsPaneAction`; the root now only applies result selection
and export intents. The architecture guard rejects `DbProApp` from the new
module.

Severity: P1 feature-boundary risk, resolved for the result-pane header.

## F63 — Result-grid toolbar mixed rendering with root actions

Evidence at discovery: filter controls, copy commands, row-count feedback and
record inspection toggles were rendered inside `result_grid_view.rs` while
invoking clipboard and inspector methods on `DbProApp`.

Fix in `6e26738c`: moved toolbar rendering to
`result_grid_toolbar_view.rs`; it consumes table data/editing and feedback
context and returns typed copy/inspect actions for the root to apply. The
architecture guard rejects `DbProApp` from the toolbar module.

Severity: P1 feature-boundary risk, resolved for the result-grid toolbar.

## F64 — Query context strip depended on the composition root

Evidence at discovery: query breadcrumb, connection/schema chip and overflow
toggle rendering were methods on `DbProApp`, so a local query chrome surface
read connection lifecycle, query state and editor toggles through the shell.

Fix in `326f800b`: moved the strip to `query_context_view.rs`. The root now
normalizes connection/schema display data and passes an explicit
`QueryContextViewContext`; common `truncate_ellipsis` is reused instead of a
second query-specific truncation helper. The architecture guard rejects
`DbProApp` from the context module.

Severity: P1 feature-boundary risk, resolved for query context chrome.

## F65 — Grid selection projection primitives lived in a view module

Evidence at discovery: `GridSelectionLookup` and `GridSelectionCache` were
pure projection/cache types declared beside `DbProApp` grid rendering. This
made state primitives appear owned by the egui view and forced unrelated table
state to import the view module.

Fix in `764548e1`: moved both types to `result_grid_projection.rs`; the old
view and crate-root paths remain explicit re-exports for compatibility, while
the architecture guard protects the pure module from root coupling.

Severity: P2 layering/maintainability risk, resolved.

## F66 — Query Run/Stop control dispatched commands from the view root

Evidence at discovery: capability checks, running-request presentation and
Run/Stop click handling were all embedded in `query_view.rs`, coupling a small
control to query dispatch, cancellation and feedback mutation on `DbProApp`.

Fix in `072f44be`: moved the control to `query_run_control_view.rs`. It
consumes connection/capability/request state and returns typed `Run`, `Cancel`,
or blocked-action intents; the root applies the runtime command and feedback.
The architecture guard rejects `DbProApp` from the control module.

Severity: P1 feature-boundary risk, resolved for query run control.

## F67 — Query context picker mixed menu rendering with document mutation

Evidence at discovery: connection/schema menu rendering, outside-click close
handling and document selection mutations were combined in `query_view.rs`.
The picker therefore depended on the composition root for both local chrome
and feature transitions.

Fix in `ecde08d7`: moved menu rendering to
`query_context_picker_view.rs`, which returns typed selection/close intents;
the root applies `set_document_connection` and `set_document_schema`. The
architecture guard rejects `DbProApp` from the picker module.

Severity: P1 feature-boundary risk, resolved for the query context picker.

## F68 — Query parameter panel lived in the query composition root

Evidence at discovery: placeholder discovery, provider capability messaging
and in-memory parameter/secret editing were rendered directly by
`query_view.rs`, coupling a bounded editor panel to the full application
facade.

Fix in `925b8b83`: moved the panel to `query_parameters_view.rs`, which owns
`QueryParametersContext` and only receives query-session state plus the
parameter capability. The architecture guard rejects `DbProApp` from the
panel module.

Severity: P1 feature-boundary risk, resolved for query parameter editing.

## F69 — Schema Workbench secondary surfaces lived in the root view

Evidence at discovery: dependency filtering/table rendering and documentation
export controls were implemented directly in `schema_workbench.rs`, alongside
the workbench compositor and mutation orchestration.

Fix in `edd17b0f`: moved dependency navigation and docs export rendering to
`schema_workbench_secondary_view.rs`. The module consumes workbench state and
edges, returning typed docs actions; the root remains responsible for schema
document generation and opening a new query document. The architecture guard
rejects `DbProApp` from the secondary-surface module.

Severity: P1 feature-boundary risk, resolved for Workbench secondary views.

## F35 — Navigation view owned transfer activity

Evidence at discovery: the navigation module rendered backup/restore entry
points, synthetic data seeding, masking previews, transfer harness actions and
transfer job history alongside shell navigation and output surfaces. This
made database-transfer behavior a concern of the navigation shell.

Fix in the current refactor: moved the complete transfer activity surface to
`transfer_activity_view.rs`; the navigation module no longer owns transfer
rendering and only coordinates shell-level navigation.

Severity: P1 feature-boundary risk, resolved for transfer activity UI.

## F36 — Navigation view owned shell chrome and output dock

Evidence at discovery: `navigation_view.rs` rendered the application top bar,
status bar and output dock in the same module as feature activity surfaces.
Those surfaces are shell composition concerns with independent layout and
interaction lifecycles.

Fix in the current refactor: moved top bar, status bar and output dock into
`shell_chrome_view.rs`; `navigation_view.rs` now contains only the remaining
navigation activity composition.

Severity: P2 shell-boundary maintainability risk, resolved.

## F37 — Query composition owned editor tool workflows

Evidence at discovery: `query_view.rs` combined the query surface compositor
with floating find/search UI, query action menus, editor preferences, folder
creation and snippet insertion. These workflows have separate interaction
lifecycle and command concerns from the editor/output layout.

Fix in the current refactor: moved query actions into
`query_actions_view.rs` and editor search overlays into
`query_search_view.rs`. The query view now coordinates the surface while
tool-specific controls remain in feature-owned modules.

Severity: P1 feature-boundary risk, resolved for query tool workflows.

## F22 — Table editor state was fragmented across the composition root

Evidence at discovery: `DbProApp` owned `table_state`, `table_data` and
`table_mutation` as separate root fields. Table metadata, grid interaction and
staged database mutations are one feature lifecycle, but views and reducers
could reach each fragment independently through the shell.

Fix in the current refactor: introduced `TableEditorState` with explicit
`state`, `data` and `mutation` children. All table consumers now cross the
single `DbProApp.table` feature boundary, while reducer/context APIs continue
to receive the smallest state references they need.

Severity: P1 feature-boundary risk and P2 composition-root maintainability,
resolved for table-editor state ownership.

## F23 — Query state was fragmented across the composition root

Evidence at discovery: query documents, editor interaction, output tabs,
execution policy and saved-query library were five independent `DbProApp`
fields. Query views and reducers therefore depended on a flat shell surface
instead of one feature lifecycle.

Fix in the current refactor: introduced `QueryFeatureState` with explicit
`session`, `editor`, `output`, `execution` and `library` children. The app
composition root now exposes one `query` aggregate; document contexts and
event reducers still receive explicit narrow state references.

Severity: P1 feature-boundary risk and P2 composition-root maintainability,
resolved for query state ownership.

## F24 — Storage hydration was embedded in app construction

Evidence at discovery: `app_state.rs` parsed connection profiles, shell layout,
grid preferences, query documents, schema pins and workspace roots directly in
the constructor. That made startup composition own every persistence key and
made feature storage changes require editing the app initializer.

Fix in the current refactor: introduced `NativeStorageContext` with explicit
feature-state dependencies and moved key parsing/clamping plus preference-state
hydration into `app_storage.rs`. The constructor now only sequences preference,
feature and workspace restore phases; workspace-session restore still happens
after query documents and pinned tables, preserving the existing invariant.

Severity: P1 boundary risk for persistence changes, resolved for native storage
hydration.

## F21 — Native lifecycle adapter mixed persistence and frame rendering

Evidence at discovery: the `eframe::App` implementation combined storage
serialization, input normalization, runtime scheduling, shell rendering and
overlay rendering in two large methods.

Fix in the current refactor: moved the trait adapter to
`crates/ui/src/app_lifecycle.rs` and decomposed it into persistence, frame
preparation, shell and overlay helpers. The adapter methods now only sequence
those responsibilities.

Severity: P2 composition-root maintainability, resolved.

## F85 — Result-grid row rendering remained coupled to the app coordinator

Evidence at `db6013ee`: `result_grid_view.rs` still combined row layout,
gutter interaction, mutation-error presentation and cell delegation in the row
coordinator.

Fix at `748889bb`: `result_grid_row_view.rs` owns the row surface through an
explicit context and renderer trait; the root adapter keeps selection commits
and cell side effects in the existing order.

Severity: P1 rendering/interaction-boundary risk, resolved for grid rows.

## F86 — Capture evidence could request a transient blank modal frame

Evidence at `748889bb`: the default 12-frame capture settled before the New
Connection body was ready in one run, producing a centered modal with a blank
body even though a 60-frame run rendered correctly.

Fix at `e4552773`: the default settle window is 60 frames and the capture
adapter separates loading preparation, requested-surface opening, viewport
pinning, screenshot handling and frame advancement.

Severity: P2 runtime-evidence reliability risk, resolved.

## F87 — Files tree rendered workspace mutations inside the shell root

Evidence at `e4552773`: `files_activity_tabs.rs` combined recursive tree
rendering, context menus, file/folder creation, deletion, query opening and
agent-context/search mutations in `DbProApp`.

Fix at `fc28d660`: `files_tree_view.rs` owns tree and context-menu rendering
and returns typed file-tree actions; the activity root now reduces those
actions into workspace and query side effects.

Severity: P1 feature-boundary risk, resolved for Files tree activity.

## F88 — Feature adapters allocated runtime request IDs through the channel

Evidence at `6ecdfb99`: feature view and event modules called
`self.task_bridge.next_request_id()` directly in connection-adjacent,
management, query, table, agent and workspace paths. Although direct command
sends were guarded, request identity allocation still coupled feature code to
the runtime channel implementation and made the command boundary incomplete.

Fix at `5064ac20`: `DbProApp::next_request_id` is the only composition-root
port used by feature adapters, while the architecture guard rejects direct
`TaskBridge::next_request_id` calls outside `app.rs`. The shortcut dispatcher
was also split into focused handlers so the touched module has no new clean
code warning.

Severity: P1 runtime-boundary incompleteness, resolved.

## F89 — Runtime dispatch failures could erase local confirmation state

Evidence at `f37a0548`: Security password/role drafts and delete
confirmations were cleared after calling `dispatch_command`, regardless of
whether the runtime channel accepted the command. The connection-delete and
query-folder dialogs had the same unconditional clear after a best-effort
send.

Fix at `e3a8fde0`: local sensitive/confirmation state is cleared only on a
successful dispatch. Failed dispatches keep the current draft or dialog open
and surface the runtime error, so the user can retry. The Security path has a
regression test proving a failed password update preserves its draft.

The same checkpoint also removes remaining protocol construction from
`SchemaWorkbenchState` and `ExplorerConnectionContext`; both now return typed
requests and leave `UiCommand` construction to root effect adapters.

Severity: P1 lost-user-input / one-way-transition violation, resolved.

## F90 — Query feature contexts depended on runtime protocol enums

Evidence at `51d476da`: `QueryExecutionContext` returned `UiCommand` and
pattern-matched `RunQuery`/`RunQueryMulti` inside its state commit path.
`QuerySaveContext` likewise returned `UiCommand::SaveQuery`, so query feature
preparation and runtime protocol mapping could not be tested or evolved
independently.

Fix at `6ff9f937`: query execution returns `PreparedQueryRun`, query saving
returns `PreparedQuerySave`, and dedicated command adapters construct the
runtime protocol only at the composition boundary. Architecture checks reject
`UiCommand` from both feature contexts.

Severity: P1 query-core boundary risk, resolved for execution and save flows.

## F91 — Saved task dispatch interpolated untrusted SQL identifiers

Evidence at the pre-fix main state: `tasks_view.rs` built export, `VACUUM` and
`ANALYZE` statements with `format!(...)` around the saved `table` or `target`
value. A saved task containing a quote, semicolon or comment could therefore
change the statement structure; the export path also emitted `LIMIT` for every
driver and maintenance syntax without a provider capability boundary.

Fix at `0cf1ed32`: `saved_task_sql.rs` is the single pure SQL-preparation
boundary. It validates the export format, quotes each qualified identifier with
the provider's delimiter and escapes embedded delimiters, uses `TOP` for SQL
Server, maps MySQL `ANALYZE TABLE`, and rejects unsupported maintenance
operations instead of emitting provider-invalid SQL. Architecture checks now
reject the old interpolation patterns from the task dispatcher. Focused tests
cover injection-shaped identifiers, provider syntax, unsupported operations and
format validation.

Severity: P1 unsafe SQL / provider-correctness risk, resolved for Saved Tasks.

## F92 — Core state transitions were not atomic with runtime dispatch

Evidence at the pre-fix main state: command-palette connection switching
mutated the active connection before the runtime accepted the connect command;
monitoring destructive confirmations were cleared after a failed dispatch; and
restore confirmation was cleared in the presentation layer before dispatch.
Connection failure classification also inferred delete operations from the
localized feedback string instead of typed lifecycle state.

Fix at `4904c81c`: palette switching reuses the guarded Explorer connection
transition, destructive confirmation state is committed only after a successful
dispatch, and `PendingConnectionOperation` makes connection failure reduction
explicit for Connect/Test/Save/Delete. Regression tests cover failed palette
switch, maintenance dispatch, restore dispatch and typed delete-failure
classification.

Severity: P1 one-way state-transition and failure-classification risk, resolved
for the audited connection, monitoring and backup/restore paths.

## F93 — Database-management adapters still used the root as a mutable facade

Evidence at the pre-fix main state: Audit, FDW, Event Trigger and Logical
Replication activities rendered typed surfaces but applied those actions from
`impl DbProApp` modules, allowing each adapter to reach unrelated root fields
and duplicating runtime-dispatch failure policy.

Fix at `10a87a9e`: those four activities now expose explicit context objects
with feature state, connection/provider snapshots, `RuntimeCommandDispatcher`
and feedback as dependencies. Cross-feature Audit navigation is returned as a
typed `AuditActivityEffect`; only the root applies the workspace/query change.
The dispatcher now owns the common failed-send transition policy, and the
connection-delete and query-folder dialogs use the same port. The number of
UI source files declaring `impl DbProApp` fell from 68 to 64.

Severity: P1 composition-boundary leak, resolved for the audited
database-management activity adapters.

## F94 — PostgreSQL settings edit state was coupled to root and lost on dispatch failure

Evidence at the pre-fix main state: `pg_settings_activity_view.rs` implemented
the entire activity as `impl DbProApp`, and applying a session setting cleared
the edit dialog immediately after attempting dispatch. A closed runtime worker
could therefore erase the user's pending setting.

Fix at `de5b0a0f`: PostgreSQL settings now use an explicit activity context with
state, provider/connection snapshots, feedback and the runtime dispatcher.
The edit state is cleared only when the dispatcher accepts the command, with a
regression test covering a failed session-setting dispatch.

Severity: P1 composition-boundary and lost-input risk, resolved for PostgreSQL
settings.

## F95 — Security activity still used the root as a mutable facade

Evidence at the pre-fix main state: `security_activity_view.rs` implemented
the role, membership, privilege and RLS activity as `impl DbProApp`, so the
surface could reach unrelated root state and duplicate dispatch/error policy.

Fix at `277fe5fa`: Security now renders and applies typed activity actions
through `SecurityActivityContext`, with explicit Security state, table state,
connection/provider snapshots, feedback and `RuntimeCommandDispatcher`
dependencies. Root composition and follow-up request adapters construct the
context; failed password-update dispatch preserves the draft through a
regression test, and RLS pending execution is committed only after dispatch
acceptance.

Severity: P1 composition-boundary and lost-input risk, resolved for Security.

## F96 — Monitoring activity still used the root as a mutable facade

Evidence at the pre-fix main state: `monitoring_activity_view.rs` implemented
the Monitor surface, polling, confirmation transitions, runtime dispatch and
cross-feature query navigation as `impl DbProApp`, even though the monitoring
state and command planning were already extracted.

Fix at `be3962ef`: Monitor now uses `MonitoringActivityContext` with explicit
state, connection/provider snapshots, feedback and `RuntimeCommandDispatcher`
dependencies. Session/workload query opens return a typed
`MonitoringActivityEffect`; the root applies only workspace navigation and
composes the already-isolated auxiliary activities. Failed maintenance
dispatch preserves its confirmation through the existing regression test.

Severity: P1 composition-boundary and retryability risk, resolved for
Monitoring.

## F97 — Welcome adapter was split from its workspace composition boundary

Evidence at the pre-fix main state: `welcome_view.rs` declared a separate
`impl DbProApp` only to compose the Welcome surface and apply workspace,
connection and palette actions.

Fix at `ae3515ac`: Welcome rendering and its typed action application now live
beside workspace composition in `workspace_view.rs`; the redundant root module
and facade are removed without changing the Welcome surface contract.

Severity: P2 topology/maintainability risk, resolved.

## F102 — Table mutation service mixed batch planning with provider execution

Evidence at the pre-fix main state: `TableDataService::apply_mutations_detailed`
owned read-only policy checks, connection/dialect resolution, delete-update-
insert ordering, parameterized SQL construction, provider transaction dispatch,
original-input index remapping and affected-row aggregation in one method.

Fix at `c297c6dd`: `TableMutationExecution` now owns that mutation execution
boundary. `TableDataService` keeps the public API and delegates the complete
batch, while the helper separates writable validation, ordered statement
planning, transaction dispatch/failure remapping and result aggregation. The
existing atomic-ordering, provider-failure and original-index tests remain
green; the duplicated invariant-error text was also corrected.

Severity: P1 core mutation-boundary and error-contract maintainability risk,
resolved for table-editor batch mutations.

## F103 — SQL safety policy mixed classification with lexical scanning

Evidence at the pre-fix main state: `domain/safety.rs` combined the public
connection policy and statement classification API with quote/comment/dollar-
quote scanning, tokenization, parenthesis matching and data-modifying CTE
analysis in one large module.

Fix at `49e51e6e`: lexical mechanics now live in `domain/safety_lexer.rs`,
while the CTE-specific analyzer lives in `domain/safety_cte.rs`. The public
policy/classification API remains in `safety.rs`; malformed CTE bodies retain
the fail-closed destructive classification and the existing safety suite stays
green.

Severity: P1 core safety-boundary and parser maintainability risk, resolved.

## F104 — Schema service mixed introspection with provider DDL rendering

Evidence at the pre-fix main state: `SchemaService` combined cache/connection
orchestration, table metadata lookup and dependency discovery with roughly 250
lines of PostgreSQL/SQLite table, index, foreign-key, constraint and trigger
DDL formatting helpers.

Fix at `4a63295d`: provider-aware DDL rendering now lives in the dedicated
`application/schema_ddl.rs` module. `SchemaService` retains the public
introspection and execution boundary and delegates rendering without changing
the generated DDL contract; the existing schema-service tests cover the
PostgreSQL/SQLite output and identifier quoting paths.

Severity: P1 core provider-boundary and schema-service maintainability risk,
resolved for schema DDL rendering.

## F105 — Table-info projection mixed cache access with dependency discovery

Evidence at the pre-fix main state: `SchemaService::get_table_info` fetched the
introspection snapshot and then also projected columns/keys/indexes while
building and deduplicating FK, view, trigger, function and sequence dependency
edges in the same service method.

Fix at `93a4e85a`: `schema_table_info.rs` now owns the pure snapshot-to-
`TableInfo` projection and dependency graph construction. `SchemaService`
retains cache/connection orchestration and delegates the projection; the
existing schema-service dependency and table-info tests remain green.

Severity: P1 core introspection-boundary and dependency-graph maintainability
risk, resolved.

## F106 — Connection update mixed secret lifecycle with persistence and live-session recovery

Evidence at the pre-fix main state: `ConnectionService::update` combined
configuration validation, database/SSH secret migration, secret rollback,
repository persistence, active-session disconnect recovery and schema-cache
invalidation in one application method.

Fix in the current checkpoint: `connection_update.rs` now owns the update use
case through `PreparedUpdate`, `DatabaseSecretChange` and `SshSecretChange`.
`ConnectionService` keeps the stable public facade, while the extracted
boundary makes the secret plan and rollback state explicit without changing the
repository, connector or secret-store ports.

Severity: P1 core lifecycle-boundary and rollback maintainability risk,
resolved for connection updates.

## F107 — Export service mixed authorized query execution with file encoding

Evidence at the pre-fix main state: `ExportService` owned query safety and
active-connection lookup together with CSV row writing, JSON value conversion,
XLSX cell encoding and Excel precision/index guards.

Fix in the current checkpoint: `export_formats.rs` now owns CSV, JSON and XLSX
rendering from a validated `QueryResult`. `ExportService` retains the
application boundary for query authorization/execution and only wraps rendered
bytes in the public `ExportResult` contract.

Severity: P1 core application/rendering-boundary maintainability risk,
resolved for export formatting.

## F108 — Object mutation service mixed plan orchestration with DDL builders

Evidence at the pre-fix main state: `ObjectMutationService` combined
capability rejection and preview assembly with statement rendering for tables,
views, indexes, constraints, triggers, namespaces, routines and RLS policies
in one application module.

Fix in the current checkpoint: `object_mutation_builders.rs` now owns the
definition/action-to-DDL dispatch and provider-specific statement builders.
`ObjectMutationService` retains preview orchestration, unsupported capability
gates, safety classification, effects and fingerprints.

Severity: P1 core mutation-boundary and provider-DDL maintainability risk,
resolved for object mutation planning.

## F109 — Database transfer mixed conversion policy with streaming adapters

Evidence at the pre-fix main state: `db_transfer.rs` combined the provider
conversion matrix, mapping preview/capability gates and row projection with
the in-memory generator source and transaction/conflict target adapter used by
the transfer harness.

Fix in the current checkpoint: `db_transfer_plan.rs` now owns conversion
types, explicit PG/SQLite mapping classification, capability gates and row
projection. `db_transfer.rs` retains the streaming source/target adapters and
re-exports the stable planning API.

Severity: P1 core transfer-policy and adapter-boundary maintainability risk,
resolved for conversion planning.

## F110 — Monitoring snapshot mixed provider queries with service composition

Evidence at the pre-fix main state: `MonitoringService::snapshot` resolved the
connection and provider port while also collecting sessions, locks, relation
sizes, server/workload data, provider fallbacks and the user-facing snapshot
message in one method.

Fix in the current checkpoint: `monitoring_snapshot.rs` now owns the snapshot
execution/read-model assembly. `MonitoringService` retains the public facade,
provider selection and command-oriented monitoring operations; existing
provider fallback and error logging semantics are preserved.

Severity: P1 monitoring provider-boundary and read-model maintainability risk,
resolved for snapshot assembly.

## F111 — Schema diff comparator mixed naming, table, column and index concerns

Evidence at the pre-fix main state: `schema_diff.rs` implemented qualified-name
quoting plus the complete table, common-column/type-mismatch and index diff in
one 95-line comparator.

Fix in the current checkpoint: `schema_diff_compare.rs` now owns the pure
comparison boundary with named helpers for qualified sets, table-column
comparison, type lookup and index/table differences. `SchemaService` keeps
only introspection orchestration and delegates the same `SchemaDiff` contract.

Severity: P2 core comparison-boundary and maintainability risk, resolved.

## F112 — Query classification and test topology were coupled to the service facade

Evidence at the pre-fix main state: `query_service.rs` owned the SQL statement
classification helpers alongside execution orchestration, while its large test
module was embedded in the same production file. This coupled a pure safety
classification concern and test topology to the service facade, increasing the
cost of changing either boundary.

Fix in the current checkpoint: `query_classification.rs` now owns statement
classification, CTE keyword scanning and leading-comment handling. The stable
`QueryService` facade imports that classifier, and its tests live in
`application/query_service/tests.rs`; behavior and visibility remain scoped to
the application module.

Severity: P2 core classification-boundary and maintainability risk, resolved.

## F113 — SQL builder mixed table reads with row mutation rendering

Evidence at the pre-fix main state: `sql_builder.rs` combined grid read
queries, filtering/sorting/pagination, primary-key lookup and insert/update/
delete rendering behind one module, while also owning shared parameter
conversion.

Fix in the current checkpoint: `sql_builder.rs` remains the compatibility
facade and shared identifier/placeholder/parameter boundary. Read query
construction now lives in `sql_builder/read.rs`, and row mutation construction
now lives in `sql_builder/mutation.rs`. Existing callers keep the same stable
imports and SQL/parameter behavior.

Severity: P1 core SQL-builder boundary and mutation/read coupling risk,
resolved.

## F114 — Safety policy enforcement was coupled to SQL classification

Evidence at the pre-fix main state: `domain/safety.rs` defined the connection
policy value object, policy constructors/defaulting and policy validation beside
the SQL classifier, script splitter and lexical helpers. Its test module also
made the safety domain file exceed the maintainability size threshold.

Fix in the current checkpoint: `safety_policy.rs` owns the backend policy value
object and validation boundary, while `safety.rs` remains the compatibility
facade for classification and script safety APIs. Safety tests now live in
`domain/safety/tests.rs`; public imports and fail-closed behavior are unchanged.

Severity: P1 core safety-policy/classifier coupling and maintainability risk,
resolved.

## F115 — Capability data model owned provider catalogs and limitation prose

Evidence at the pre-fix main state: `DatabaseCapabilities` combined the
capability value objects, feature lookup, a large driver-specific limitation
match and all PostgreSQL/SQLite/MySQL/SQL Server preset literals in one domain
module.

Fix in the current checkpoint: `capability_presets.rs` owns provider capability
construction and `capability_limitations.rs` owns driver-specific unavailable
reasons. `capabilities.rs` remains the stable domain API for the value model
and delegates provider policy without changing any flags or public methods.

Severity: P1 provider-capability policy coupling and maintainability risk,
resolved.

## F116 — Agent workflow mixed user-error formatting with state transitions

Evidence at the pre-fix main state: `AgentToolError::format_user_error` held
several independent label maps and message assembly branches inside the agent
state-machine module, while its tests were embedded in the same file.

Fix in the current checkpoint: error formatting now delegates to focused query,
permission and confirmation-label helpers; workflow tests live in
`domain/agent_workflow/tests.rs`. State transitions, error text and public
workflow API remain unchanged.

Severity: P2 agent-domain cohesion and maintainability risk, resolved.

## F99 — Transitional Tauri startup failures were converted into panics

Evidence at the pre-fix main state: `crates/tauri-app/src/lib.rs` used
`expect` for app-data lookup, shared-runtime initialization and the final
Tauri run, making recoverable boundary failures terminate through panic and
discarding typed startup context.

Fix at `18a9869b`: setup now propagates app-data/runtime initialization errors
through the Tauri setup result, while terminal run failure is logged with its
source error instead of an opaque panic. The `pg_dump` PATH test also avoids a
test-only unwrap that polluted the full clean-code scan.

Severity: P1 startup error-handling risk, resolved for the transitional Tauri
adapter.

## F100 — Migration planning mixed all provider phases in one function

Evidence at the pre-fix main state: `MigrationPlanner::plan_from_schema_diff`
contained the complete create/add/alter/index/drop pipeline in one roughly
200-line function, while also owning operation IDs, dependency lookup,
SQLite capability decisions and final plan assembly.

Fix at `c3209277`: `MigrationPlanBuilder` owns sequence allocation and plan
assembly, with one method per migration phase and a typed pending-operation
value for construction. Public preview/fingerprint APIs are unchanged;
operation ordering, dependency IDs, destructive warnings and SQLite
unsupported markers remain covered by the core tests.

Severity: P1 core maintainability/provider-policy risk, resolved.

## F101 — Query service owned both batch orchestration and execution details

Evidence at the pre-fix main state: `QueryService::execute_multi` mixed
connection/policy lookup, statement classification, transactional dispatch,
sequential dispatch, result conversion, schema-cache invalidation and history
persistence in one method of roughly 200 lines. The same service boundary was
therefore responsible for choosing an execution mode and interpreting every
provider transaction result.

Fix at `c85f219e`: `MultiQueryExecution` now owns transactional validation,
transaction failure mapping, sequential execution and result assembly in a
dedicated application module. `QueryService::execute_multi` remains the stable
orchestration API for lookup, mode selection, cache invalidation and history
persistence. Existing multi-query routing, transaction-control rejection,
partial-result and unknown-commit tests remain green.

Severity: P1 core composition and transaction-error maintainability risk,
resolved for the multi-query boundary.

## F98 — Workspace tab adapter was split from its workspace boundary

Evidence at the pre-fix main state: `workspace_tabs_view.rs` declared a
separate `impl DbProApp` only to compose the tab surface and apply workspace
tab actions.

Fix at `ba39ca58`: workspace-tab rendering and action application now live in
`workspace_view.rs`, beside Welcome and the workspace surface composition. The
redundant root module is removed without changing tab, query, table or close
request behavior.

Severity: P2 topology/maintainability risk, resolved.
