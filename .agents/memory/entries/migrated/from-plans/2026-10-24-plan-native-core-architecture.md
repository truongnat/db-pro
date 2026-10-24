---
schema_version: 1
id: "MEM-PLAN-native-core-architecture-ed6fac5aefec"
title: "Plan archive: Native Core Architecture"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, native-core-architecture]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "ed6fac5aefecdfa1b7704525f13cecacdb1a11c52b0debb7653d8850619d69e5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-core-architecture/PLAN.md"
  - "docs/plans/active/native-core-architecture/FINDINGS.md"
  - "docs/plans/active/native-core-architecture/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-core-architecture/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native Core Architecture — Plan

State: IMPLEMENTING
Branch: `main`

## Objective

Rebuild the native UI around explicit feature-owned state and one-way
application transitions. `DbProApp` remains the egui composition root, but it
must stop being the owner of every feature's mutable state and business
transition.

## Target shape

```text
egui views
  -> feature intent
  -> feature reducer / state aggregate
  -> UiCommand
  -> TaskBridge
  -> native-app adapter
  -> runtime/core/infrastructure
  -> UiEvent
  -> feature reducer
  -> repaint
```

The UI layer owns presentation state and intent mapping only. Domain rules and
provider behavior stay in `crates/core` and `crates/infrastructure`.

## Migration order

1. Connection dialog state aggregate — completed in `f8a091eb`.
2. Connection session state and lifecycle transitions — lifecycle completed in
   `13dedb61`; saved-connection read model completed in `d9ff6b00`.
3. Workspace shell state and navigation reducer — completed in `a483000a`.
4. Query document/session state and output lifecycle — completed in
   `24f8692a`.
5. Table/data editor state and mutation effects — grid/editor interaction state
   completed in `77a27f0c`; table metadata/request state completed in
   `0fc757d8`; mutation effects and the remaining table surface adapters are
    now feature-owned in `f5fc419d`; Explain validation and transitions are
    isolated in `ef33bb1c`; saved-query preparation is isolated in
    `2b711995`; Schema Workbench mutation planning is feature-owned in
    `1d8647dc`; command dispatch bypasses were closed and guarded in
    `d687de8e`; Explorer navigation reset invariants are unified in
    `bdf2e363`.
6. Agent state and query-editor/schema-explorer state — completed; workspace
   files, diagram, database operations, palette, query execution policy, query
   library, saved tasks, named sessions, overlays, feedback, preferences and
   welcome state are also extracted.
7. Keep the runtime event dispatch table isolated in `event_router.rs`; split
   feature reducers out of `events.rs` and keep `DbProApp` as composition,
   persistence, event pump, and top-level orchestration only — completed.
8. Add architecture checks so new feature code cannot reach another feature's
   internals or reintroduce raw control paths — completed by
   `scripts/check-ui-architecture.sh` and CI.
9. Migrate native Explorer rendering in vertical slices: the complete Explorer
   surface now emits typed intents through explicit view boundaries for
   toolbar, empty state, connection nodes, database/schema trees,
   schema-object folders, tables and table details. Root effect adapters remain
   in progress alongside equivalent adapters in other large surfaces.
