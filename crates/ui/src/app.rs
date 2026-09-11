use crate::{
    activity_bar_frame, badge, card_frame, compact_button, compact_button_enabled, compact_button_with_icon,
    compact_icon_button, compact_icon_button_enabled, danger_button, editor_frame, ghost_button,
    ghost_button_with_icon, icon_button, icon_text, input, input_full_width, panel_frame, password_input,
    primary_button, primary_button_with_icon, secondary_button_with_icon, section_label, sidebar_frame, sidebar_item,
    tab_frame, toolbar_frame, AgentContext, AgentMessage, AgentProvider, AgentRole, DbProTheme, OfflineAgentProvider,
    TaskBridge, UiCell, UiCommand, UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiQueryFolderSummary,
    UiQueryResult, UiSavedQuerySummary, UiSchemaSummary, UiSslMode, UiTableDataFilter, UiTableDataSort, UiTableInfo,
    UiTableSummary,
};
use bigdecimal::BigDecimal;
use eframe::egui::text::LayoutJob;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, TextEdit, TextFormat, TopBottomPanel};
use lucide_icons::Icon;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::sync::Arc;

#[path = "agent_view.rs"]
mod agent_view;
#[path = "app_state.rs"]
mod app_state;
#[path = "connection_view.rs"]
mod connection_view;
#[path = "diagram_view.rs"]
mod diagram_view;
#[path = "events.rs"]
mod events;
#[path = "navigation_view.rs"]
mod navigation_view;
#[path = "palette_view.rs"]
mod palette_view;
#[path = "query_view.rs"]
mod query_view;
#[path = "schema_object_view.rs"]
mod schema_object_view;
#[path = "table_editor_view.rs"]
mod table_editor_view;
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
    History,
    Settings,
    Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteMode {
    QuickOpen,
    Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
enum WorkspaceTab {
    Welcome,
    Query,
    Table,
    SchemaObject,
    Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableView {
    Structure,
    Data,
    Ddl,
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
    active_tab: WorkspaceTab,
    sidebar_open: bool,
    agent_open: bool,
    sidebar_open_before_agent: Option<bool>,
    query_text: String,
    welcome_prompt: String,
    selected_query: String,
    query_documents: Vec<QueryDocument>,
    active_query_document: usize,
    editor_search: String,
    editor_search_open: bool,
    editor_font_size: f32,
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
    task_bridge: TaskBridge,
    next_query_request: Option<crate::RequestId>,
    runtime_message: String,
    query_result: Option<UiQueryResult>,
    grid_filter: String,
    grid_sort_column: Option<usize>,
    grid_sort_desc: bool,
    grid_column_widths: Vec<f32>,
    grid_resize_start: Option<(usize, f32)>,
    selected_cell: Option<(usize, usize)>,
    selected_row: Option<usize>,
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
    mutation_table: String,
    mutation_column: String,
    mutation_value: String,
    mutation_pk_column: String,
    mutation_pk_value: String,
    mutation_delete_confirmation: bool,
    connections: Vec<UiConnectionSummary>,
    saved_queries: Vec<UiSavedQuerySummary>,
    query_folders: Vec<UiQueryFolderSummary>,
    schema: UiSchemaSummary,
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
    table_data_filter_column: String,
    table_data_filter_value: String,
    table_data_sort_column: Option<String>,
    table_data_sort_desc: bool,
    table_data_error: Option<String>,
    table_info_request: Option<crate::RequestId>,
    table_ddl_request: Option<crate::RequestId>,
    table_data_request: Option<crate::RequestId>,
    table_mutation_request: Option<crate::RequestId>,
    table_view: TableView,
    query_folder: String,
    backup_output_path: String,
    restore_input_path: String,
    restore_confirmation: bool,
    active_connection_id: Option<String>,
    pending_connection_request: Option<crate::RequestId>,
    connections_requested: bool,
    connection_dialog_open: bool,
    editing_connection_id: Option<String>,
    connection_draft: UiConnectionDraft,
    connection_error: String,
    connection_test_valid: bool,
    connection_test_draft: Option<UiConnectionDraft>,
    delete_confirmation_id: Option<String>,
    folder_delete_confirmation: Option<String>,
}

impl eframe::App for DbProApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(widths) = serde_json::to_string(&self.grid_column_widths) {
            storage.set_string("dbpro.native.grid-widths", widths);
        }
        self.persist_active_query_document();
        if let Ok(documents) = serde_json::to_string(&self.query_documents) {
            storage.set_string("dbpro.native.query-documents", documents);
        }
        storage.set_string("dbpro.native.dark-mode", self.dark_mode.to_string());
        storage.set_string("dbpro.native.reduce-motion", self.reduce_motion.to_string());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.request_connections_once();
        self.apply_runtime_events();
        self.theme = if self.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
        self.theme.apply(ctx);
        self.handle_shortcuts(ctx);
        self.draw_topbar(ctx);
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
    }
}

