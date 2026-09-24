# Sidebar Explorer Baseline

This document records the current source-observed feature set of the first sidebar tab: **Explorer / Database Navigator**. It is a design baseline for later compare-feature documentation, not runtime evidence.

The important distinction in this document is:

- **Function**: a Rust rendering, mapping, or reducer function.
- **Action**: a typed UI intent emitted by a view and consumed by the application root.
- **Effect**: the state/workspace/runtime change made after an action is consumed.

## 1. Source basis

Main source files:

- `crates/ui/src/sidebar_view.rs`
- `crates/ui/src/sidebar_surface_view.rs`
- `crates/ui/src/sidebar_chrome_view.rs`
- `crates/ui/src/explorer_view.rs`
- `crates/ui/src/explorer_surface_view.rs`
- `crates/ui/src/explorer_toolbar_view.rs`
- `crates/ui/src/explorer_connection_node_view.rs`
- `crates/ui/src/explorer_connection_row_view.rs`
- `crates/ui/src/explorer_database_node_view.rs`
- `crates/ui/src/explorer_schema_node_view.rs`
- `crates/ui/src/explorer_schema_tree_view.rs`
- `crates/ui/src/explorer_schema_objects_view.rs`
- `crates/ui/src/explorer_schema_object_folders_view.rs`
- `crates/ui/src/explorer_schema_object_row_view.rs`
- `crates/ui/src/explorer_table_folder_view.rs`
- `crates/ui/src/explorer_table_row_view.rs`
- `crates/ui/src/explorer_table_details_view.rs`
- `crates/ui/src/explorer_connections.rs`
- `crates/ui/src/explorer_details.rs`
- `crates/ui/src/explorer_folders.rs`
- `crates/ui/src/explorer_navigation.rs`

## 2. Overall action pipeline

Explorer follows a presentation-to-reducer pipeline. Views do not directly execute database operations; they return typed actions.

```text
UI click / context-menu item
  → local view action
  → ExplorerSurfaceAction
  → DbProApp::apply_explorer_surface_action
  → nested action mapper
  → DbProApp reducer/helper
  → workspace state, overlay state, command dispatch, or clipboard
```

### Action layers

| Layer | Enum | Variants | Consumer |
|---|---|---|---|
| Sidebar chrome | `SidebarChromeAction` | `OpenCommandPalette`, `NewConnection`, `NewQuery` | `DbProApp::draw_sidebar` |
| Sidebar shell | `SidebarSurfaceAction` | `Chrome(...)`, `Resize(f32)` | `DbProApp::draw_sidebar` |
| Explorer surface | `ExplorerSurfaceAction` | `NewConnection`, `RefreshSchema`, `Connection`, `Schema` | `DbProApp::apply_explorer_surface_action` |
| Schema tree | `ExplorerSchemaTreeAction` | `RefreshSchema`, `ActivateSchema(String)`, `SchemaObjects(...)` | `DbProApp::apply_schema_tree_action` |
| Schema contents | `ExplorerSchemaObjectsAction` | `SelectTable(String)`, `TableRow`, `SchemaObject` | `apply_schema_objects_action` |
| Connection row | `ConnectionRowAction` | connection actions listed below | `apply_connection_row_action` |
| Table row | `TableRowAction` | table actions listed below | `apply_table_row_action` |
| View/function/trigger row | `SchemaObjectRowAction` | `Open`, `OpenQuery`, `Modify`, `Drop`, `CopyName` | mapped by `SchemaObjectFoldersView` |
| Schema object folder | `SchemaObjectFolderAction` | `Open`, `OpenQuery`, `ModifyView`, `DropObject`, `CopyName` | `apply_schema_object_folder_action` |

## 3. Sidebar shell functions and actions

### `DbProApp::draw_sidebar`

File: `sidebar_view.rs`

Responsibilities:

1. Resolve the active connection display name, falling back to `DB Pro`.
2. Build shortcut labels for command palette, new connection, and new query.
3. Build `SidebarSurfaceContext`.
4. Call `sidebar_surface_view::draw`.
5. Consume shell actions:
   - `OpenCommandPalette` → open command palette in `Commands` mode.
   - `NewConnection` → call `self.connection.open_new()`.
   - `NewQuery` → create query document and activate `WorkspaceTab::Query`.
   - `Resize(width)` → persist the new sidebar width through `workspace.set_sidebar_width`.