10. Continue the same intent/reducer boundary through the remaining large
    native surfaces. The Agent workflow/settings/header/confirmation/context
    slices, large Settings sections, table surfaces and query execution
    preparation are now explicit; Saved Tasks, Settings system /
    Appearance/Backup, Welcome, Query dialogs and the shell Output Panel now
    emit through explicit surface contexts; Query output dock geometry and tab
    chrome now use a dedicated context while pane effects remain at the root;
    shell topbar navigation and chrome now use the same boundary; Explorer
    statusbar chrome now uses the same boundary; Explorer adapters,
    Agent thread rendering and panel/context presentation now use explicit
    contexts; Security confirmation, the composed Security surface, monitoring
    presentation and result-grid viewport composition now use the same
    boundary; query panel geometry, Audit, Event Trigger, FDW, Logical
    Replication, PostgreSQL settings, workspace-files, query-actions, Visual
    Query Builder, workspace secondary-tab, table-structure, query-snippets,
    table-profile, Backup Settings, saved-query confirmation, table-indexes and
    schema-definition, saved-task confirmation, result-grid value inspector,
    record inspector, inline cell editor and sidebar shell presentation are now
    explicit; native feature dialogs now use the shared Dialog primitive;
    Audit, FDW, Event Trigger, Logical Replication and PostgreSQL settings root
    adapters are now migrated to explicit contexts; Security activity is now
    migrated to `SecurityActivityContext` and Monitoring to
    `MonitoringActivityContext`; Welcome composition is co-located with the
    workspace boundary; workspace-tab composition is co-located there as well;
    the remaining root adapters still require migration.
11. Enforce the runtime command boundary as a two-phase transition: prepare
    typed command, dispatch through `TaskBridge`, then commit local pending or
    running state only after dispatch succeeds. This is now applied to the
    connection, schema, table, query and mutation paths; remaining surfaces
    still need the same audit.

## Non-goals

- No provider behavior changes.
- No SQL semantics changes.
- No React/frontend restoration.
- No speculative service or microservice split.

## Completion criteria

- Every migrated feature state has one owner and a public transition surface.
- UI event dispatch is isolated from feature handlers and remains testable
  without egui painting.
- `DbProApp` contains no feature-specific draft/result collection once the
  corresponding feature migration is complete.
- Core behavior tests cover open/edit/duplicate/close, stale request guards,
  and error/loading/success transitions.
- Native build, workspace tests, clippy, formatting, and runtime evidence are
  recorded truthfully.


