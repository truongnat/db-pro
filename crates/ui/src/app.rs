use crate::components::*;
use crate::tokens::*;
use crate::{
    activity_bar_frame, agent_message_frame, badge, card_frame, compact_button, compact_button_with_icon,
    compact_icon_button, compact_icon_button_enabled, danger_button, editor_frame, empty_state, ghost_button_with_icon,
    grid_frame, icon_button, icon_text, input, input_full_width, menu_button_with_icon, panel_frame,
    primary_button_with_icon, secondary_button_with_icon, section_label, sidebar_frame, sidebar_item, tab_frame,
    toolbar_frame, AgentContext, AgentMessage, AgentProvider, AgentRole, DbProTheme, OfflineAgentProvider, TaskBridge,
    UiCell, UiCommand, UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary,
    UiQueryFolderSummary, UiQueryResult, UiSavedQuerySummary, UiSchemaForeignKey, UiSchemaSummary, UiSslMode,
    UiTableDataFilter, UiTableDataSort, UiTableFilterOperator, UiTableInfo, UiTableSummary, UiTriggerSummary,
    UiViewSummary,
};
use bigdecimal::BigDecimal;
use db_pro_core::domain::capabilities::DatabaseCapabilities;
use db_pro_core::domain::connection::DriverType;
use eframe::egui::text::LayoutJob;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, TextEdit, TextFormat, TopBottomPanel};
use lucide_icons::Icon;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Duration;

#[path = "agent_state.rs"]
mod agent_state;
#[path = "agent_view.rs"]
mod agent_view;
#[path = "app_state.rs"]
mod app_state;
#[path = "component_gallery_view.rs"]
mod component_gallery_view;
#[path = "connection_view.rs"]
mod connection_view;