### `draw_sidebar_activity_content`

- If activity is `Explorer`, calls `draw_explorer_sub_panes` without wrapping it in a generic activity scroll area.
- Other activity tabs use a vertical scroll area and call `draw_sidebar_activity_body`.

### `draw_sidebar_activity_body`

Dispatches non-Explorer activities such as Queries, Files, Data, History, Problems, Transfers, Monitor, Security, Settings, Diagram, Schema, Compare, and Tasks. Explorer itself is handled by the branch above and is unreachable in this function.

### `sidebar_surface_view::draw`

Functions:

- `draw_panel`: creates the fixed-width egui side panel.
- `draw_content`: lays out chrome, separator, and activity content.
- `content_rect`: calculates horizontal and vertical padding.
- `sidebar_ui`: creates the clipped content UI.
- `draw_resize_handle`: handles horizontal dragging at the sidebar edge.
- `resize_stroke`: selects resize-line styling for idle, hover, and drag states.

Action emitted:

- `SidebarSurfaceAction::Chrome(SidebarChromeAction)`.
- `SidebarSurfaceAction::Resize(width)` while dragging.

### `SidebarChromeContext::draw`

Calls:

- `draw_connection_launcher`: active connection selector/search affordance and plus button.
- `draw_new_query`: full-width New query button.

Actions:

| UI | Action | Effect |
|---|---|---|
| Active connection selector | `OpenCommandPalette` | Opens command palette. |
| Plus button | `NewConnection` | Opens new connection flow. |
| New query button | `NewQuery` | Creates and activates a query document. |

## 4. Explorer entry and surface functions

### `DbProApp::draw_explorer_sub_panes`

Builds `ExplorerSurfaceContext` using:

- connection catalog and lifecycle state;
- schema explorer state;
- active schema;
- selected table metadata;
- reduced-motion preference;
- provider capability for functions;
- primary modifier shortcut.

Then calls `ExplorerSurfaceContext::draw` and dispatches every returned action through `apply_explorer_surface_action`.

### `ExplorerSurfaceContext::draw`

Responsibilities:

1. Draw toolbar if the connection catalog is non-empty.
2. Create the navigator scroll area.
3. Draw either empty state or connection nodes.
4. Return all nested actions as `Vec<ExplorerSurfaceAction>`.

### `draw_toolbar`

Calls `ExplorerToolbarContext::draw_toolbar`.

- `RefreshSchema` is converted to `ExplorerSurfaceAction::RefreshSchema`.
- The toolbar's `NewConnection` action is intentionally ignored because the shell owns the global new-connection action.

### `draw_empty_state`

Calls `ExplorerToolbarContext::draw_empty_state`.

- `NewConnection` becomes `ExplorerSurfaceAction::NewConnection`.
- Refresh is ignored in the empty state.

### `draw_connections`

For each catalog connection:

1. Determines whether it is active and connected.
2. Builds `ExplorerConnectionNodeModel`.
3. Builds a schema tree model only for connected connections.
4. Calls `ExplorerConnectionNodeView::draw`.
5. Converts connection and schema actions to `ExplorerSurfaceAction`.

### `schema_tree_model`

Builds the model passed to `ExplorerSchemaTreeView`, including connection id, database name, active schema, schema loading/error state, selected table, table metadata, reduced-motion state, and function capability.

## 5. Explorer toolbar functions and actions

File: `explorer_toolbar_view.rs`

### `ExplorerToolbarContext::draw_toolbar`

Renders:

- `SearchInput` bound to `SchemaExplorerState::explorer_search`.
- Refresh icon button.
- Refresh context menu item with `F5`.

Both direct click and context-menu click emit:

- `ExplorerToolbarAction::RefreshSchema`.

### `ExplorerToolbarContext::draw_empty_state`

Renders:

- database icon;
- `No connections` title;
- explanatory text;
- New connection button.