## Source: `FINDINGS.md`

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
The root now owns one `ConnectionFeatureStat

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Native Core Architecture — Verification

Source checkpoint: `357ca2dc`.

## Current change

The native UI interaction boundary is being migrated in vertical slices.
Explorer surface composition, connection nodes, database/schema trees,
schema-object folders, table-folder/table-detail rendering, the Agent surface,
large Settings sections, table surfaces and query execution preparation,
Explain transitions, saved-query preparation and Schema Workbench mutation
planning now collect typed intents/effects in feature-owned contexts. Agent
confirmation planning now owns document targeting,
patch application and continuation payload preparation outside the app root.
Direct multiline channel bypasses are guarded.
Runtime-bound request transitions now use a prepare → dispatch → commit shape:
failed dispatches do not leave fake loading, pending, connecting, deleting or
query-running state behind. This applies to connection/schema/table/query and
mutation paths, including SQL prediction requests.
Saved Tasks now follows the same surface boundary: `saved_tasks_surface_view.rs`
owns egui rendering and emits typed actions, while `tasks_view.rs` keeps only
draft persistence, scheduler policy, runtime dispatch and the root action
adapter. Per-payload dispatch is split into focused SQL, backup, export and
maintenance handlers.
Workspace-files transitions now follow the same ownership rule: panel
selection, root/environment/trust changes, directory expansion, search routing
and external-change dismissal are reduced through `WorkspaceFilesState`.
Search/replace/refactor inputs use a `WorkspaceSearchDraft` snapshot and a
typed `UpdateDraft` intent instead of mutable field references from the root.
Agent lifecycle state is now isolated from composition-root adapters in
`agent_actions.rs`; every feature `*_state.rs` module is guarded against
`DbProApp` dependencies. Dialog, query-document and editor runtime sends now
cross `RuntimeCommandDispatcher`, with direct best-effort `TaskBridge` sends
rejected by the architecture guard.
Settings Data Grid, Connections, AI, Security, Advanced and Appearance panes
now render through explicit settings contexts; `settings_view.rs` keeps
navigation, persistence/runtime adapters and the remaining backup/diagnostics
orchestration.
Backup and Restore settings now render through `SettingsBackupContext` and
return typed actions; request-id allocation and runtime command dispatch remain
at the root adapter.
The Welcome surface now renders through `WelcomeSurfaceContext`; it owns the
responsive start page, connection rows and presentation intent collection,
while `welcome_view.rs` only applies the typed root actions.
Query destructive/export/save-as/dirty-close dialogs now render through
explicit dialog contexts and return typed actions. The shell Output Panel now
owns its panel layout and tab rendering through `ShellOutputPanelContext`;
`DbProApp` retains only result selection, persistence and runtime adapters.
The plan remains `IMPLEMENTING` because other large feature surfaces
still implement rendering directly on the root and the full runtime evidence
matrix is not complete.

Result-grid inspector follow-up at source SHA `27639023`: the advanced Value
Inspector window now renders through `result_grid_inspector_surface_view.rs`
with a centered anchor, explicit value modes and typed CopyRaw, ExportBytes,
Apply and Close actions. The full-record inspector now renders through
`result_grid_record_surface_view.rs` with shared Button components and a typed
Inspect action. `result_grid_edit.rs` retains only read-model preparation,
editor-state mutation and effect adapters. The architecture guard explicitly
tracks both surfaces.

- Focused `cargo check -p db-pro-ui`: passed.
- Focused `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: passed.
- `cargo test -p db-pro-ui --quiet`: passed; 677 tests, 0 failed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed with 16 checks, 0 warnings and 0 failures.
- `git diff --check`: passed before the source commit.
- Full workspace regression after the source and documentation commits passed:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --no-fail-fast --quiet`; 404 core, 119
  infrastructure, 32 runtime, 4 tauri and 677 UI tests passed, with only
  environment-gated tests ignored.

Result-grid and sidebar shell follow-up at source SHA `e6a56fdc`: inline
boolean/text editing now renders through
`result_grid_cell_editor_surface_view.rs` and returns typed Commit/Cancel
actions. Sidebar panel geometry, clipping, chrome and resize hit-testing now
render through `sidebar_surface_view.rs`; `sidebar_view.rs` retains activity
selection and applies typed chrome/resize actions. The architecture guard
tracks both new surfaces.

- Focused UI check, clippy, architecture guard and clean-code scan: passed;
  clean scan reported 16 checks, 0 warnings and 0 failures.
- `cargo test -p db-pro-ui --quiet`: passed; 677 tests, 0 failed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- Full workspace fmt/check/clippy/test gate at this SHA: passed; 404 core,
  119 infrastructure, 32 runtime, 4 tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- The rebuilt release binary is running in terminal session `63219` for manual
  verification.

Shared-dialog follow-up at source SHA `7dec13d3`: all feature-level direct
`egui::Window::new` calls were migrated to the common `Dialog` primitive,
including monitoring, event triggers, FDW, PostgreSQL settings, replication,
query save/dirty-close, schema workbench confirmation, security, table indexes,
table structure, pending changes, row conflicts and the result-cell inspector.
Each migrated surface now gets the shared centered card, dim backdrop,
separated header, right-aligned close icon and typed footer/body actions.

- Focused UI check, clippy, architecture guard and clean-code scan: passed;
  clean scan reported 16 checks, 0 warnings and 0 failures.
- `cargo test -p db-pro-ui --quiet`: passed; 677 tests, 0 failed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `rg -n 'egui::Window::new' crates/ui/src -g '*.rs'`: no feature-level
  direct Window construction remains; the implementation is centralized in
  `components/dialog/modal.rs`.
- The rebuilt release binary is running in terminal session `34197` for manual
  verification.

- Full workspace regression at source SHA `7dec13d3` passed:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --no-fail-fast --quiet`; 404 core, 119
  infrastructure, 32 runtime, 4 tauri and 677 UI tests passed, with only
  environment-gated tests ignored.
- Follow-up guard at SHA `f2f58eb0`: `scripts/check-ui-architecture.sh` now
  fails on any direct `egui::Window::new` in native UI code, preserving the
  shared-dialog invariant in future changes.

