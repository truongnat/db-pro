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