pub use component_gallery_view::ComponentGalleryState;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "events.rs"]
mod events;
#[path = "explorer_details.rs"]
mod explorer_details;
#[path = "explorer_folders.rs"]
mod explorer_folders;
#[path = "explorer_tree.rs"]
mod explorer_tree;
#[path = "explorer_view.rs"]
mod explorer_view;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "query_view.rs"]
mod query_view;
#[path = "result_grid_view.rs"]
mod result_grid_view;
#[path = "schema_object_view.rs"]
mod schema_object_view;
#[path = "table_ddl_view.rs"]
mod table_ddl_view;
#[path = "table_editor_view.rs"]
mod table_editor_view;
#[path = "table_metadata_view.rs"]
mod table_metadata_view;
#[path = "table_view.rs"]
mod table_view;
#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
#[path = "workspace_view.rs"]
mod workspace_view;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct QueryDocument {
    title: String,
    content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Activity {
    Explorer,
    Queries,
    History,
    Transfers,
    Monitor,
    Settings,
    Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteMode {
    QuickOpen,
    Commands,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaletteAction {
    Welcome,
    Query,
    History,
    Settings,
    Diagram,
    Agent,
    NewQuery,
    NewConnection,
    RefreshSchema,
    ToggleExplorer,
    OpenTable(String),
    ExplainQuery,
    ExportResults,
    RunQuery,
    FormatSql,
    SwitchConnection(String),
    ComponentGallery,
}

#[derive(Debug, Clone)]
pub(crate) struct PaletteItem {
    icon: Icon,
    title: String,
    subtitle: String,
    shortcut: Option<String>,
    action: PaletteAction,
}

const AGENT_SIDEBAR_COLLAPSE_WIDTH: f32 = 1180.0;
const SIDEBAR_MIN_WIDTH: f32 = 220.0;
const SIDEBAR_MAX_WIDTH: f32 = 380.0;
const AGENT_MIN_WIDTH: f32 = 300.0;
const AGENT_MAX_WIDTH: f32 = 480.0;
const OUTPUT_MIN_HEIGHT: f32 = 120.0;
const OUTPUT_MAX_HEIGHT: f32 = 420.0;
const TABLE_PAGE_SIZE: u64 = 100;
const GRID_ROW_NUMBER_WIDTH: f32 = 48.0;
const ER_MAX_TABLES: usize = 5;
const ER_MAX_COLUMNS: usize = 8;
const ER_MAX_EDGES: usize = 6;
const ER_LARGE_SCHEMA_THRESHOLD: usize = 200;
const EXPLORER_MAX_TABLES: usize = 100;
const ER_NODE_WIDTH: f32 = 280.0;
const ER_HEADER_HEIGHT: f32 = 40.0;
const ER_ROW_HEIGHT: f32 = 24.0;
const ER_GAP_X: f32 = 84.0;
const ER_GAP_Y: f32 = 76.0;
const ER_CANVAS_MARGIN: f32 = 48.0;

#[derive(Debug, Clone)]
struct ErNode {
    table: UiTableSummary,
    rect: egui::Rect,
}

fn er_column_anchor(node: &ErNode, column: Option<&str>, right_side: bool, zoom: f32) -> egui::Pos2 {
    let column_index = column
        .and_then(|name| node.table.columns.iter().position(|item| item.name == name))
        .unwrap_or(0);
    let y = if node.table.columns.is_empty() {
        node.rect.center().y
    } else {
        node.rect.top() + ER_HEADER_HEIGHT * zoom + ER_ROW_HEIGHT * zoom * (column_index as f32 + 0.5)
    };
    egui::pos2(
        if right_side {
            node.rect.right()
        } else {
            node.rect.left()
        },
        y,
    )
}

fn matches_diagram_search(table: &UiTableSummary, query: &str) -> bool {
    table.name.to_ascii_lowercase().contains(query)
        || table.schema.to_ascii_lowercase().contains(query)
        || table
            .columns
            .iter()
            .any(|column| column.name.to_ascii_lowercase().contains(query))
}

fn ddl_impact_summary(sql: &str, target: &str) -> String {
    let verb = sql.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    match verb.as_str() {
        "drop" => format!("Drop {target}: removes the database object; recovery requires a backup."),
        "truncate" => format!("Truncate {target}: removes its rows; recovery requires a backup."),
        "alter" => format!("Alter {target}: changes its structure and may invalidate dependent queries."),
        "create" => format!("Create {target}: adds or rebuilds a database object."),
        _ => format!("This SQL changes {target}; review the preview and keep a backup before applying."),
    }
}

fn matches_explorer_table(table: &str, query: &str) -> bool {
    query.is_empty() || table.to_ascii_lowercase().contains(query)
}

fn filtered_explorer_tables(tables: &[String], query: &str) -> (usize, Vec<String>) {
    let matching_count = tables
        .iter()
        .filter(|table| matches_explorer_table(table, query))
        .count();
    let visible_tables = tables
        .iter()
        .filter(|table| matches_explorer_table(table, query))
        .take(EXPLORER_MAX_TABLES)
        .cloned()
        .collect();
    (matching_count, visible_tables)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceTab {
    Welcome,
    Query,
    Table,
    SchemaObject,
    Diagram,
    ComponentGallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableView {
    Structure,
    Data,
    Indexes,
    Relations,
    Constraints,
    Dependencies,
    Ddl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputTab {
    Results,
    Messages,
    Explain,
    History,
}

#[derive(Debug, Clone, PartialEq)]
enum StagedChange {
    Update {
        row_index: usize,
        column_index: usize,
        column: String,
        original: UiCell,
        value: UiCell,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Delete {
        row_index: usize,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SchemaObjectSelection {
    View(String),
    Trigger(String),
    Function(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaObjectView {
    Definition,
    Data,
}

/// First native vertical slice: visual shell, navigation, workspace tabs and
/// a functional query surface. The task bridge is backend-agnostic so the
/// same UI can run with the native runtime adapter or in an isolated preview.
pub struct DbProApp {
    theme: DbProTheme,
    dark_mode: bool,
    reduce_motion: bool,
    activity: Activity,
    welcome_open: bool,
    active_tab: WorkspaceTab,
    sidebar_open: bool,
    sidebar_width: f32,
    agent_open: bool,
    agent_width: f32,
    bottom_panel_open: bool,
    bottom_panel_height: f32,
    sidebar_open_before_agent: Option<bool>,
    query_text: String,
    welcome_prompt: String,
    selected_query: String,
    query_documents: Vec<QueryDocument>,
    active_query_document: usize,
    editor_search: String,
    editor_search_open: bool,
    query_editor_focused: bool,
    query_cursor_line: usize,
    query_cursor_column: usize,
    editor_font_size: f32,
    query_tools_open: bool,
    completion_open: bool,
    snippets_open: bool,
    diagnostics: Vec<String>,
    query_history: Vec<String>,
    connection_name: String,
    connected: bool,
    palette_mode: Option<PaletteMode>,
    palette_query: String,
    palette_selected: usize,
    palette_focus_requested: bool,
    agent_provider: Box<dyn AgentProvider>,
    agent_request: Option<crate::RequestId>,
    agent_pending_prompt: Option<String>,
    agent_pending_context: Option<AgentContext>,
    agent_provider_label: String,
    agent_provider_detail: String,
    agent_input: String,
    agent_messages: Vec<AgentMessage>,
    agent_settings_open: bool,
    agent_api_key_draft: String,
    agent_configure_request: Option<crate::RequestId>,
    task_bridge: TaskBridge,
    next_query_request: Option<crate::RequestId>,
    runtime_message: String,
    toasts: crate::components::overlay::ToastManager,
    query_result: Option<UiQueryResult>,
    output_tab: OutputTab,
    query_messages: Vec<String>,
    explain_plan: Option<String>,
    explain_request: Option<crate::RequestId>,
    grid_filter: String,
    grid_sort_column: Option<usize>,
    grid_sort_desc: bool,
    grid_column_widths: Vec<f32>,
    grid_column_order: Vec<usize>,
    grid_columns_user_resized: bool,
    selected_cell: Option<(usize, usize)>,
    selected_row: Option<usize>,
    selected_rows: BTreeSet<usize>,
    selection_anchor_row: Option<usize>,
    selection_anchor_cell: Option<(usize, usize)>,
    data_editing_cell: Option<(usize, usize)>,
    data_edit_value: String,
    data_delete_confirmation: bool,
    insert_row_open: bool,
    insert_row_values: Vec<String>,
    insert_row_error: String,
    copy_status: String,
    export_open: bool,
    export_format: String,
    export_path: String,
    connections: Vec<UiConnectionSummary>,
    saved_queries: Vec<UiSavedQuerySummary>,
    query_folders: Vec<UiQueryFolderSummary>,
    schema: UiSchemaSummary,
    selected_schema: Option<String>,
    explorer_search: String,
    schema_error: Option<String>,
    schema_request: Option<crate::RequestId>,
    selected_table: Option<String>,
    selected_schema_object: Option<SchemaObjectSelection>,
    schema_object_view: SchemaObjectView,
    diagram_zoom: f32,
    diagram_pan: egui::Vec2,
    diagram_pan_origin: Option<egui::Vec2>,
    diagram_search: String,
    diagram_show_all: bool,
    table_info: Option<UiTableInfo>,
    table_ddl: Option<String>,
    table_info_error: Option<String>,
    table_ddl_error: Option<String>,
    ddl_execute_confirmation: bool,
    ddl_execution_request: Option<crate::RequestId>,
    refresh_table_info_after_schema: bool,
    table_data_result: Option<UiQueryResult>,
    table_data_total_rows: Option<u64>,
    table_data_offset: u64,
    table_data_limit: u64,
    table_data_filter_column: String,
    table_data_filter_operator: UiTableFilterOperator,
    table_data_filter_value: String,
    table_data_sort_column: Option<String>,
    table_data_sort_desc: bool,
    table_data_error: Option<String>,
    table_structure_search: String,
    table_metadata_search: String,
    table_dependency_filter: String,
    table_constraint_filter: String,
    table_info_request: Option<crate::RequestId>,
    table_ddl_request: Option<crate::RequestId>,
    table_data_request: Option<crate::RequestId>,
    table_mutation_request: Option<crate::RequestId>,
    staged_changes: Vec<StagedChange>,
    staged_apply_request: Option<crate::RequestId>,
    table_view: TableView,
    query_folder: String,
    backup_output_path: String,
    restore_input_path: String,
    restore_confirmation: bool,
    active_connection_id: Option<String>,
    pending_connection_id: Option<String>,
    pending_connection_request: Option<crate::RequestId>,
    connection_errors: std::collections::HashMap<String, String>,
    failed_connection_ids: std::collections::HashSet<String>,
    connections_requested: bool,
    connections_request_pending: bool,
    connection_dialog_open: bool,
    editing_connection_id: Option<String>,
    connection_draft: UiConnectionDraft,
    connection_show_password: bool,
    connection_error: String,
    connection_test_valid: bool,
    connection_test_draft: Option<UiConnectionDraft>,
    delete_confirmation_id: Option<String>,
    folder_delete_confirmation: Option<String>,
    /// Persisted height of the Connections sub-pane inside the Explorer sidebar.
    connections_pane_height: f32,
    /// Persisted height of the Schemas sub-pane inside the Explorer sidebar.
    schemas_pane_height: f32,
    /// Counter for initial render frames to ensure window is maximized on startup.
    initial_frames_count: u8,
    pub gallery_state: component_gallery_view::ComponentGalleryState,
}

impl eframe::App for DbProApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.surface_app.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(widths) = serde_json::to_string(&self.grid_column_widths) {
            storage.set_string("dbpro.native.grid-widths", widths);
        }
        storage.set_string(
            "dbpro.native.grid-widths-customized",
            self.grid_columns_user_resized.to_string(),
        );
        self.persist_active_query_document();
        if let Ok(documents) = serde_json::to_string(&self.query_documents) {
            storage.set_string("dbpro.native.query-documents", documents);
        }
        storage.set_string("dbpro.native.theme-version", "light-first-v1".to_owned());
        storage.set_string("dbpro.native.dark-mode", self.dark_mode.to_string());
        storage.set_string("dbpro.native.reduce-motion", self.reduce_motion.to_string());
        storage.set_string("dbpro.native.sidebar-width", self.sidebar_width.to_string());
        storage.set_string("dbpro.native.agent-width", self.agent_width.to_string());
        storage.set_string("dbpro.native.output-open", self.bottom_panel_open.to_string());
        storage.set_string("dbpro.native.output-height", self.bottom_panel_height.to_string());
        storage.set_string(
            "dbpro.native.connections-pane-height",
            self.connections_pane_height.to_string(),
        );
        storage.set_string("dbpro.native.schemas-pane-height", self.schemas_pane_height.to_string());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Map Ctrl to Command in input events so Ctrl+A/C/V/X/Z work seamlessly on macOS
        ctx.input_mut(|i| {
            if i.modifiers.ctrl {
                i.modifiers.command = true;
            }
            for event in &mut i.events {
                if let egui::Event::Key { modifiers, .. } = event {
                    if modifiers.ctrl {
                        modifiers.command = true;
                    }
                }
            }
        });

        if self.initial_frames_count < 3 {
            self.initial_frames_count += 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
        self.request_connections_once();
        self.apply_runtime_events();
        if self.runtime_work_pending() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.theme = if self.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
        self.theme.apply(ctx);
        self.handle_shortcuts(ctx);
        self.draw_topbar(ctx);
        self.draw_output_panel(ctx);
        self.draw_statusbar(ctx);
        self.draw_activity_bar(ctx);

        if self.sidebar_open {
            self.draw_sidebar(ctx);
        }

        if self.agent_open {
            self.draw_agent_panel(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.surface_app))
            .show(ctx, |ui| {
                self.draw_workspace(ui);
            });

        if self.connection_dialog_open {
            self.draw_connection_dialog(ctx);
        }
        if self.delete_confirmation_id.is_some() {
            self.draw_delete_confirmation(ctx);
        }
        if self.folder_delete_confirmation.is_some() {
            self.draw_folder_delete_confirmation(ctx);
        }
        if self.insert_row_open {
            self.draw_insert_row_dialog(ctx);
        }
        if self.palette_mode.is_some() {
            self.draw_palette(ctx);
        }

        self.toasts.render_ctx(ctx, self.theme);
        if !self.toasts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

impl DbProApp {
    pub(crate) fn show_toast_error(&mut self, message: impl Into<String>) {
        self.toasts
            .error(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    pub(crate) fn show_toast_success(&mut self, message: impl Into<String>) {
        self.toasts
            .success(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    #[allow(dead_code)]
    pub(crate) fn show_toast_info(&mut self, message: impl Into<String>) {
        self.toasts
            .info(message, crate::components::overlay::ToastPosition::BottomRight);
    }

    pub(super) fn primary_modifier_pressed(input: &egui::InputState) -> bool {
        input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd
    }

    fn primary_modifier_label() -> &'static str {
        if cfg!(target_os = "macos") {
            "⌘"
        } else {
            "Ctrl"
        }
    }

    /// Queues a command for the runtime worker, ignoring transport failures.
    ///
    /// The UI is fire-and-forget: a send only fails once the worker channel is
    /// closed (shutdown), and a frame that already drew its widgets has nothing
    /// actionable to do about it. Runtime-side problems are reported back
    /// through `UiEvent`, not through this return value.
    fn dispatch_command(&mut self, command: UiCommand) {
        // Intentionally ignored — see the method contract above.
        let _ = self.task_bridge.send(command);
    }

    fn active_connection(&self) -> Option<&UiConnectionSummary> {
        self.connections
            .iter()
            .find(|connection| Some(connection.id.as_str()) == self.active_connection_id.as_deref())
    }

    fn active_connection_name(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.name.as_str())
            .unwrap_or(self.connection_name.as_str())
    }

    fn active_driver(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.driver.as_str())
            .unwrap_or("PostgreSQL")
    }

    pub(crate) fn active_capabilities(&self) -> Option<DatabaseCapabilities> {
        let driver = self.active_connection()?.driver.as_str();
        let driver = if driver.eq_ignore_ascii_case("sqlite") {
            DriverType::SQLite
        } else if driver.eq_ignore_ascii_case("postgresql") || driver.eq_ignore_ascii_case("postgres") {
            DriverType::Postgres
        } else {
            return None;
        };
        Some(DatabaseCapabilities::for_driver(driver))
    }

    fn active_schema(&self) -> &str {
        self.selected_schema
            .as_deref()
            .or_else(|| self.schema.schemas.first().map(String::as_str))
            .unwrap_or_else(|| {
                if self.active_driver().eq_ignore_ascii_case("sqlite") {
                    "main"
                } else {
                    "public"
                }
            })
    }

    fn active_schema_table_names(&self) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.tables.clone();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == self.active_schema())
            .map(|table| table.name.clone())
            .collect()
    }

    fn active_schema_column_names(&self) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.columns.clone();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == self.active_schema())
            .flat_map(|table| table.columns.iter().map(|column| column.name.clone()))
            .collect()
    }

    fn has_runtime_error(&self) -> bool {
        self.runtime_message.contains("failed")
            || self.runtime_message.contains("Failed")
            || self.runtime_message.contains("error")
            || self.runtime_message.contains("Error")
    }

    fn statusbar_state(&self) -> (Icon, Color32, &'static str) {
        if self.connected && self.active_connection_id.is_some() {
            return (Icon::CircleCheck, self.theme.success, "Connected");
        }
        if self.runtime_message.starts_with("Connecting") {
            return (Icon::Circle, self.theme.accent, "Connecting…");
        }
        if self.has_runtime_error() {
            return (Icon::TriangleAlert, self.theme.danger, "Runtime error");
        }
        (Icon::Circle, self.theme.warning, "Not connected")
    }

    pub(super) fn shows_editor_status(&self) -> bool {
        self.active_tab == WorkspaceTab::Query
    }

    pub(super) fn statusbar_context_label(&self) -> &'static str {
        match self.active_tab {
            WorkspaceTab::Welcome => "Workspace",
            WorkspaceTab::Query => "SQL Editor",
            WorkspaceTab::Table => match self.table_view {
                TableView::Structure => "Table Structure",
                TableView::Data => "Data Editor",
                TableView::Indexes => "Table Indexes",
                TableView::Relations => "Table Relations",
                TableView::Constraints => "Table Constraints",
                TableView::Dependencies => "Table Dependencies",
                TableView::Ddl => "Table DDL",
            },
            WorkspaceTab::SchemaObject => "Schema Object",
            WorkspaceTab::Diagram => "ER Diagram",
            WorkspaceTab::ComponentGallery => "Component Gallery",
        }
    }

    fn connection_indicator(&self, connection: &UiConnectionSummary) -> (Icon, Color32) {
        let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
        let is_connected = is_active && self.connected;
        let is_failed = self.failed_connection_ids.contains(&connection.id);
        let icon = if is_connected {
            Icon::CircleCheck
        } else if is_failed {
            Icon::AlertCircle
        } else {
            Icon::Circle
        };
        let color = if is_connected && connection.readonly {
            self.theme.warning
        } else if is_connected {
            self.theme.success
        } else if is_failed {
            self.theme.danger
        } else if is_active {
            self.theme.accent
        } else {
            self.theme.text_muted
        };
        (icon, color)
    }

    fn persist_active_query_document(&mut self) {
        if let Some(document) = self.query_documents.get_mut(self.active_query_document) {
            document.content = self.query_text.clone();
        }
    }

    pub(crate) fn switch_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() || index == self.active_query_document {
            return;
        }
        self.persist_active_query_document();
        self.active_query_document = index;
        self.query_text = self.query_documents[index].content.clone();
        self.reset_query_cursor();
        self.query_result = None;
        self.runtime_message = format!("Opened {}", self.query_documents[index].title);
    }

    pub(crate) fn new_query_document(&mut self) {
        self.persist_active_query_document();
        let index = self.query_documents.len() + 1;
        self.query_documents.push(QueryDocument {
            title: format!("Query {index}"),
            content: String::new(),
        });
        self.active_query_document = self.query_documents.len() - 1;
        self.query_text.clear();
        self.reset_query_cursor();
        self.query_result = None;
        self.activity = Activity::Queries;
        self.sidebar_open = true;
        self.active_tab = WorkspaceTab::Query;
    }

    pub(crate) fn close_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }

        self.persist_active_query_document();
        let closed_title = self.query_documents[index].title.clone();
        self.query_documents.remove(index);

        if self.query_documents.is_empty() {
            self.active_query_document = 0;
            self.query_text.clear();
            self.reset_query_cursor();
            self.query_result = None;
            if self.active_tab == WorkspaceTab::Query {
                self.activate_fallback_workspace_tab();
            }
            self.runtime_message = format!("Closed {closed_title}");
            return;
        }

        if self.active_query_document > index {
            self.active_query_document -= 1;
        } else if self.active_query_document == index {
            self.active_query_document = self.active_query_document.min(self.query_documents.len() - 1);
        }
        self.query_text = self.query_documents[self.active_query_document].content.clone();
        self.reset_query_cursor();
        self.query_result = None;
        self.runtime_message = format!("Closed {}", self.query_documents[self.active_query_document].title);
    }

    fn reset_query_cursor(&mut self) {
        self.query_cursor_line = 1;
        self.query_cursor_column = 1;
    }

    pub(crate) fn duplicate_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }
        self.persist_active_query_document();
        let src = &self.query_documents[index];
        let title = format!("{} (Copy)", src.title);
        let content = src.content.clone();
        self.query_documents.push(QueryDocument { title, content });
        self.active_query_document = self.query_documents.len() - 1;
        self.query_text = self.query_documents[self.active_query_document].content.clone();
        self.query_result = None;
        self.active_tab = WorkspaceTab::Query;
        self.runtime_message = format!("Duplicated {}", self.query_documents[index].title);
    }

    pub(crate) fn close_other_query_documents(&mut self, keep_index: usize) {
        if keep_index >= self.query_documents.len() {
            return;
        }
        self.persist_active_query_document();
        let kept = self.query_documents[keep_index].clone();
        self.query_documents = vec![kept];
        self.active_query_document = 0;
        self.query_text = self.query_documents[0].content.clone();
        self.query_result = None;
        self.runtime_message = "Closed other queries".to_owned();
    }

    pub(crate) fn close_query_documents_to_right(&mut self, index: usize) {
        if index >= self.query_documents.len() {
            return;
        }
        self.persist_active_query_document();
        self.query_documents.truncate(index + 1);
        if self.active_query_document > index {
            self.active_query_document = index;
            self.query_text = self.query_documents[index].content.clone();
            self.query_result = None;
        }
        self.runtime_message = "Closed queries to the right".to_owned();
    }

    pub(crate) fn close_all_tabs(&mut self) {
        self.persist_active_query_document();
        self.welcome_open = true;
        self.query_documents = vec![QueryDocument {
            title: "Query 1".to_string(),
            content: String::new(),
        }];
        self.active_query_document = 0;
        self.query_text.clear();
        self.query_result = None;
        self.selected_table = None;
        self.selected_schema_object = None;
        self.active_tab = WorkspaceTab::Welcome;
        self.runtime_message = "Closed all tabs".to_owned();
    }

    pub(crate) fn close_welcome_tab(&mut self) {
        self.welcome_open = false;
        if self.active_tab == WorkspaceTab::Welcome {
            self.activate_fallback_workspace_tab();
        }
        self.runtime_message = "Closed Welcome".to_owned();
    }

    fn activate_welcome_tab(&mut self) {
        self.welcome_open = true;
        self.active_tab = WorkspaceTab::Welcome;
    }

    fn activate_fallback_workspace_tab(&mut self) {
        if !self.query_documents.is_empty() {
            self.active_tab = WorkspaceTab::Query;
        } else if self.selected_table.is_some() {
            self.active_tab = WorkspaceTab::Table;
        } else if self.selected_schema_object.is_some() {
            self.active_tab = WorkspaceTab::SchemaObject;
        } else {
            self.activate_welcome_tab();
        }
    }

    pub(crate) fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
        match tab {
            WorkspaceTab::Table => {
                self.selected_table = None;
                self.table_info = None;
                self.table_ddl = None;
                self.table_info_error = None;
                self.table_ddl_error = None;
                self.table_data_result = None;
                self.table_data_total_rows = None;
                self.table_data_request = None;
                self.table_info_request = None;
                self.table_ddl_request = None;
                self.table_mutation_request = None;
                self.staged_changes.clear();
                self.staged_apply_request = None;
                self.selected_cell = None;
                self.selected_row = None;
                self.selected_rows.clear();
                self.selection_anchor_row = None;
                self.selection_anchor_cell = None;
                self.data_editing_cell = None;
                self.data_delete_confirmation = false;
            }
            WorkspaceTab::SchemaObject => {
                self.selected_schema_object = None;
                self.schema_object_view = SchemaObjectView::Definition;
                self.table_data_result = None;
                self.table_data_total_rows = None;
                self.table_data_request = None;
            }
            WorkspaceTab::Diagram => {
                self.diagram_search.clear();
                self.diagram_show_all = false;
                self.diagram_pan = egui::Vec2::ZERO;
                self.diagram_pan_origin = None;
            }
            WorkspaceTab::ComponentGallery => {}
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.active_tab == tab {
            self.activate_welcome_tab();
        }
        self.runtime_message = "Workspace closed".to_owned();
    }

    fn open_palette(&mut self, mode: PaletteMode) {
        self.palette_mode = Some(mode);
        self.palette_query.clear();
        self.palette_selected = 0;
        self.palette_focus_requested = true;
    }

    fn open_new_connection(&mut self) {
        self.editing_connection_id = None;
        self.connection_draft = UiConnectionDraft::default();
        self.connection_error.clear();
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.pending_connection_request = None;
        self.connection_dialog_open = true;
    }

    pub(crate) fn set_agent_open(&mut self, open: bool, ctx: &egui::Context) {
        if open == self.agent_open {
            return;
        }
        self.agent_open = open;
        if open {
            self.sidebar_open_before_agent = Some(self.sidebar_open);
            if ctx.screen_rect().width() < AGENT_SIDEBAR_COLLAPSE_WIDTH {
                self.sidebar_open = false;
            }
        } else if let Some(sidebar_open) = self.sidebar_open_before_agent.take() {
            self.sidebar_open = sidebar_open;
        }
    }

    pub(crate) fn open_agent_prompt(&mut self, prompt: impl Into<String>, ctx: &egui::Context) {
        self.agent_input = prompt.into();
        self.set_agent_open(true, ctx);
    }

    fn insert_agent_sql(&mut self, sql: &str) {
        self.query_text = sql.to_owned();
        self.persist_active_query_document();
        self.active_tab = WorkspaceTab::Query;
        self.runtime_message = "Inserted Agent draft into Query".to_owned();
    }

    fn run_agent_read_only(&mut self, sql: &str) {
        if !self.connected || self.active_connection_id.is_none() || self.next_query_request.is_some() {
            self.runtime_message = "Connect to a database before running the Agent draft".to_owned();
            return;
        }
        self.query_text = sql.to_owned();
        self.selected_query.clear();
        self.persist_active_query_document();
        self.active_tab = WorkspaceTab::Query;
        self.dispatch_query();
    }

    fn request_connections_once(&mut self) {
        if self.connections_requested {
            return;
        }
        self.connections_requested = true;
        self.connections_request_pending = true;
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListConnections { request_id });
    }

    fn runtime_work_pending(&self) -> bool {
        self.connections_request_pending
            || self.pending_connection_request.is_some()
            || self.schema_request.is_some()
            || self.next_query_request.is_some()
            || self.explain_request.is_some()
            || self.agent_request.is_some()
            || self.table_info_request.is_some()
            || self.table_ddl_request.is_some()
            || self.table_data_request.is_some()
            || self.table_mutation_request.is_some()
            || self.staged_apply_request.is_some()
            || self.ddl_execution_request.is_some()
    }

    fn request_schema_introspection(&mut self, connection_id: String, force_refresh: bool) {
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::IntrospectSchema {
            request_id,
            connection_id,
            force_refresh,
        });
        self.schema_request = Some(request_id);
        self.schema_error = None;
        self.runtime_message = if force_refresh {
            "Refreshing schema…"
        } else {
            "Loading schema…"
        }
        .to_owned();
    }
}