New connection emits:

- `ExplorerToolbarAction::NewConnection`.

## 6. Connection node functions and actions

### `ExplorerConnectionNodeView::draw`

Creates `ConnectionRowContext` and calls `ConnectionRowContext::draw`.

When the connection row is open:

- connected → draws the database/schema tree;
- failed → draws retry hint and can emit `Connect`;
- connecting → draws `Connecting…` hint;
- disconnected → draws connect hint and can emit `Connect`.

### `ConnectionRowContext::draw`

Functions involved:

- `draw_row`: paints the connection tree row.
- `apply_row_interaction`: handles click/chevron expansion and implicit connect.
- `status_dot`: selects status color.
- `icon_color`: selects database icon color.
- `badge_text`: returns `PG`, `SQLITE`, or `ERR`.
- `connection_context_menu`: builds context-menu actions.

A normal row click behaves as follows:

| State | Click result |
|---|---|
| Connected | Toggle open/closed. |
| Disconnected | Emit `Connect` and open the row. |
| Connecting | Does not emit another connect request. |
| Failed | Opens failed state; retry hint emits `Connect`. |
| Chevron | Only toggles expansion. |

### `ConnectionRowAction` mapping

Handled by `DbProApp::apply_connection_row_action` in `explorer_connections.rs`.

| Action | Meaning | Effect |
|---|---|---|
| `Connect` | Start connection to this catalog entry. | Calls `connect_to_connection`; builds and dispatches connect command if guards pass. |
| `Disconnect` | Explicitly disconnect. | Calls `disconnect_from_connection`; clears connection-scoped state unless transaction guard blocks it. |
| `Reconnect` | Disconnect then connect again. | Calls disconnect followed by connect. |
| `RefreshSchema` | Re-introspect the connection schema. | Calls `request_schema_introspection(connection.id, true)`. |
| `NewScript` | Create SQL editor document for this connection context. | Creates query document and activates `WorkspaceTab::Query`. |
| `OpenErDiagram` | Open ER diagram. | Activates `WorkspaceTab::Diagram`; connects first if needed. |
| `AskAgent` | Ask AI about database architecture. | Opens Agent prompt containing database, connection, and driver. |
| `CreateTable` | Start a table DDL draft. | Creates query containing a sample `CREATE TABLE` statement and activates Query. |
| `CopyName` | Copy display name. | Writes connection name to clipboard and sets feedback message. |
| `CopyConnectionString` | Copy driver-aware connection URI. | SQLite copies database path; other drivers receive a URI using driver scheme, user, host, port, and database. |
| `Edit` | Edit saved connection. | Calls `open_edit_connection`. |
| `Duplicate` | Duplicate saved connection. | Calls `open_duplicate_connection`. |
| `Delete` | Request deletion. | Sets `overlay.delete_confirmation_id`; deletion is confirmation-driven. |

### `connection_display_uri`

Builds the copied connection string:

- SQLite → database/path string.
- MySQL-like driver → `mysql://...`.
- SQL Server/MSSQL → `sqlserver://...`.
- Otherwise → `postgresql://...`.

## 7. Database and schema tree functions/actions

### `ExplorerSchemaTreeView::draw`

1. Calls `draw_feedback` for loading/error/refresh state.
2. Calls `DatabaseNodeContext::draw`.
3. If database is open, calls `draw_schemas`.

### `draw_feedback`

Maps `ExplorerSchemaFeedbackAction::RefreshSchema` to `ExplorerSchemaTreeAction::RefreshSchema`.

### `draw_schemas`

- Clones the introspected schema list.
- Filters with `is_user_visible_schema`.
- Calls `draw_schema_node` for each visible schema.
- If no schema exists, draws schema-scoped objects using an empty schema name.

### `draw_schema_node`

- Computes active state and table count.
- Calls `SchemaNodeContext::draw`.
- Active/open schema → draws schema objects.
- Inactive/open schema → shows `Inactive schema — click to activate`.
- Row activation emits `ExplorerSchemaTreeAction::ActivateSchema(schema)`.

### `ExplorerSchemaTreeAction` mapping

