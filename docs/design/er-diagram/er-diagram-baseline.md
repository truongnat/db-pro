# ER Diagram Baseline

> Status: source analysis; no implementation or runtime verification performed.
>
> Source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`  
> Research date: 2026-09-24

## Scope

This document analyzes the **ER Diagram activity and central diagram workspace**: entry points, schema graph construction, search/large-schema behavior, navigation, and the optional Design Mode. It does not claim that the native UI was launched or that PostgreSQL/SQLite behavior was exercised.

The activity-rail entry and workspace tab are related but not identical state transitions. The activity rail selects `Activity::Diagram` and `WorkspaceTab::Diagram`; connection-row and palette entry points can select the diagram workspace tab without necessarily selecting the Diagram sidebar activity.

## Evidence boundary

All source claims below refer to SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. Line anchors are source locations at that SHA. These findings do not establish rendered appearance, database/provider behavior, or completion of runtime acceptance gates.

## Surface and dispatch

- The activity rail labels `Activity::Diagram` as “ER diagram” (`crates/ui/src/activity_bar_view.rs:85-92`). The normal `OpenDiagram` action sets both the activity and active workspace tab, then opens the sidebar (`crates/ui/src/app_lifecycle.rs:213-216`). The sidebar's Diagram branch draws a schema navigation panel; its “Back to Explorer” action switches the sidebar activity back to Explorer (`crates/ui/src/sidebar_view.rs:75-79`).
- The actual diagram is a central `WorkspaceTab::Diagram`. `draw_diagram` receives `DiagramState`, `SchemaExplorerState`, active driver, and connection state (`crates/ui/src/diagram_view.rs:4-17`; workspace dispatch in `crates/ui/src/workspace_view.rs:92-107`).
- Explorer connection context menu “View ER Diagram” activates the Diagram tab and connects that connection if needed (`crates/ui/src/explorer_connection_row_view.rs:202-213`; `crates/ui/src/explorer_connections.rs:46-50`). This path does not itself assign `Activity::Diagram`.
- The Diagram sidebar panel is a schema summary/navigation surface, not the canvas. The canvas and toolbar live in the central workspace.

### Action pipeline

```text
Activity rail / Explorer connection menu / palette
  → WorkspaceTab::Diagram and (for rail path) Activity::Diagram
  → workspace_view builds DiagramViewContext
  → draw_diagram reads SchemaExplorerState.schema.table_details
  → ER graph + spatial index + optional background layout worker
  → canvas click emits OpenTable(name)
  → app.apply_diagram_action opens the table workspace

Design Mode controls
  → draft model and undo/redo state
  → ObjectMutationService preview plan
  → explicit apply emits ExecuteQuery(sql)
  → active query runtime dispatch