Result-pane shell follow-up at source SHA `de775cc8`: the result selector,
row-count/export header, empty state and grid-frame presentation now live in
`query_results_surface_view.rs`. `query_output_view.rs` keeps the output-tab
router, typed action application and feature-specific export/destructive
dialogs; the result-grid renderer remains injected as a callback. The old
header-only `query_results_pane_view.rs` was removed so the shell has one
presentation boundary instead of splitting the same surface across two files.

- Focused UI check, clippy, architecture guard and clean-code scan: passed;
  clean scan reported 15 checks, 1 pre-existing file-size warning and 0
  failures.
- `cargo test -p db-pro-ui --quiet`: passed; 677 tests, 0 failed.
- `git diff --check`: passed.

### Safety policy boundary full release checkpoint at `357ca2dc`

- Full workspace gate passed:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --no-fail-fast --quiet`; 404 core, 119
  infrastructure, 32 runtime, 4 tauri and 695 UI tests passed. Environment-
  gated tests remain ignored; no test failed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed with 16 checks, 0 warnings and 0 failures.
- `git diff --check`: passed.
- The rebuilt normal release binary is running as native process PID `80343`
  with `DB_PRO_DATA_DIR=/tmp/dbpro_manual_data` for manual verification.

### Capability and agent workflow boundary checkpoint (pre-commit)

- Provider capability construction now lives in `domain/capability_presets.rs`;
  driver-specific limitation reasons live in `domain/capability_limitations.rs`.
- `AgentToolError` formatting uses focused helpers and agent workflow tests now
  live in `domain/agent_workflow/tests.rs`.
- Focused `cargo test -p db-pro-core capabilities --quiet`: passed; 9 passed,
  0 failed, 395 filtered out.
- Focused `cargo test -p db-pro-core agent_workflow --quiet`: passed; 10 passed,
  0 failed, 394 filtered out.
- `cargo fmt --all`: passed.
- `cargo clippy -p db-pro-core --all-targets -- -D warnings`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed with 16 checks, 0 warnings and 0 failures.
- `git diff --check`: passed.

### Schema diff comparator boundary checkpoint at `f3dc6e36`

- `schema_diff_compare.rs` now owns qualified-name formatting, table/index set
  differences, common-column comparison and typed mismatch lookup;
  `SchemaService::diff_schemas` remains the introspection adapter.
- Focused `cargo test -p db-pro-core schema_diff --quiet`: passed; 4 passed,
  0 failed, 400 filtered out.
- Full workspace gate passed:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --no-fail-fast --quiet`; 404 core, 119
  infrastructure, 32 runtime, 4 tauri and 695 UI tests passed. Environment-
  gated tests remain ignored; no test failed.
- `cargo build --release --locked -p db-pro-native`: passed.
- `cargo build --release --locked -p db-pro-native --features capture`: passed.
- `bash scripts/check-ui-architecture.sh`: passed.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`:
  passed with 16 checks, 0 warnings and 0 failures.
- `git diff --check`: passed.
- The rebuilt normal release binary is running as native process PID `68194`
  with `DB_PRO_DATA_DIR=/tmp/dbpro_manual_data` for manual verification.

### Monitoring snapshot boundary checkpoint at `f4c1599d`

- `MonitoringService::snapshot` now delegates provider query collection,
  fallback handling and `MonitoringSnapshot` assembly to
  `application/monitoring_snapshot.rs`.
- Focused `cargo test -p db-pro-core monitoring_service --quiet`: passed; 6
  passed, 0 failed, 398 filtered out.
- Full workspace gate passed on rerun:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace --no-fail-fast --quiet`; 404 core, 119
  infrastructure, 32 runtime, 4 tauri and 695 UI tests passed. The first
  workspace run had one transient diagram timing assertion
  (`scene_prep_1000_1280x800`); the focused test and full rerun both passed.
- `cargo build --release --locked -p db-pro

… [truncated for memory entry; see source file]