Handled by `DbProApp::apply_schema_tree_action`.

| Action | Meaning | Effect |
|---|---|---|
| `RefreshSchema` | Refresh metadata for this connection. | Calls `request_schema_introspection(connection_id, true)`. |
| `ActivateSchema(schema)` | Make a schema the active navigation scope. | Calls `SchemaActivationContext::activate`; updates table/schema workspace scope. |
| `SchemaObjects(action)` | Forward table/view/function/trigger action. | Calls `apply_schema_objects_action`. |

### `DatabaseNodeContext::draw`

- Persistent id: connection id plus database name.
- Defaults open.
- Click or chevron toggles the database node.
- Empty database name is shown as `database`.

### `SchemaNodeContext::draw`

- Persistent id: connection id plus schema name.
- Active schema defaults open.
- Active row click toggles open/closed.
- Inactive row click opens and requests activation.
- Chevron click only toggles open/closed.

## 8. Schema object functions and actions

File: `explorer_schema_objects_view.rs`

### `ExplorerSchemaObjectsView::draw`

1. Normalizes the filter query to lowercase.
2. Calls `draw_tables`.
3. Draws Views.
4. Draws Functions only when `functions_enabled` is true.
5. Draws Triggers.

### `draw_tables`

- Computes total and matching table counts.
- Calls `TableFolderContext::draw_header`.
- Loads tables through `SchemaExplorerState::cached_tables`.
- Shows empty state when no table matches.
- Uses clip-rectangle checks and row spacers for offscreen rows.
- Calls `draw_table_row` for visible rows.
- Calls `draw_overflow_hint` when not all matching rows are shown.

### `matching_table_count`

Uses the explorer navigation cache when its connection/schema/search key matches. Otherwise falls back to `SchemaExplorerState::matching_table_count`.

### `draw_table_row`

- Computes selected state and whether metadata details exist.
- Calls `TableRowContext::draw`.
- Converts row selection to `ExplorerSchemaObjectsAction::SelectTable`.
- Converts table context-menu actions to `ExplorerSchemaObjectsAction::TableRow`.
- If selected and expanded, calls `TableDetailsView::draw`.

### `ExplorerSchemaObjectsAction` mapping

Handled by `apply_schema_objects_action` in `explorer_connections.rs`.

| Action | Meaning | Effect |
|---|---|---|
| `SelectTable(table)` | Select/open a table from a normal row click. | Calls `open_table(table)`. |
| `TableRow { table, action }` | Apply a table context-menu action. | Resolves active schema and calls `apply_table_row_action`. |
| `SchemaObject(action)` | Apply a view/function/trigger action. | Calls `apply_schema_object_folder_action`. |

## 9. Table actions in detail

File: `explorer_table_row_view.rs`

### `TableRowContext::draw`

- Creates persistent table-details state.
- Calls `draw_row`.
- Calls `table_context_menu`.
- A normal row click selects the table unless it is a context-menu interaction.
- A context-menu action that returns `selects_table() == true` also selects the table.
- Chevron toggles details only when table metadata exists.

### `TableRowAction::selects_table`

Returns true for actions that need the table selected first:

- `OpenData`
- `OpenStructure`
- `OpenModifyTable`
- `DropTable`
- `OpenQuery`
- `GenerateInsert`
- `GenerateUpdate`
- `GenerateDelete`
- `OpenDdl`
- `AskAgent`

Returns false for `CopyName`, `CopyQualifiedName`, and `RefreshSchema`.

### `TableMenu` functions

- `add_view_actions`: View Data, View Structure.
- `add_ddl_workbench_actions`: Modify Table, Drop Table.
- `add_sql_actions`: SELECT, INSERT, UPDATE, DELETE, and DDL generation.
- `add_copy_actions`: qualified name, name, and Agent action.
- `add_refresh_action`: Refresh Schema / `F5`.
- `add_item`: common context-menu item-to-action mapper.

### `DbProApp::apply_table_row_action`

File: `explorer_details.rs`