impl DbProApp {
    fn primary_modifier_label() -> &'static str {
        if cfg!(target_os = "macos") {
            "⌘"
        } else {
            "Ctrl"
        }
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

    fn active_schema(&self) -> &str {
        if self.active_driver().eq_ignore_ascii_case("sqlite") {
            "main"
        } else {
            "public"
        }
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

    fn connection_indicator(&self, connection: &UiConnectionSummary) -> (Icon, Color32) {
        let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
        let is_connected = is_active && self.connected;
        let icon = if is_connected { Icon::CircleCheck } else { Icon::Circle };
        let color = if is_connected && connection.readonly {
            self.theme.warning
        } else if is_connected {
            self.theme.success
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

    fn switch_query_document(&mut self, index: usize) {
        if index >= self.query_documents.len() || index == self.active_query_document {
            return;
        }
        self.persist_active_query_document();
        self.active_query_document = index;
        self.query_text = self.query_documents[index].content.clone();
        self.query_result = None;
        self.runtime_message = format!("Opened {}", self.query_documents[index].title);
    }

    fn new_query_document(&mut self) {
        self.persist_active_query_document();
        let index = self.query_documents.len() + 1;
        self.query_documents.push(QueryDocument {
            title: format!("Query {index}"),
            content: String::new(),
        });
        self.active_query_document = self.query_documents.len() - 1;
        self.query_text.clear();
        self.query_result = None;
        self.active_tab = WorkspaceTab::Query;
    }

    fn close_query_document(&mut self, index: usize) {
        if self.query_documents.len() <= 1 || index >= self.query_documents.len() {
            return;
        }

        self.persist_active_query_document();
        self.query_documents.remove(index);
        if self.active_query_document > index {
            self.active_query_document -= 1;
        } else if self.active_query_document == index {
            self.active_query_document = self.active_query_document.min(self.query_documents.len() - 1);
        }
        self.query_text = self.query_documents[self.active_query_document].content.clone();
        self.query_result = None;
        self.runtime_message = format!("Closed {}", self.query_documents[self.active_query_document].title);
    }

    fn request_close_workspace_tab(&mut self, tab: WorkspaceTab) {
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
                self.selected_cell = None;
                self.selected_row = None;
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
            WorkspaceTab::Welcome | WorkspaceTab::Query => return,
        }
        if self.active_tab == tab {
            self.active_tab = WorkspaceTab::Welcome;
        }
        self.runtime_message = "Workspace closed".to_owned();
    }

    fn agent_context(&self) -> AgentContext {
        let connection_name = Some(self.active_connection_name().to_owned());
        let driver = self.active_driver().to_owned();

        AgentContext {
            connection_name,
            driver,
            tables: self.schema.tables.clone(),
            columns: self.schema.columns.clone(),
        }
    }

    fn submit_agent_prompt(&mut self) {
        let prompt = self.agent_input.trim().to_owned();
        if prompt.is_empty() {
            return;
        }

        self.agent_messages.push(AgentMessage {
            role: AgentRole::User,
            content: prompt.clone(),
            sql: None,
            requires_confirmation: false,
        });
        let context = self.agent_context();
        let request_id = self.task_bridge.next_request_id();
        self.agent_request = Some(request_id);
        self.agent_pending_prompt = Some(prompt.clone());
        self.agent_pending_context = Some(context.clone());
        self.agent_input.clear();
        self.runtime_message = "Sending request to Codex…".to_owned();
        if self
            .task_bridge
            .send(UiCommand::RunAgent {
                request_id,
                prompt,
                context,
            })
            .is_err()
        {
            self.agent_request = None;
            self.runtime_message = "Agent runtime unavailable · using offline draft".to_owned();
            self.fallback_agent_response(None);
        }
    }

    fn fallback_agent_response(&mut self, reason: Option<&str>) {
        let Some(prompt) = self.agent_pending_prompt.take() else {
            return;
        };
        let context = self.agent_pending_context.take().unwrap_or_default();
        let mut response = self
            .agent_provider
            .respond(&prompt, &context)
            .unwrap_or_else(|error| AgentMessage {
                role: AgentRole::Assistant,
                content: format!("Agent provider unavailable: {error}"),
                sql: None,
                requires_confirmation: false,
            });
        if let Some(reason) = reason {
            response.content = format!("{reason}\n\n{}", response.content);
        }
        let info = self.agent_provider.info();
        self.agent_provider_label = info.label.to_owned();
        self.agent_provider_detail = info.detail.to_owned();
        self.agent_messages.push(response);
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

    fn set_agent_open(&mut self, open: bool, ctx: &egui::Context) {
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
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::ListConnections { request_id });
    }

    fn request_schema_introspection(&mut self, connection_id: String, force_refresh: bool) {
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::IntrospectSchema {
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
