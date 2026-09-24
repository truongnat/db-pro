# Data Activity Baseline

This document records the source-observed behavior of the **Data** sidebar activity. It explains the activity's product role, how it routes into the existing table workspace, and the identity/persistence boundaries that matter for its evaluation. This is source analysis, not runtime evidence or an implementation commitment.

The distinction used throughout:

- **Function**: a rendering, mapping, or workspace operation.
- **Action**: a typed intent returned by a view.
- **Effect**: a workspace, table-selection, query-document, or local-state change after the action is consumed.
- **Source-observed**: visible in Rust at the exact source SHA below; it does not prove every UI action was exercised.

## 1. Source basis

**Source SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (current `HEAD`, 2026-09-24).  
**Originating product issue:** [#212 — Data Activity: Recent, Pinned, and Reusable Dataset Navigation](https://github.com/truongnat/db-pro/issues/212).

Primary implementation files:

- `crates/ui/src/activity_bar_view.rs` — Data activity rail entry.
- `crates/ui/src/sidebar_view.rs` — activity-to-sidebar dispatch.
- `crates/ui/src/sidebar_activities_view.rs` — Data action consumer.
- `crates/ui/src/sidebar_data_view.rs` — pinned/recent rows and context menu.
- `crates/ui/src/schema_explorer_state.rs` — selected table, pinned and recent table state.
- `crates/ui/src/workspace_actions.rs`, `explorer_navigation.rs` — opening a table and routing to the canonical table workspace.
- `crates/ui/src/palette_view.rs`, `palette_search_view.rs` — Quick Open entries, pinning, and structure route.
- `crates/ui/src/app_lifecycle.rs`, `app_storage.rs` — local persistence and restoration.
- `crates/ui/src/table_view.rs`, `table_workspace_surface_view.rs` — central table data/structure workspace.

Line anchors in this document refer to the source SHA above.

## 2. Why the Data activity exists

Issue #212 states the intended goal: **“fast return to frequently used tables/results without duplicating the Table Data Editor or Result Grid.”** The current sidebar copy gives the same rationale: pinned tables are for “fast reopen,” and recently opened tables appear in most-recent order (`sidebar_data_view.rs:27-50`).

So **Data is a quick-access surface, not a second data editor**. The activity is a persistent launcher for pinned/recent tables. Clicking an item routes to the existing `WorkspaceTab::Table`; the central table workspace then renders `TableView::Data`, `Structure`, and other table views (`sidebar_activities_view.rs:74-78`; `table_view.rs:7-34,52-82`).

It is intentionally adjacent to, but distinct from:

- **Explorer** — connection/schema/object tree; select a table there to open its canonical table workspace.
- **Queries** — SQL documents and saved queries; Data can create a new query seeded for a table.
- **Table Data workspace** — the central editor/grid where table data is shown; it is not the sidebar activity.
- **Quick Open** — palette also exposes pinned/recent table entries, while Data keeps those entries visible in a dedicated sidebar (`palette_search_view.rs:134-174`; `palette_catalog.rs:166-172`).

The rail gives recent/pinned table access an always-available, visible home. It does not create a second table-data state model.

## 3. Activity and action pipeline

`Activity::Data` is labeled **Data** with a table icon in the core activity group (`activity_bar_view.rs:64-79`). The sidebar dispatcher calls `draw_data_activity` when selected (`sidebar_view.rs:64-69`).

```text
egui click / context-menu action
  → SidebarDataAction
  → DbProApp::draw_data_activity
  → table open / structure route / new query / pin / remove-recent
  → canonical Table or Query workspace, or SchemaExplorerState
```

`draw_data_activity` passes `pinned_tables`, `recent_tables`, and `selected_table` from `SchemaExplorerState` to `SidebarDataContext`; it then applies each typed action (`sidebar_activities_view.rs:62-94`). The view itself returns only these intents (`sidebar_data_view.rs:6-20`):

| UI action | Typed intent | Effect |
|---|---|---|
| Click table row / Open data | `OpenData(String)` | Calls `open_table`, then selects `TableView::Data` (`sidebar_activities_view.rs:74-77`). The open path protects staged table changes by requesting confirmation before switching to a different table (`workspace_actions.rs:54-84`). |
| Open structure | `OpenStructure(String)` | Calls `open_table_from_palette`, which requests metadata and routes to `WorkspaceTab::Table` with `TableView::Structure` (`sidebar_activities_view.rs:78`; `palette_view.rs:79-107`). |
| New query for table | `OpenQuery(String)` | Creates a query document, inserts `SELECT * FROM {active_schema}.{table} LIMIT 100;`, and activates Queries (`sidebar_activities_view.rs:79-86`). |
| Pin / Unpin | `TogglePinned(String)` | Adds/removes a table-name string in `SchemaExplorerState::pinned_tables` and reports feedback (`sidebar_activities_view.rs:87`; `palette_view.rs:138-161`). |
| Remove from recent | `RemoveRecent(String)` | Removes the matching table-name string from the recent list and reports feedback (`sidebar_activities_view.rs:88-90`; `schema_explorer_state.rs:114-116`). |

Each row is rendered as a table name, with a pin/table icon; row click opens data and the context menu offers Open data, Open structure, New query for table, Pin/Unpin, and—only for recent rows—Remove from recent (`sidebar_data_view.rs:72-82,85-165`). There is no table-data grid rendered inside this sidebar.

## 4. State, persistence, and overlap

### Pinned/recent state

`SchemaExplorerState` stores pins and recent tables as `Vec<String>` alongside the current selected table (`schema_explorer_state.rs:3-18`). Recent updates remove an equal string, insert the name at index 0, and cap the list at `RECENT_TABLES_MAX = 20`; removing recent also matches by the string (`schema_explorer_state.rs:103-116`; `app_types.rs:65-66`).

Pins and recents are serialized to local eframe storage under `dbpro.native.pinned-tables-v1` and `dbpro.native.recent-tables-v1`, then restored as `Vec<String>` (`app_lifecycle.rs:53-65`; `app_storage.rs:150-166`). The connection-scope reset clears the selected table and schema metadata, but leaves `pinned_tables` and `recent_tables` untouched (`schema_explorer_state.rs:43-55`). The stored/listed item itself carries **no connection id or schema**.

### Quick Open duplication

The palette turns the same pinned/recent strings into `OpenTable(table)` items (`palette_search_view.rs:134-174`). Unit tests in `app_tests.rs:6374-6412` encode both pin visibility in Quick Open and MRU recency. Data therefore provides a persistent list-oriented view of capabilities also available through the palette and Explorer row actions; its distinct value is visibility and direct management, not unique table-opening functionality.

### Canonical table workspace

`open_table` selects the table under the current active connection/schema context and routes to `WorkspaceTab::Table`; it also records the table-name string in recent history (`workspace_actions.rs:54-84`). The central workspace reads that selection, displays connection/schema/table breadcrumbs, and routes between Data, Profile, Structure, Indexes, Relations, Constraints, Dependencies, and DDL views (`table_view.rs:7-34,52-82`; `table_workspace_surface_view.rs:86-117,120-150`).

## 5. Source-observed correctness and UX boundaries

### Table identity loses its database scope

The pinned/recent/sidebar action model stores and passes only `String` table names (`sidebar_data_view.rs:7-20`; `schema_explorer_state.rs:13-15`). The same strings are persisted without connection/schema identity (`app_lifecycle.rs:60-65`; `app_storage.rs:155-166`). Opening a row then resolves it under the **currently active** connection/schema through `open_table` (`workspace_actions.rs:54-84`). On a database where another schema or connection has a same-named table, a pin/recent entry can therefore open the current-context table instead of the original object. The row also shows no origin connection/schema to warn about. This conflicts with issue #212 acceptance that duplicate table names across connections remain unambiguous.

This is a source-level identity/routing risk; a live multi-connection scenario was not run.

### Data overlaps with Quick Open and Explorer

Quick Open already lists pinned and recent entries; Explorer table rows already open data and structure. Data's unique contribution is the visible, persistent two-list surface and direct pin/remove management, not new database functionality. If that persistent access is important to the product, the activity has a clear purpose. If a minimal rail is the product goal, its current feature set could instead live as an Explorer section plus Quick Open actions.

### The label does not describe the sidebar's contents precisely

A user selecting **Data** sees pinned/recent table names, not rows of data. Rows route into the central Table workspace. There is also a central `TableView::Data` view, which makes the duplicated word “Data” refer to two different UI levels. The code's empty-state copy explains the quick-access role, but the activity label alone does not.

### Seeded SQL is assembled from raw identifier strings

The “New query for table” route interpolates the active schema and table name directly into SQL text (`sidebar_activities_view.rs:79-85`). The Explorer table context menu uses the same raw `FROM {schema}.{table}` construction (`explorer_details.rs:121-124`). This can produce malformed SQL for quoted/reserved identifiers and deserves provider-aware identifier quoting before the generated draft is trusted. The route inserts a draft into the editor; this baseline does not establish that it auto-executes or prove an injection exploit.

## 6. Assessment

**Product rationale: valid.** The Data activity gives frequently used tables an always-visible, persistent, one-click return path, while reusing the canonical Table workspace. Its purpose is supported by the originating issue and by the sidebar's own copy; it is not intended to replace Explorer, Queries, or the table grid.

**Current information architecture: ambiguous but recoverable.** “Data” implies the data grid, while the sidebar contains only table shortcuts. The feature overlaps with Quick Open and Explorer. Rename it to a concept that describes its content (for example, **Tables** or **Pinned & Recent**) or make the actual table-data workspace—not the shortcut list—the activity's primary content. Do not retain an unexplained second Data label.

**Top correctness gap: missing object scope.** Pins/recent entries need stable table identity including connection and schema. Until then, users can invoke an item under a different active database context. Treat cross-connection duplicate names as a P1 safety/target-selection concern, consistent with issue #212's own acceptance criteria.

No database provider behavior or live UI interaction was verified here.