| Action | Effect |
|---|---|
| `OpenData` | Set `TableView::Data`; activate `WorkspaceTab::Table`. |
| `OpenStructure` | Set `TableView::Structure`; activate `WorkspaceTab::Table`. |
| `OpenModifyTable` | Populate table Schema Workbench from metadata; activate `Activity::Schema` and `WorkspaceTab::SchemaWorkbench`. |
| `DropTable` | Configure table drop operation, request confirmation, and open Schema Workbench. |
| `OpenDdl` | Set `TableView::Ddl`; activate Table workspace. |
| `OpenQuery` | Open `SELECT * FROM schema.table LIMIT 100;`. |
| `GenerateInsert` | Build an INSERT template using known columns or fallback placeholders; open Query. |
| `GenerateUpdate` | Build an UPDATE template using non-PK columns and PK predicates; open Query. |
| `GenerateDelete` | Build a DELETE template using PK predicates; open Query. |
| `CopyQualifiedName` | Copy `schema.table` and show feedback. |
| `CopyName` | Copy table name and show feedback. |
| `AskAgent` | Open Agent prompt asking for table explanation and useful queries. |
| `RefreshSchema` | Re-introspect the active connection schema. |

Helper functions:

- `build_insert_query` uses metadata columns when available, otherwise `column1, column2` and placeholder values.
- `build_update_query` excludes primary keys from `SET` and uses primary keys in `WHERE`; falls back to `column_name = DEFAULT` and `id = 1`.
- `build_delete_query` uses primary keys in `WHERE`; falls back to `id = 1`.
- `open_table_view` changes table view and activates Table workspace.
- `open_query_document` sets active SQL and activates Query workspace.

## 10. Table detail functions

File: `explorer_table_details_view.rs`

### `TableDetailsView::draw`

Calls:

- `draw_columns`.
- `draw_foreign_keys`.
- `draw_indexes`.

### `draw_columns`

Renders each column with:

- column name;
- shortened data type;
- primary-key and foreign-key-aware icon/color;
- non-expandable row.

### `draw_foreign_keys`

Renders each foreign key with its name and referenced table. Empty state is `No foreign keys`.

### `draw_indexes`

Renders each index name. Empty state is `No indexes`.

## 11. Views, functions, and triggers actions

File: `explorer_schema_object_folders_view.rs`

### Folder functions

- `draw_views`: filters views by schema, renders Views category, count, empty state, and view rows.
- `draw_functions`: filters functions by schema, renders Functions category, count, empty state, and routine rows.
- `draw_triggers`: filters triggers by schema, renders Triggers category, count, empty state, and trigger rows.
- `draw_view_row`: maps a view row action to `SchemaObjectFolderAction`.
- `draw_function_row`: formats routine label and maps it through the common row mapper.
- `draw_trigger_row`: formats trigger label and maps supported trigger actions.
- `draw_row`: common mapper for function-like schema objects.

### `SchemaObjectRowContext::draw`

File: `explorer_schema_object_row_view.rs`

- Paints a non-expandable object row.
- Opens context menu.
- Normal click emits `SchemaObjectRowAction::Open`.
- Context menu uses configured labels and optional actions.

### `SchemaObjectRowAction`

| Action | Meaning |
|---|---|
| `Open` | Open the object in the schema-object workspace. |
| `OpenQuery` | Generate/open an object-specific SQL query. |
| `Modify` | Open a modify workbench when supported. |
| `Drop` | Plan a drop operation when supported. |
| `CopyName` | Copy the object name. |

### View mapping

A view row configures:

- `Open` → `SchemaObjectFolderAction::Open` with `SchemaObjectSelection::View`.
- `OpenQuery` → `SELECT * FROM schema.view LIMIT 100;`.
- `Modify` → `ModifyView(UiViewSummary)`.
- `Drop` → `DropObject { kind: "VIEW" }`.
- `CopyName` → copy view name.

### Function/procedure mapping

A function row configures:

- `Open` → open routine with `SchemaObjectSelection::Function { name, identity_arguments }`.
- `OpenQuery` → `SELECT * FROM schema.function();`.
- `CopyName` → copy routine name.
- Modify and drop are not offered by the current row configuration.