```

The canvas hit action carries the table name and opens the table surface (`crates/ui/src/diagram_canvas_view.rs:118-129`; `crates/ui/src/workspace_actions.rs:42-50`). Design Mode's apply routes generated SQL through the existing query runtime rather than mutating from a canvas gesture (`diagram_design_actions.rs:55-83`; `workspace_actions.rs:42-50`).

## Current interaction and graph behavior

- The graph is built from `SchemaExplorerState.schema.table_details`; an edge is created from foreign-key metadata only when the referenced `(schema, table)` is present in the graph (`crates/ui/src/diagram_view.rs:107-138`; `crates/ui/src/diagram/model.rs:82-118`). A foreign key whose target is not among loaded table details has no graph edge in this graph.
- Table cards present schema-qualified table names, columns, primary/foreign-key markers, and data types at the applicable level of detail. Large cards abbreviate columns and show a remaining-column count (`crates/ui/src/diagram_view.rs:442-613`). Composite foreign keys are represented by one relationship edge; the edge's anchor uses the first source/target column (`diagram/model.rs:108-153`).
- Canvas supports pan/zoom, spatial-index hit testing, and opening a table by clicking its node (`diagram_canvas_view.rs:60-149`). Rendering uses a spatially culled scene and levels of detail (`diagram_canvas_view.rs:87-104`; `diagram/diagram_scene.rs`).
- Schemas above `ER_LARGE_SCHEMA_THRESHOLD` (200 tables) default to search mode unless “Show all” is selected. Search matches tables/columns, shows a neighborhood of one or two FK hops, and caps the neighborhood at 100 graph nodes (`diagram/model.rs:10-13`; `diagram_view.rs:17-21,69-97,232-269`). The graph itself is still constructed from all loaded table details before filtering the visible neighborhood (`diagram_view.rs:107-138`).
- Layout rebuilds keep the old graph renderable while a background worker computes a replacement. Results are accepted only when both request id and graph version match (`diagram_view.rs:23-30,107-153`).
- Empty states distinguish no loaded schema, large-schema search prompt, and no matching tables (`diagram_canvas_view.rs:8-58`).

## Design Mode

Design Mode is opt-in and starts disabled (`diagram_state.rs:27-51`). The panel describes itself as a draft backed by `ObjectMutationService` and a schema fingerprint (`diagram_design_panel_view.rs:5-32`). Its current UI supports adding a draft table, adding a column, entering a foreign key, undo/redo, discard, SQL preview, and explicit apply (`diagram_design_panel_view.rs:34-145`). The draft model stores tables, columns, and foreign keys with bounded undo history (`diagram/design_mode.rs:13-58,60-145`).

Preview converts draft tables and foreign keys into `ObjectMutationRequest` create operations and calls `ObjectMutationService::plan`; apply checks connectivity and fingerprint, then dispatches the reviewed SQL via the Query workspace/runtime (`diagram/design_mode.rs:155-233`; `diagram_design_actions.rs:28-83`; `workspace_actions.rs:42-50`). This source path does not show graph gestures editing persisted objects.

### Design Mode scope boundary

The current UI is narrower than the full ER Design Mode issue scope. Source-visible actions create draft tables/columns/foreign keys; it does not expose rename/remove column, modification of persisted objects, index editing, or node repositioning in the canvas (`diagram_design_panel_view.rs:34-125`; `diagram/design_mode.rs:155-233`). The fingerprint hashes only the ordered `schema.table` name list (`diagram/design_mode.rs:138-153`; `diagram_design_actions.rs:55-68`), not column definitions or FK metadata. Therefore a change to metadata of an existing table while table names remain the same is not represented by that stale check.

### Design Mode can report an apply that was not dispatched

`er_design_apply_plan` clears the draft before returning `ExecuteQuery(sql)` (`diagram_design_actions.rs:76-83`). The handler then replaces the active query text, calls `dispatch_query`, ignores its boolean result, and unconditionally sets “Design Mode mutation plan applied via query runtime” (`workspace_actions.rs:42-50`). `dispatch_query` can return `false`, including while another query is running or when its destructive-run gate holds execution (`events_query_dispatch.rs:110-143`). In those cases, the draft has already been discarded and the source path still emits success feedback. This is a source-observed failure path; runtime reproduction was not performed.

## Source-observed correctness risk

### Existing graph can remain stale after same-size schema transition

`ensure_diagram_graph` decides whether to rebuild using only node count and `graph.schema_version` (`diagram_view.rs:107-110`). In that same function, the graph version is advanced only after `graph_dirty` has already become true (`diagram_view.rs:127-138`). The schema-load reducer replaces `SchemaExplorerState.schema` but does not invalidate `DiagramState.graph` or advance its version (`schema_events.rs:37-60`); connection-scope reset clears the Explorer schema and selection, not DiagramState (`schema_explorer_state.rs:43-55`; `explorer_navigation.rs:177-180,214-217`).

Consequently, after a connection/schema switch to a different table set with the same number of tables, the cached graph can satisfy both dirty checks and continue to render prior table and FK details. The graph-canvas rendering reads cached graph nodes and edges (`diagram_canvas_view.rs:88-103`), so this is a source-supported stale-state risk. It is particularly visible after switching to an empty schema and then to a different same-size schema: the empty-state early return does not clear the previous graph (`diagram_view.rs:32-35`). Severity: **P1** — users can see and open objects from a different schema context.

Existing graph tests prove node-count mismatch marks different-sized graphs distinguishable, and same count/version is clean for the same data (`diagram/tests.rs:313-330`). They do not cover different schema/table metadata with identical count and version. No test was run for this analysis.

### Layout requests can be reissued while a replacement graph is pending

When a non-empty graph becomes dirty, `ensure_diagram_graph` increments `schema_version` and dispatches a layout request but leaves the old graph installed until a matching worker result arrives (`diagram_view.rs:107-153`). On a subsequent frame before that result is accepted, the old graph version still differs from the incremented state version, so the same dirty condition can dispatch another request and advance the version again. That makes earlier results stale by version/request id and can delay replacement under slow layouts. This is a source-level scheduling risk; runtime frequency and impact were not measured.

## State and responsibility boundaries

- `DiagramState` owns canvas viewport/search, Design Mode draft state, graph/spatial index, and background layout-worker state (`diagram_state.rs:3-24`).
- `SchemaExplorerState` owns the introspected schema read model used as the graph source (`diagram_view.rs:4-9`; `schema_events.rs:37-60`).
- `ObjectMutationService` plans DDL from explicit draft requests; the Query runtime performs the apply dispatch (`diagram/design_mode.rs:155-233`; `workspace_actions.rs:42-50`).
- PostgreSQL/SQLite runtime outcomes, SQL transaction semantics, and visual acceptance are not established by these source paths.

## Evidence status

Source review only at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`. No build, test, database runtime, or native UI screenshot/recording was run. The same-size stale-graph scenario is a source-derived risk, not a reproduced runtime result.