The label includes routine type and identity arguments when present. Procedures use a branch icon; functions use a code icon.

### Trigger mapping

A trigger row configures:

- `Open` → open trigger with `SchemaObjectSelection::Trigger`.
- `Drop` → `DropObject { kind: "TRIGGER" }`.
- `CopyName` → copy trigger name.
- Query and modify are not offered.

### `SchemaObjectFolderAction` consumption

Handled by `DbProApp::apply_schema_object_folder_action` in `explorer_folders.rs`.

| Action | Effect |
|---|---|
| `Open(request)` | Calls `open_schema_object`. |
| `OpenQuery(query)` | Sets active SQL and activates Query workspace. |
| `ModifyView(view)` | Loads view definition into View Schema Workbench. |
| `DropObject { schema, name, kind }` | Selects View/Trigger/Table workbench mode, plans `ObjectAction::Drop`, requests confirmation, and opens Schema Workbench. |
| `CopyName(name)` | Copies name and shows feedback. |

### `open_schema_object`

Creates `SchemaObjectActivationContext` and calls its `open` method. The activation:

- records selected schema object;
- switches object view to Definition;
- clears selected table and resets table workspace;
- activates `WorkspaceTab::SchemaObject`;
- synchronizes routine state for functions;
- resets routine confirmation/DDL preview;
- emits an opened-object feedback message.

## 12. Connection lifecycle and guard functions

File: `explorer_navigation.rs`

### `connect_to_connection`

- No-ops if the same connection is already connected.
- Builds `ConnectRequest` through `ExplorerConnectionContext::connect`.
- Dispatches the runtime connect command.
- Commits pending connection state only if dispatch succeeds.

### `disconnect_from_connection`

Delegates to `ExplorerConnectionContext::disconnect`.

Guards:

- Open transaction → sets disconnect transaction guard and refuses disconnect.
- Otherwise clears connected state, schema scope, and table workspace.

### `ExplorerConnectionContext::connect`

Refuses to create a connect request when:

- the requested connection is already active and connected;
- staged table changes require a pending change-connection confirmation;
- an open transaction requires commit/rollback first.

Otherwise returns `ConnectRequest`.

### `commit_connect`

After runtime command dispatch:

- clears pending navigation;
- clears Agent input;
- marks active connection and pending request;
- resets schema/table workspace scope;
- sets pending operation to connect;
- reports `Connecting to ...`.

## 13. Workspace and runtime effects summary

| Explorer intent | Destination/effect |
|---|---|
| Connect/disconnect/reconnect | Connection lifecycle and runtime command bridge. |
| Refresh schema | Schema introspection request. |
| Select table | Table workspace and table metadata scope. |
| View data/structure/DDL | Table workspace with selected view. |
| Generate SQL | Query workspace with generated SQL text. |
| Open view/function/trigger | Schema Object workspace. |
| Modify/drop object | Schema Workbench, often with confirmation state. |
| Open ER diagram | Diagram workspace. |
| Ask Agent | Agent prompt surface. |
| Copy actions | egui clipboard output plus feedback message. |
| Delete connection | Delete confirmation overlay. |

## 14. Current capability inventory

The Explorer currently provides:

- connection catalog navigation;
- connection lifecycle controls;
- database/schema/table hierarchy;
- schema filtering and refresh;
- table data, structure, DDL, and SQL generation entry points;
- table columns, foreign keys, and indexes details;
- views, functions/procedures, and triggers folders;
- schema object open/modify/drop/copy actions where configured;
- ER diagram, SQL editor, Schema Workbench, Agent, and clipboard integrations;
- connection, transaction, staged-change, and delete-confirmation guardrails;
- function-folder capability gating;
- offscreen table-row rendering and navigation-cache-aware filtering.

## 15. Evidence boundary

This is a source inventory. It describes functions, action variants, and reducer mappings visible in the code. It does not prove runtime behavior against PostgreSQL or SQLite and does not replace UI runtime evidence.

For a later compare-feature document, compare each action variant and its effect separately. A renamed menu label, a new action, a removed action, a changed workspace destination, or a changed guard should be recorded as a distinct difference.
