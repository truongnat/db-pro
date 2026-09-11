use crate::{
    activity_bar_frame, badge, card_frame, compact_button, compact_button_with_icon, compact_icon_button,
    compact_icon_button_enabled, danger_button, editor_frame, ghost_button, ghost_button_with_icon, icon_button,
    icon_text, input, input_full_width, panel_frame, password_input, primary_button, primary_button_with_icon,
    secondary_button_with_icon, section_label, sidebar_frame, sidebar_item, tab_frame, toolbar_frame, AgentContext,
    AgentMessage, AgentProvider, AgentRole, DbProTheme, OfflineAgentProvider, TaskBridge, UiCell, UiCommand,
    UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiQueryFolderSummary, UiQueryResult,
    UiSavedQuerySummary, UiSchemaSummary, UiSslMode, UiTableDataFilter, UiTableDataSort, UiTableInfo, UiTableSummary,
};
use bigdecimal::BigDecimal;
use eframe::egui::text::LayoutJob;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, TextEdit, TextFormat, TopBottomPanel};
use lucide_icons::Icon;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use std::sync::Arc;

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
enum PaletteMode {
    QuickOpen,
    Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteAction {
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
struct PaletteItem {
    icon: Icon,
    title: String,
    subtitle: String,
    shortcut: Option<&'static str>,
    action: PaletteAction,
}

const AGENT_SIDEBAR_COLLAPSE_WIDTH: f32 = 1180.0;
const TABLE_PAGE_SIZE: u64 = 100;
const GRID_ROW_NUMBER_WIDTH: f32 = 48.0;
const ER_MAX_TABLES: usize = 5;
const ER_MAX_COLUMNS: usize = 8;
const ER_MAX_EDGES: usize = 6;
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
    activity: Activity,
    active_tab: WorkspaceTab,
    sidebar_open: bool,
    agent_open: bool,
    sidebar_open_before_agent: Option<bool>,
    query_text: String,
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
    selected_table: Option<String>,
    selected_schema_object: Option<SchemaObjectSelection>,
    schema_object_view: SchemaObjectView,
    diagram_zoom: f32,
    diagram_pan: egui::Vec2,
    diagram_pan_origin: Option<egui::Vec2>,
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
    connections_requested: bool,
    connection_dialog_open: bool,
    editing_connection_id: Option<String>,
    connection_draft: UiConnectionDraft,
    connection_error: String,
    delete_confirmation_id: Option<String>,
    folder_delete_confirmation: Option<String>,
}

impl DbProApp {
    pub fn with_task_bridge(task_bridge: TaskBridge) -> Self {
        Self::with_task_bridge_and_storage(task_bridge, None)
    }

    pub fn with_task_bridge_and_storage(task_bridge: TaskBridge, storage: Option<&dyn eframe::Storage>) -> Self {
        let mut app = Self {
            task_bridge,
            ..Self::default()
        };
        if let Some(storage) = storage {
            if let Some(widths) = storage.get_string("dbpro.native.grid-widths") {
                if let Ok(widths) = serde_json::from_str::<Vec<f32>>(&widths) {
                    app.grid_column_widths = widths.into_iter().map(|width| width.clamp(90.0, 520.0)).collect();
                }
            }
            if let Some(documents) = storage.get_string("dbpro.native.query-documents") {
                if let Ok(documents) = serde_json::from_str::<Vec<QueryDocument>>(&documents) {
                    if !documents.is_empty() {
                        app.query_documents = documents;
                        app.query_text = app.query_documents[0].content.clone();
                    }
                }
            }
        }
        app
    }
}

impl Default for DbProApp {
    fn default() -> Self {
        let offline_provider = OfflineAgentProvider;
        let offline_info = offline_provider.info();
        Self {
            theme: DbProTheme::default(),
            activity: Activity::Explorer,
            active_tab: WorkspaceTab::Welcome,
            sidebar_open: true,
            agent_open: false,
            sidebar_open_before_agent: None,
            query_text: "select\n  id, name, status\nfrom customers\nlimit 100;".to_owned(),
            selected_query: String::new(),
            query_documents: vec![QueryDocument {
                title: "Query 1".to_owned(),
                content: "select\n  id, name, status\nfrom customers\nlimit 100;".to_owned(),
            }],
            active_query_document: 0,
            editor_search: String::new(),
            editor_search_open: false,
            editor_font_size: 14.0,
            completion_open: false,
            snippets_open: false,
            diagnostics: Vec::new(),
            query_history: Vec::new(),
            connection_name: "Local PostgreSQL".to_owned(),
            connected: false,
            palette_mode: None,
            palette_query: String::new(),
            palette_selected: 0,
            palette_focus_requested: false,
            agent_provider: Box::new(offline_provider),
            agent_request: None,
            agent_pending_prompt: None,
            agent_pending_context: None,
            agent_provider_label: offline_info.label.to_owned(),
            agent_provider_detail: offline_info.detail.to_owned(),
            agent_input: String::new(),
            agent_messages: Vec::new(),
            task_bridge: TaskBridge::default(),
            next_query_request: None,
            runtime_message: "Ready".to_owned(),
            query_result: None,
            grid_filter: String::new(),
            grid_sort_column: None,
            grid_sort_desc: false,
            grid_column_widths: Vec::new(),
            grid_resize_start: None,
            selected_cell: None,
            selected_row: None,
            data_editing_cell: None,
            data_edit_value: String::new(),
            data_delete_confirmation: false,
            insert_row_open: false,
            insert_row_values: Vec::new(),
            insert_row_error: String::new(),
            copy_status: String::new(),
            export_open: false,
            export_format: "CSV".to_owned(),
            export_path: String::new(),
            mutation_table: String::new(),
            mutation_column: String::new(),
            mutation_value: String::new(),
            mutation_pk_column: String::new(),
            mutation_pk_value: String::new(),
            mutation_delete_confirmation: false,
            connections: Vec::new(),
            saved_queries: Vec::new(),
            query_folders: Vec::new(),
            schema: UiSchemaSummary {
                tables: Vec::new(),
                columns: Vec::new(),
                table_details: Vec::new(),
                views: Vec::new(),
                triggers: Vec::new(),
                functions: Vec::new(),
            },
            selected_table: None,
            selected_schema_object: None,
            schema_object_view: SchemaObjectView::Definition,
            diagram_zoom: 1.0,
            diagram_pan: egui::Vec2::ZERO,
            diagram_pan_origin: None,
            table_info: None,
            table_ddl: None,
            table_info_error: None,
            table_ddl_error: None,
            ddl_execute_confirmation: false,
            ddl_execution_request: None,
            refresh_table_info_after_schema: false,
            table_data_result: None,
            table_data_total_rows: None,
            table_data_offset: 0,
            table_data_filter_column: String::new(),
            table_data_filter_value: String::new(),
            table_data_sort_column: None,
            table_data_sort_desc: false,
            table_data_error: None,
            table_info_request: None,
            table_ddl_request: None,
            table_data_request: None,
            table_mutation_request: None,
            table_view: TableView::Structure,
            query_folder: String::new(),
            backup_output_path: String::new(),
            restore_input_path: String::new(),
            restore_confirmation: false,
            active_connection_id: None,
            connections_requested: false,
            connection_dialog_open: false,
            editing_connection_id: None,
            connection_draft: UiConnectionDraft::default(),
            connection_error: String::new(),
            delete_confirmation_id: None,
            folder_delete_confirmation: None,
        }
    }
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
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.request_connections_once();
        self.apply_runtime_events();
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

    fn palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        match mode {
            PaletteMode::QuickOpen => vec![
                PaletteItem {
                    icon: Icon::House,
                    title: "Welcome".to_owned(),
                    subtitle: "Database workspace home".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Welcome,
                },
                PaletteItem {
                    icon: Icon::FileCode2,
                    title: "Query".to_owned(),
                    subtitle: "Open the SQL editor".to_owned(),
                    shortcut: Some("⌘P"),
                    action: PaletteAction::Query,
                },
                PaletteItem {
                    icon: Icon::History,
                    title: "Query history".to_owned(),
                    subtitle: "Browse saved and recent queries".to_owned(),
                    shortcut: None,
                    action: PaletteAction::History,
                },
                PaletteItem {
                    icon: Icon::ArrowRightLeft,
                    title: "ER diagram".to_owned(),
                    subtitle: "Explore tables and relationships".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Diagram,
                },
                PaletteItem {
                    icon: Icon::Settings2,
                    title: "Settings".to_owned(),
                    subtitle: "Connections, backups and restore".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Settings,
                },
                PaletteItem {
                    icon: Icon::Bot,
                    title: "Agent".to_owned(),
                    subtitle: "Open the Codex database copilot".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Agent,
                },
            ],
            PaletteMode::Commands => vec![
                PaletteItem {
                    icon: Icon::Plus,
                    title: "New query".to_owned(),
                    subtitle: "Create a fresh SQL document".to_owned(),
                    shortcut: None,
                    action: PaletteAction::NewQuery,
                },
                PaletteItem {
                    icon: Icon::Database,
                    title: "New connection".to_owned(),
                    subtitle: "Add a PostgreSQL or SQLite connection".to_owned(),
                    shortcut: None,
                    action: PaletteAction::NewConnection,
                },
                PaletteItem {
                    icon: Icon::RotateCcw,
                    title: "Refresh schema".to_owned(),
                    subtitle: "Reload tables, views and relationships".to_owned(),
                    shortcut: None,
                    action: PaletteAction::RefreshSchema,
                },
                PaletteItem {
                    icon: Icon::PanelLeft,
                    title: "Toggle explorer".to_owned(),
                    subtitle: "Show or hide the connection sidebar".to_owned(),
                    shortcut: Some("⌘B"),
                    action: PaletteAction::ToggleExplorer,
                },
                PaletteItem {
                    icon: Icon::Bot,
                    title: "Open Agent".to_owned(),
                    subtitle: "Ask Codex about the active schema".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Agent,
                },
                PaletteItem {
                    icon: Icon::ArrowRightLeft,
                    title: "Open ER diagram".to_owned(),
                    subtitle: "Show the active schema map".to_owned(),
                    shortcut: None,
                    action: PaletteAction::Diagram,
                },
            ],
        }
    }

    fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let query = self.palette_query.trim().to_lowercase();
        self.palette_items(mode)
            .into_iter()
            .filter(|item| {
                query.is_empty()
                    || item.title.to_lowercase().contains(&query)
                    || item.subtitle.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn execute_palette_action(&mut self, action: PaletteAction, ctx: &egui::Context) {
        self.palette_mode = None;
        match action {
            PaletteAction::Welcome => self.active_tab = WorkspaceTab::Welcome,
            PaletteAction::Query => self.active_tab = WorkspaceTab::Query,
            PaletteAction::History => {
                self.activity = Activity::History;
                self.sidebar_open = true;
                self.active_tab = WorkspaceTab::Welcome;
            }
            PaletteAction::Settings => {
                self.activity = Activity::Settings;
                self.sidebar_open = true;
                self.active_tab = WorkspaceTab::Welcome;
            }
            PaletteAction::Diagram => {
                self.activity = Activity::Diagram;
                self.sidebar_open = true;
                self.active_tab = WorkspaceTab::Diagram;
            }
            PaletteAction::Agent => self.set_agent_open(true, ctx),
            PaletteAction::NewQuery => self.new_query_document(),
            PaletteAction::NewConnection => {
                self.connection_dialog_open = true;
                self.editing_connection_id = None;
                self.connection_draft = UiConnectionDraft::default();
                self.connection_error.clear();
            }
            PaletteAction::RefreshSchema => {
                if let Some(connection_id) = self.active_connection_id.clone() {
                    self.refresh_table_info_after_schema = self.selected_table.is_some();
                    let request_id = self.task_bridge.next_request_id();
                    let _ = self.task_bridge.send(UiCommand::IntrospectSchema {
                        request_id,
                        connection_id,
                        force_refresh: true,
                    });
                    self.runtime_message = "Refreshing schema…".to_owned();
                } else {
                    self.runtime_message = "Connect to a database before refreshing schema".to_owned();
                }
            }
            PaletteAction::ToggleExplorer => self.sidebar_open = !self.sidebar_open,
        }
    }

    fn draw_palette(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.palette_mode else {
            return;
        };
        let items = self.filtered_palette_items(mode);
        if items.is_empty() {
            self.palette_selected = 0;
        } else {
            self.palette_selected = self.palette_selected.min(items.len() - 1);
        }
        let mut activate = false;
        egui::Area::new(egui::Id::new("palette_scrim"))
            .order(egui::Order::Foreground)
            .fixed_pos(ctx.screen_rect().min)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, ctx.screen_rect().size()),
                    0.0,
                    Color32::from_black_alpha(24),
                );
            });
        egui::Window::new("command_palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .default_width(560.0)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 72.0))
            .frame(card_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(
                        if mode == PaletteMode::QuickOpen {
                            Icon::Search
                        } else {
                            Icon::Command
                        },
                        if mode == PaletteMode::QuickOpen {
                            "Quick Open"
                        } else {
                            "Command Palette"
                        },
                        self.theme.accent,
                    ));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::X, self.theme).clicked() {
                            self.palette_mode = None;
                        }
                    });
                });
                ui.add_space(8.0);
                let response = ui.add(
                    TextEdit::singleline(&mut self.palette_query)
                        .hint_text(RichText::new("Search commands and workspaces…").color(self.theme.text_muted))
                        .desired_width(ui.available_width())
                        .margin(egui::Margin::symmetric(10.0, 7.0))
                        .text_color(self.theme.text_primary),
                );
                if self.palette_focus_requested {
                    response.request_focus();
                    self.palette_focus_requested = false;
                }
                if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                    self.palette_mode = None;
                    return;
                }
                if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) && !items.is_empty() {
                    self.palette_selected = (self.palette_selected + 1) % items.len();
                }
                if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) && !items.is_empty() {
                    self.palette_selected = if self.palette_selected == 0 {
                        items.len() - 1
                    } else {
                        self.palette_selected - 1
                    };
                }
                if ctx.input(|input| input.key_pressed(egui::Key::Enter)) && !items.is_empty() {
                    activate = true;
                }
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    if items.is_empty() {
                        ui.label(RichText::new("No matching command").color(self.theme.text_muted));
                    }
                    for (index, item) in items.iter().enumerate() {
                        let selected = index == self.palette_selected;
                        let response = ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 42.0),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let frame = tab_frame(self.theme, selected);
                                frame.show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(icon_text(item.icon, "", self.theme.accent));
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&item.title).color(self.theme.text_primary));
                                            ui.label(
                                                RichText::new(&item.subtitle).small().color(self.theme.text_muted),
                                            );
                                        });
                                        if let Some(shortcut) = item.shortcut {
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(RichText::new(shortcut).small().color(self.theme.text_muted));
                                            });
                                        }
                                    });
                                });
                            },
                        );
                        if response.response.hovered() {
                            self.palette_selected = index;
                        }
                        if response.response.clicked() {
                            self.palette_selected = index;
                            activate = true;
                        }
                    }
                });
                ui.add_space(6.0);
                ui.label(
                    RichText::new("↑↓ to navigate · Enter to open · Esc to close")
                        .small()
                        .color(self.theme.text_muted),
                );
            });
        if activate {
            if let Some(item) = items.get(self.palette_selected) {
                self.execute_palette_action(item.action, ctx);
            }
        }
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

    fn request_table_info(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_info_request = Some(request_id);
        let _ = self.task_bridge.send(UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table structure…".to_owned();
    }

    fn can_mutate_active_connection(&self) -> bool {
        self.connected && self.active_connection().is_some_and(|connection| !connection.readonly)
    }

    fn row_identity(
        result: &UiQueryResult,
        info: &UiTableInfo,
        row_index: usize,
    ) -> Result<(Vec<String>, Vec<UiCell>), String> {
        let Some(primary_key) = info.primary_key.as_ref() else {
            return Err("This table has no primary key for safe row editing".to_owned());
        };
        let mut pk_values = Vec::with_capacity(primary_key.len());
        for pk_column in primary_key {
            let Some(pk_index) = result.columns.iter().position(|item| item.name == *pk_column) else {
                return Err(format!(
                    "The primary-key column {pk_column} is not present in this result"
                ));
            };
            let Some(pk_cell) = result.rows.get(row_index).and_then(|row| row.get(pk_index)) else {
                return Err("The selected row is no longer available".to_owned());
            };
            if matches!(pk_cell, UiCell::Null) {
                return Err(format!("A NULL primary key ({pk_column}) cannot identify a row"));
            }
            pk_values.push(pk_cell.clone());
        }
        Ok((primary_key.clone(), pk_values))
    }

    fn begin_data_cell_edit(&mut self, row_index: usize, column_index: usize, cell: &UiCell) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to edit rows".to_owned();
            return;
        }
        self.selected_cell = Some((row_index, column_index));
        self.selected_row = Some(row_index);
        self.data_editing_cell = Some((row_index, column_index));
        self.data_edit_value = match cell {
            UiCell::Null => String::new(),
            _ => crate::cell_text(cell),
        };
        self.copy_status.clear();
    }

    fn submit_data_cell_edit(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(table) = self.selected_table.clone() else {
            self.data_editing_cell = None;
            return;
        };
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            self.data_editing_cell = None;
            return;
        };
        let Some(column_info) = info.columns.iter().find(|item| item.name == column) else {
            self.runtime_message = "The selected column is not present in the table metadata".to_owned();
            self.data_editing_cell = None;
            return;
        };
        let value = match Self::parse_update_value(&self.data_edit_value, &column_info.data_type) {
            Ok(value) => value,
            Err(error) => {
                self.runtime_message = format!("{}: {error}", column_info.name);
                self.data_editing_cell = None;
                return;
            }
        };
        let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.runtime_message = error;
                self.data_editing_cell = None;
                return;
            }
        };
        let Some(connection) = self.active_connection().cloned() else {
            self.data_editing_cell = None;
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::UpdateTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            column,
            value,
            pk_columns,
            pk_values,
        });
        self.table_mutation_request = Some(request_id);
        self.data_editing_cell = None;
        self.runtime_message = "Saving cell…".to_owned();
    }

    fn request_delete_selected_data_row(&mut self, result: &UiQueryResult) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to delete rows".to_owned();
            return;
        }
        let Some(row_index) = self.selected_row else {
            self.runtime_message = "Select a row before deleting".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        if let Err(error) = Self::row_identity(result, &info, row_index) {
            self.runtime_message = error;
            return;
        }
        self.data_delete_confirmation = true;
        self.data_editing_cell = None;
        self.data_edit_value.clear();
    }

    fn submit_delete_selected_data_row(&mut self, result: &UiQueryResult) {
        let Some(row_index) = self.selected_row else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Select a row before deleting".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.data_delete_confirmation = false;
                self.runtime_message = error;
                return;
            }
        };
        let Some(connection) = self.active_connection().cloned() else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Connect to a database before deleting a row".to_owned();
            return;
        };
        let Some(table) = self.selected_table.clone() else {
            self.data_delete_confirmation = false;
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::DeleteTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            pk_columns,
            pk_values,
        });
        self.table_mutation_request = Some(request_id);
        self.data_delete_confirmation = false;
        self.selected_cell = None;
        self.selected_row = None;
        self.runtime_message = "Deleting row…".to_owned();
    }

    fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_ddl_request = Some(request_id);
        let _ = self.task_bridge.send(UiCommand::LoadTableDdl {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table DDL…".to_owned();
    }

    fn request_table_data(&mut self) {
        let Some(connection_id) = self.active_connection_id.clone() else {
            return;
        };
        let table = self
            .selected_table
            .clone()
            .or_else(|| match self.selected_schema_object.as_ref() {
                Some(SchemaObjectSelection::View(name)) => Some(name.clone()),
                _ => None,
            });
        let Some(table) = table else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_data_request = Some(request_id);
        let filter = (!self.table_data_filter_value.trim().is_empty()
            && !self.table_data_filter_column.trim().is_empty())
        .then(|| UiTableDataFilter {
            column: self.table_data_filter_column.clone(),
            value: self.table_data_filter_value.trim().to_owned(),
        });
        let sort = self.table_data_sort_column.clone().map(|column| UiTableDataSort {
            column,
            descending: self.table_data_sort_desc,
        });
        let _ = self.task_bridge.send(UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
            limit: TABLE_PAGE_SIZE,
            offset: self.table_data_offset,
            filter,
            sort,
        });
        self.runtime_message = "Loading table data…".to_owned();
    }

    fn reload_table_data_from_start(&mut self) {
        self.table_data_offset = 0;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_error = None;
        self.grid_filter.clear();
        self.grid_sort_column = None;
        self.grid_sort_desc = false;
        self.request_table_data();
    }

    fn request_connections_once(&mut self) {
        if self.connections_requested {
            return;
        }
        self.connections_requested = true;
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::ListConnections { request_id });
    }

    fn apply_runtime_events(&mut self) {
        let events: Vec<UiEvent> = self.task_bridge.drain_events().collect();
        for event in events {
            match event {
                UiEvent::ConnectionsLoaded { connections, .. } => {
                    self.connections = connections;
                    if self.active_connection_id.is_none() {
                        self.active_connection_id = self.connections.first().map(|connection| connection.id.clone());
                    }
                    self.runtime_message = format!("Loaded {} connections", self.connections.len());
                    if let Some(connection_id) = self.active_connection_id.clone() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::ListSavedQueries {
                            request_id,
                            connection_id: connection_id.clone(),
                        });
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::IntrospectSchema {
                            request_id,
                            connection_id: connection_id.clone(),
                            force_refresh: false,
                        });
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::ListQueryFolders {
                            request_id,
                            connection_id,
                        });
                    }
                }
                UiEvent::SavedQueriesLoaded { queries, .. } => {
                    self.saved_queries = queries;
                }
                UiEvent::QueryFoldersLoaded { folders, .. } => {
                    self.query_folders = folders;
                }
                UiEvent::SchemaLoaded { schema, .. } => {
                    let refresh_selected_table = self.refresh_table_info_after_schema;
                    self.refresh_table_info_after_schema = false;
                    self.schema = schema;
                    if self
                        .selected_table
                        .as_ref()
                        .is_some_and(|table| !self.schema.tables.iter().any(|candidate| candidate == table))
                    {
                        self.selected_table = None;
                        self.selected_schema_object = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_offset = 0;
                        self.table_data_filter_column.clear();
                        self.table_data_filter_value.clear();
                        self.table_data_sort_column = None;
                        self.table_data_sort_desc = false;
                        self.table_data_error = None;
                        self.table_info_request = None;
                        self.table_ddl_request = None;
                        self.table_data_request = None;
                    }
                    let object_exists = match self.selected_schema_object.as_ref() {
                        Some(SchemaObjectSelection::View(name)) => {
                            self.schema.views.iter().any(|view| &view.name == name)
                        }
                        Some(SchemaObjectSelection::Trigger(name)) => {
                            self.schema.triggers.iter().any(|trigger| &trigger.name == name)
                        }
                        Some(SchemaObjectSelection::Function(name)) => {
                            self.schema.functions.iter().any(|function| &function.name == name)
                        }
                        None => true,
                    };
                    if !object_exists {
                        self.selected_schema_object = None;
                        if self.active_tab == WorkspaceTab::SchemaObject {
                            self.active_tab = WorkspaceTab::Welcome;
                        }
                    }
                    self.runtime_message = format!(
                        "Schema loaded · {} tables · {} views · {} triggers · {} functions",
                        self.schema.tables.len(),
                        self.schema.views.len(),
                        self.schema.triggers.len(),
                        self.schema.functions.len()
                    );
                    if refresh_selected_table && self.active_tab == WorkspaceTab::Table && self.selected_table.is_some()
                    {
                        self.request_table_info();
                    }
                }
                UiEvent::AgentCompleted {
                    request_id,
                    provider,
                    message,
                } => {
                    if self.agent_request == Some(request_id) {
                        self.agent_request = None;
                        self.agent_pending_prompt = None;
                        self.agent_pending_context = None;
                        self.agent_provider_label = provider;
                        self.agent_provider_detail = "OpenAI Responses API · SQL drafts stay unexecuted".to_owned();
                        self.agent_messages.push(message);
                        self.runtime_message = "Codex response received".to_owned();
                    }
                }
                UiEvent::AgentFailed { request_id, message } => {
                    if self.agent_request == Some(request_id) {
                        self.agent_request = None;
                        self.runtime_message = "Codex unavailable · switched to offline draft".to_owned();
                        self.fallback_agent_response(Some(&format!("Codex unavailable: {message}")));
                    }
                }
                UiEvent::TableInfoLoaded { request_id, table_info } => {
                    if self.table_info_request == Some(request_id) {
                        if self.table_data_filter_column.is_empty() {
                            self.table_data_filter_column = table_info
                                .columns
                                .first()
                                .map(|column| column.name.clone())
                                .unwrap_or_default();
                        }
                        if self.table_data_sort_column.is_none() {
                            self.table_data_sort_column = table_info
                                .primary_key
                                .as_ref()
                                .and_then(|columns| columns.first().cloned())
                                .or_else(|| table_info.columns.first().map(|column| column.name.clone()));
                        }
                        self.table_info = Some(table_info);
                        self.table_info_error = None;
                        self.table_info_request = None;
                        self.runtime_message = "Table structure loaded".to_owned();
                    }
                }
                UiEvent::TableDdlLoaded { request_id, sql } => {
                    if self.table_ddl_request == Some(request_id) {
                        self.table_ddl = Some(sql);
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = None;
                        self.table_ddl_request = None;
                        self.runtime_message = "Table DDL loaded".to_owned();
                    }
                }
                UiEvent::TableDataLoaded {
                    request_id,
                    result,
                    total_rows,
                } => {
                    if self.table_data_request == Some(request_id) {
                        if self.table_data_filter_column.is_empty() {
                            self.table_data_filter_column = result
                                .columns
                                .first()
                                .map(|column| column.name.clone())
                                .unwrap_or_default();
                        }
                        if self.table_data_sort_column.is_none() {
                            self.table_data_sort_column = result.columns.first().map(|column| column.name.clone());
                        }
                        self.table_data_result = Some(result);
                        self.table_data_total_rows = Some(total_rows);
                        self.table_data_error = None;
                        self.table_data_request = None;
                        self.runtime_message = format!("Table data loaded · {total_rows} rows");
                    }
                }
                UiEvent::FilePicked { kind, path, .. } => {
                    if let Some(path) = path {
                        if kind == "sqlite" {
                            self.connection_draft.database = path;
                        } else if kind == "ssh-key" {
                            self.connection_draft.ssh_private_key = path;
                        } else if kind == "backup" {
                            self.backup_output_path = path;
                        } else if kind == "restore" {
                            self.restore_input_path = path;
                        }
                        self.connection_error.clear();
                    }
                }
                UiEvent::OperationProgress { operation, status, .. } => {
                    self.runtime_message = format!("{operation}: {status}");
                }
                UiEvent::BackupCompleted {
                    output_path,
                    size_bytes,
                    ..
                } => {
                    self.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
                }
                UiEvent::DdlCompleted {
                    request_id,
                    affected_rows,
                } => {
                    if self.ddl_execution_request == Some(request_id) {
                        self.ddl_execution_request = None;
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = None;
                        self.refresh_table_info_after_schema = self.selected_table.is_some();
                        self.runtime_message = format!("DDL applied · {affected_rows} affected rows");
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::IntrospectSchema {
                                request_id,
                                connection_id,
                                force_refresh: true,
                            });
                        }
                    }
                }
                UiEvent::OperationCompleted { request_id, operation } => {
                    self.runtime_message = operation.clone();
                    self.connections_requested = false;
                    if operation.starts_with("table-row.") {
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        if self.table_mutation_request == Some(request_id) {
                            self.table_mutation_request = None;
                            self.table_data_result = None;
                            self.table_data_total_rows = None;
                            self.table_data_error = None;
                            self.table_data_request = None;
                            if self.active_tab == WorkspaceTab::Table {
                                self.request_table_data();
                            }
                        }
                    }
                    if operation == "connection.created" || operation == "connection.updated" {
                        self.connection_dialog_open = false;
                        self.editing_connection_id = None;
                    }
                    if operation.starts_with("query") || operation.starts_with("query-folder") {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::ListSavedQueries {
                                request_id,
                                connection_id,
                            });
                        }
                    }
                    if operation == "connection.deleted" {
                        self.active_connection_id = None;
                        self.connected = false;
                    }
                }
                UiEvent::Connected { connection_id, .. } => {
                    self.active_connection_id = Some(connection_id);
                    self.connected = true;
                    self.runtime_message = "Connection established".to_owned();
                    if let Some(connection_id) = self.active_connection_id.clone() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::IntrospectSchema {
                            request_id,
                            connection_id,
                            force_refresh: false,
                        });
                    }
                }
                UiEvent::QueryQueued { request_id } => {
                    self.next_query_request = Some(request_id);
                    self.runtime_message = format!("Query queued · request {}", request_id.0);
                }
                UiEvent::QueryCompleted { request_id, result } => {
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query completed · {} rows", result.row_count);
                        self.grid_sort_column = None;
                        self.grid_column_widths = vec![180.0; result.columns.len()];
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.copy_status.clear();
                        self.query_result = Some(result);
                        self.next_query_request = None;
                    }
                }
                UiEvent::QueryCancelled { request_id } => {
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = "Query cancelled".to_owned();
                        self.next_query_request = None;
                    }
                }
                UiEvent::QueryFailed { request_id, message } => {
                    if self.table_mutation_request == Some(request_id) {
                        self.table_mutation_request = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.runtime_message = format!("Row mutation failed · {message}");
                    } else if self.table_info_request == Some(request_id) {
                        self.table_info_request = None;
                        self.table_info_error = Some(message.clone());
                        self.runtime_message = format!("Table structure failed · {message}");
                    } else if self.table_ddl_request == Some(request_id) {
                        self.table_ddl_request = None;
                        self.table_ddl_error = Some(message.clone());
                        self.runtime_message = format!("Table DDL failed · {message}");
                    } else if self.table_data_request == Some(request_id) {
                        self.table_data_request = None;
                        self.table_data_error = Some(message.clone());
                        self.runtime_message = format!("Table data failed · {message}");
                    } else if self.ddl_execution_request == Some(request_id) {
                        self.ddl_execution_request = None;
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = Some(message.clone());
                        self.runtime_message = format!("DDL execution failed · {message}");
                    } else if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query failed · {message}");
                        self.next_query_request = None;
                    }
                }
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.palette_mode.is_some() {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.command && i.modifiers.shift) {
            self.open_palette(PaletteMode::Commands);
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.command) {
            self.open_palette(PaletteMode::QuickOpen);
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.command) {
            self.sidebar_open = !self.sidebar_open;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) {
            self.editor_search_open = true;
        }
        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5) || (!self.agent_open && i.key_pressed(egui::Key::Enter) && i.modifiers.command)
        }) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.next_query_request {
                self.cancel_query(request_id);
            } else if self.editor_search_open {
                self.editor_search_open = false;
            } else {
                self.set_agent_open(false, ctx);
            }
        }
    }

    fn cancel_query(&mut self, request_id: crate::RequestId) {
        let _ = self.task_bridge.send(UiCommand::CancelQuery { request_id });
        self.runtime_message = "Cancelling query…".to_owned();
    }

    fn dispatch_query(&mut self) {
        if self.next_query_request.is_some() {
            return;
        }
        let Some(connection_id) = self.active_connection().map(|connection| connection.id.clone()) else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() {
            self.query_text.clone()
        } else {
            self.selected_query.clone()
        };
        if !self.query_history.iter().any(|query| query == &sql) {
            self.query_history.push(sql.clone());
            if self.query_history.len() > 20 {
                self.query_history.remove(0);
            }
        }
        let request_id = self.task_bridge.next_request_id();
        self.next_query_request = Some(request_id);
        self.runtime_message = "Sending query to runtime…".to_owned();
        let _ = self.task_bridge.send(UiCommand::RunQuery {
            request_id,
            connection_id,
            sql,
        });
    }

    fn draw_topbar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("topbar")
            .exact_height(48.0)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("DB").strong().color(self.theme.accent));
                    ui.label(RichText::new("PRO").strong().color(self.theme.text_primary));
                    ui.separator();
                    ui.label(RichText::new("Workspace").color(self.theme.text_secondary));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ghost_button_with_icon(ui, Icon::Bot, "Agent", self.theme).clicked() {
                            self.set_agent_open(!self.agent_open, ctx);
                        }
                        if ghost_button_with_icon(ui, Icon::Search, "Quick Open", self.theme).clicked() {
                            self.open_palette(PaletteMode::QuickOpen);
                        }
                        if compact_icon_button(ui, Icon::Command, self.theme)
                            .on_hover_text("Command Palette (⌘⇧P)")
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
                        ui.label(
                            RichText::new("v0.1 native preview")
                                .small()
                                .color(self.theme.text_muted),
                        );
                    });
                });
            });
    }

    fn draw_statusbar(&self, ctx: &egui::Context) {
        TopBottomPanel::bottom("statusbar")
            .exact_height(26.0)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    let (color, label) = if self.connected {
                        (self.theme.success, "Connected")
                    } else {
                        (self.theme.warning, "Not connected")
                    };
                    ui.label(icon_text(Icon::CircleCheck, "", color));
                    ui.label(RichText::new(label).small().color(self.theme.text_secondary));
                    ui.separator();
                    ui.label(RichText::new(self.active_driver()).small().color(self.theme.text_muted));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("UTF-8").small().color(self.theme.text_muted));
                        ui.label(RichText::new("Ln 1, Col 1").small().color(self.theme.text_muted));
                    });
                });
            });
    }

    fn draw_activity_bar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("activity_bar")
            .resizable(false)
            .exact_width(54.0)
            .frame(activity_bar_frame(self.theme))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    for (activity, icon, hint) in [
                        (Activity::Explorer, Icon::Database, "Explorer"),
                        (Activity::History, Icon::History, "History"),
                        (Activity::Settings, Icon::Settings2, "Settings"),
                        (Activity::Diagram, Icon::ArrowRightLeft, "ER diagram"),
                    ] {
                        let active = self.activity == activity;
                        let response = icon_button(ui, icon, active, self.theme);
                        if response.on_hover_text(hint).clicked() {
                            self.activity = activity;
                            self.sidebar_open = true;
                            if activity == Activity::Diagram {
                                self.active_tab = WorkspaceTab::Diagram;
                            }
                        }
                        ui.add_space(4.0);
                    }
                });
            });
    }

    fn draw_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .default_width(260.0)
            .min_width(220.0)
            .max_width(380.0)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    section_label(
                        ui,
                        match self.activity {
                            Activity::Explorer => "EXPLORER",
                            Activity::History => "QUERY HISTORY",
                            Activity::Settings => "SETTINGS",
                            Activity::Diagram => "ER DIAGRAM",
                        },
                        self.theme,
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::PanelLeftClose, self.theme)
                            .on_hover_text("Hide sidebar (⌘B)")
                            .clicked()
                        {
                            self.sidebar_open = false;
                        }
                    });
                });
                ui.add_space(12.0);

                match self.activity {
                    Activity::Explorer => self.draw_explorer(ui),
                    Activity::History => self.draw_history(ui),
                    Activity::Settings => self.draw_settings(ui),
                    Activity::Diagram => self.draw_diagram_sidebar(ui),
                }
            });
    }

    fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} tables · {} relationships",
                    self.schema.table_details.len(),
                    self.schema
                        .table_details
                        .iter()
                        .map(|table| table.foreign_keys.len())
                        .sum::<usize>()
                ))
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(10.0);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "NAVIGATION", self.theme);
            ui.add_space(8.0);
            ui.label(
                RichText::new("Drag the canvas to pan. Use the controls above the map to zoom or fit the schema.")
                    .small()
                    .color(self.theme.text_secondary),
            );
            ui.add_space(10.0);
            if secondary_button_with_icon(ui, Icon::Database, "Back to Explorer", self.theme).clicked() {
                self.activity = Activity::Explorer;
                self.sidebar_open = true;
            }
        });
    }

    fn draw_explorer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::ChevronDown, "", self.theme.text_muted));
            ui.label(RichText::new(self.active_connection_name()).strong());
            ui.label(RichText::new(self.active_driver()).small().color(self.theme.accent));
        });
        ui.add_space(8.0);
        if self.connections.is_empty() {
            ui.label(RichText::new("No saved connections").color(self.theme.text_muted));
            ui.label(
                RichText::new("Create one from the next backend slice.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for connection in self.connections.clone() {
                let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
                ui.horizontal(|ui| {
                    let (connection_state_icon, connection_state_color) = self.connection_indicator(&connection);
                    ui.label(icon_text(connection_state_icon, "", connection_state_color));
                    let connection_button = ui.add(
                        egui::Button::new(
                            RichText::new(connection.name.as_str())
                                .strong()
                                .color(self.theme.text_primary),
                        )
                        .fill(if is_active {
                            self.theme.accent_soft
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(egui::Stroke::NONE)
                        .rounding(egui::Rounding::same(5.0)),
                    );
                    if connection_button.clicked() {
                        self.active_connection_id = Some(connection.id.clone());
                        self.selected_table = None;
                        self.selected_schema_object = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_offset = 0;
                        self.table_data_filter_column.clear();
                        self.table_data_filter_value.clear();
                        self.table_data_sort_column = None;
                        self.table_data_sort_desc = false;
                        self.table_data_error = None;
                        self.table_info_request = None;
                        self.table_ddl_request = None;
                        self.table_data_request = None;
                        self.table_mutation_request = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.table_view = TableView::Structure;
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::Connect {
                            request_id,
                            connection_id: connection.id.clone(),
                        });
                        self.runtime_message = format!("Connecting to {}…", connection.name);
                    }
                    ui.label(
                        RichText::new(connection.driver.as_str())
                            .small()
                            .color(self.theme.accent),
                    );
                    if is_active && compact_button(ui, "Edit", self.theme).clicked() {
                        self.open_edit_connection(&connection);
                    }
                    if is_active
                        && compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Delete connection")
                            .clicked()
                    {
                        self.delete_confirmation_id = Some(connection.id.clone());
                    }
                });
            }
        }
        ui.add_space(12.0);
        let schema_name = self.active_schema().to_owned();
        ui.collapsing(icon_text(Icon::Layers, "Schemas", self.theme.text_primary), |ui| {
            let _ = sidebar_item(ui, Icon::Database, &schema_name, false, self.theme);
        });
        let tables = self.schema.tables.clone();
        ui.collapsing(
            icon_text(
                Icon::Table2,
                &format!("Tables ({})", tables.len()),
                self.theme.text_primary,
            ),
            |ui| {
                for table in tables.iter().take(100) {
                    let is_selected = self.selected_table.as_deref() == Some(table.as_str());
                    if sidebar_item(ui, Icon::Table2, table, is_selected, self.theme).clicked() {
                        self.selected_table = Some(table.clone());
                        self.selected_schema_object = None;
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_offset = 0;
                        self.table_data_filter_column.clear();
                        self.table_data_filter_value.clear();
                        self.table_data_sort_column = None;
                        self.table_data_sort_desc = false;
                        self.table_data_error = None;
                        self.table_info_request = None;
                        self.table_ddl_request = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Structure;
                        self.table_mutation_request = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
                        self.request_table_info();
                        self.active_tab = WorkspaceTab::Table;
                    }
                    if is_selected {
                        if let Some(info) = self.table_info.clone() {
                            ui.indent(("table-sidebar-details", table.as_str()), |ui| {
                                ui.collapsing(
                                    icon_text(
                                        Icon::Columns3,
                                        &format!("Columns ({})", info.columns.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        for column in &info.columns {
                                            ui.label(
                                                RichText::new(format!("{} · {}", column.name, column.data_type))
                                                    .small()
                                                    .color(self.theme.text_muted),
                                            );
                                        }
                                    },
                                );
                                ui.collapsing(
                                    icon_text(
                                        Icon::List,
                                        &format!("Indexes ({})", info.indexes.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        if info.indexes.is_empty() {
                                            ui.label(RichText::new("No indexes").small().color(self.theme.text_muted));
                                        }
                                        for index in &info.indexes {
                                            ui.label(RichText::new(&index.name).small().color(self.theme.text_muted));
                                        }
                                    },
                                );
                                ui.collapsing(
                                    icon_text(
                                        Icon::ArrowRightLeft,
                                        &format!("Foreign keys ({})", info.foreign_keys.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        if info.foreign_keys.is_empty() {
                                            ui.label(
                                                RichText::new("No foreign keys").small().color(self.theme.text_muted),
                                            );
                                        }
                                        for foreign_key in &info.foreign_keys {
                                            ui.label(
                                                RichText::new(&foreign_key.name).small().color(self.theme.text_muted),
                                            );
                                        }
                                    },
                                );
                            });
                        }
                    }
                }
            },
        );
        let views = self.schema.views.clone();
        ui.collapsing(
            icon_text(Icon::Eye, &format!("Views ({})", views.len()), self.theme.text_primary),
            |ui| {
                if views.is_empty() {
                    ui.label(RichText::new("No views").small().color(self.theme.text_muted));
                }
                for view in views.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::View(selected)) if selected == &view.name
                    );
                    if sidebar_item(ui, Icon::Eye, &view.name, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::View(view.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened view {}.{}", view.schema, view.name);
                    }
                }
            },
        );
        let triggers = self.schema.triggers.clone();
        ui.collapsing(
            icon_text(
                Icon::Zap,
                &format!("Triggers ({})", triggers.len()),
                self.theme.text_primary,
            ),
            |ui| {
                if triggers.is_empty() {
                    ui.label(RichText::new("No triggers").small().color(self.theme.text_muted));
                }
                for trigger in triggers.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Trigger(selected)) if selected == &trigger.name
                    );
                    let label = format!("{} · {}", trigger.name, trigger.event);
                    if sidebar_item(ui, Icon::Zap, &label, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Trigger(trigger.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened trigger {}", trigger.name);
                    }
                }
            },
        );
        let functions = self.schema.functions.clone();
        ui.collapsing(
            icon_text(
                Icon::Code2,
                &format!("Functions ({})", functions.len()),
                self.theme.text_primary,
            ),
            |ui| {
                if functions.is_empty() {
                    ui.label(RichText::new("No functions").small().color(self.theme.text_muted));
                }
                for function in functions.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Function(selected)) if selected == &function.name
                    );
                    let label = format!("{} · {}", function.name, function.routine_type);
                    if sidebar_item(ui, Icon::Code2, &label, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Function(function.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened function {}.{}", function.schema, function.name);
                    }
                }
            },
        );
        if self.table_info.is_none() {
            ui.collapsing(
                icon_text(
                    Icon::Columns3,
                    &format!("Columns ({})", self.schema.columns.len()),
                    self.theme.text_primary,
                ),
                |ui| {
                    for column in self.schema.columns.iter().take(100) {
                        ui.label(
                            RichText::new(format!("  {column}"))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                },
            );
        }
        ui.add_space(16.0);
        if primary_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
            self.editing_connection_id = None;
            self.connection_draft = UiConnectionDraft::default();
            self.connection_error.clear();
            self.connection_dialog_open = true;
        }
    }

    fn draw_history(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Saved queries")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        if self.saved_queries.is_empty() {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(icon_text(Icon::Bookmark, "", self.theme.accent));
                    ui.add_space(6.0);
                    ui.label(RichText::new("No saved queries yet").strong());
                    ui.label(
                        RichText::new("Save a query to keep it close at hand.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(8.0);
                    if compact_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                        self.new_query_document();
                    }
                });
            });
        } else {
            let saved = self.saved_queries.clone();
            let mut groups: Vec<(String, Vec<UiSavedQuerySummary>)> = Vec::new();
            for query in saved {
                let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
                if let Some((_, queries)) = groups.iter_mut().find(|(name, _)| name == &folder) {
                    queries.push(query);
                } else {
                    groups.push((folder, vec![query]));
                }
            }

            for (folder, queries) in groups {
                let folder_id = self
                    .query_folders
                    .iter()
                    .find(|item| item.name == folder)
                    .map(|item| item.id.clone());
                let mut delete_requested = false;
                let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id(("saved-query-folder", folder.as_str())),
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label(icon_text(Icon::FolderOpen, &folder, self.theme.text_primary));
                    ui.label(
                        RichText::new(format!("{} queries", queries.len()))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    if folder_id.is_some() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if compact_icon_button(ui, Icon::X, self.theme)
                                .on_hover_text("Delete folder")
                                .clicked()
                            {
                                delete_requested = true;
                            }
                        });
                    }
                });
                header.body(|ui| {
                    for query in queries {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    false,
                                    icon_text(Icon::FileCode2, &query.name, self.theme.text_secondary),
                                )
                                .clicked()
                            {
                                self.query_text = query.sql.clone();
                                self.active_tab = WorkspaceTab::Query;
                            }
                            if compact_button(ui, "rename", self.theme).clicked() {
                                let request_id = self.task_bridge.next_request_id();
                                let name = if self.query_folder.trim().is_empty() {
                                    format!("{} (renamed)", query.name)
                                } else {
                                    self.query_folder.trim().to_owned()
                                };
                                let _ = self.task_bridge.send(UiCommand::RenameSavedQuery {
                                    request_id,
                                    id: query.id.clone(),
                                    name,
                                });
                            }
                            if compact_button(ui, "delete", self.theme).clicked() {
                                self.delete_confirmation_id = Some(query.id.clone());
                            }
                        });
                    }
                });
                if delete_requested {
                    self.folder_delete_confirmation = folder_id;
                }
            }
            if let Some(id) = self.delete_confirmation_id.clone() {
                ui.colored_label(self.theme.warning, "Delete this saved query?");
                ui.horizontal(|ui| {
                    if compact_button(ui, "Confirm delete", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteSavedQuery { request_id, id });
                        self.delete_confirmation_id = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            }
        }
        ui.separator();
        ui.label(
            RichText::new("Local history")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        if self.query_history.is_empty() {
            ui.label(RichText::new("No queries run yet").color(self.theme.text_muted));
            return;
        }
        for query in self.query_history.iter().rev() {
            let title = query.lines().next().unwrap_or("query");
            ui.vertical(|ui| {
                ui.label(RichText::new(title).color(self.theme.text_secondary));
                ui.label(RichText::new("local history").small().color(self.theme.text_muted));
            });
            ui.add_space(12.0);
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "APPEARANCE", self.theme);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::Sun, "", self.theme.accent));
                ui.label(RichText::new("Light theme").color(self.theme.text_primary));
            });
            ui.label(
                RichText::new("Quiet surfaces, violet focus states, and high-contrast data.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
        ui.add_space(12.0);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATABASE FILES", self.theme);
            ui.add_space(10.0);
            ui.label(
                RichText::new("Backup destination")
                    .small()
                    .color(self.theme.text_secondary),
            );
            input_full_width(
                ui,
                &mut self.backup_output_path,
                "Choose a .sql backup path",
                self.theme,
            );
            ui.horizontal_wrapped(|ui| {
                if compact_button_with_icon(ui, Icon::FolderOpen, "Choose path", self.theme).clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    let _ = self.task_bridge.send(UiCommand::PickBackupFile { request_id });
                }
                if secondary_button_with_icon(ui, Icon::Archive, "Create backup", self.theme).clicked() {
                    if let Some(connection) = self.connections.first() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::Backup {
                            request_id,
                            connection_id: connection.id.clone(),
                            output_path: self.backup_output_path.clone(),
                            custom_format: false,
                        });
                    }
                }
            });
            ui.add_space(14.0);
            ui.label(
                RichText::new("Restore from backup")
                    .small()
                    .color(self.theme.text_secondary),
            );
            input_full_width(ui, &mut self.restore_input_path, "Choose a backup file", self.theme);
            ui.horizontal_wrapped(|ui| {
                if compact_button_with_icon(ui, Icon::FolderOpen, "Choose file", self.theme).clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    let _ = self.task_bridge.send(UiCommand::PickRestoreFile { request_id });
                }
                if secondary_button_with_icon(ui, Icon::RotateCcw, "Restore database", self.theme).clicked() {
                    self.restore_confirmation = true;
                }
            });
            if self.restore_confirmation {
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "Overwrite the active database?");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Confirm restore", self.theme).clicked() {
                        if let Some(connection) = self.connections.first() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::Restore {
                                request_id,
                                connection_id: connection.id.clone(),
                                input_path: self.restore_input_path.clone(),
                                custom_format: false,
                            });
                        }
                        self.restore_confirmation = false;
                    }
                    if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        self.restore_confirmation = false;
                    }
                });
            }
        });
    }

    fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        let toolbar_width = ui.max_rect().width();
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((toolbar_width - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                let welcome = tab_frame(self.theme, self.active_tab == WorkspaceTab::Welcome).show(ui, |ui| {
                    ui.selectable_label(
                        self.active_tab == WorkspaceTab::Welcome,
                        icon_text(Icon::House, "Welcome", self.theme.text_primary),
                    )
                });
                if welcome.inner.clicked() {
                    self.active_tab = WorkspaceTab::Welcome;
                }
                let documents: Vec<(usize, String)> = self
                    .query_documents
                    .iter()
                    .enumerate()
                    .map(|(index, document)| (index, document.title.clone()))
                    .collect();
                for (index, title) in documents {
                    let selected = self.active_tab == WorkspaceTab::Query && self.active_query_document == index;
                    let can_close = self.query_documents.len() > 1;
                    let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        let mut tab_clicked = false;
                        let mut close_clicked = false;
                        ui.horizontal(|ui| {
                            tab_clicked = ui
                                .selectable_label(selected, icon_text(Icon::FileCode2, &title, self.theme.text_primary))
                                .clicked();
                            if can_close {
                                close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                                    .on_hover_text("Close query")
                                    .clicked();
                            }
                        });
                        (tab_clicked, close_clicked)
                    });
                    if tab.inner.0 {
                        self.switch_query_document(index);
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if tab.inner.1 {
                        self.close_query_document(index);
                    }
                }
                if let Some(table_name) = self.selected_table.clone() {
                    let selected = self.active_tab == WorkspaceTab::Table;
                    let table_tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        ui.selectable_label(selected, icon_text(Icon::Table2, &table_name, self.theme.text_primary))
                    });
                    if table_tab.inner.clicked() {
                        self.active_tab = WorkspaceTab::Table;
                    }
                }
                if let Some(selection) = self.selected_schema_object.clone() {
                    let (icon, name) = match selection {
                        SchemaObjectSelection::View(name) => (Icon::Eye, name),
                        SchemaObjectSelection::Trigger(name) => (Icon::Zap, name),
                        SchemaObjectSelection::Function(name) => (Icon::Code2, name),
                    };
                    let selected = self.active_tab == WorkspaceTab::SchemaObject;
                    let object_tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        ui.selectable_label(selected, icon_text(icon, &name, self.theme.text_primary))
                    });
                    if object_tab.inner.clicked() {
                        self.active_tab = WorkspaceTab::SchemaObject;
                    }
                }
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New query")
                    .clicked()
                {
                    self.new_query_document();
                }
            });
        });
        ui.add_space(12.0);
        match self.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => self.draw_diagram(ui),
        }
    }

    fn draw_diagram(&mut self, ui: &mut egui::Ui) {
        let tables = self.schema.table_details.clone();
        let relationship_count: usize = tables.iter().map(|table| table.foreign_keys.len()).sum();
        let visible_tables = tables.len().min(ER_MAX_TABLES);

        ui.horizontal(|ui| {
            section_label(ui, "ER DIAGRAM", self.theme);
            ui.add_space(8.0);
            badge(
                ui,
                &format!("{visible_tables} tables"),
                self.theme.accent_soft,
                self.theme.accent,
            );
            badge(
                ui,
                &format!("{} relationships", relationship_count.min(ER_MAX_EDGES)),
                self.theme.surface_hover,
                self.theme.text_secondary,
            );
            if tables.len() > ER_MAX_TABLES {
                ui.label(
                    RichText::new(format!("+ {} more outside view", tables.len() - ER_MAX_TABLES))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_button(ui, "+", self.theme).clicked() {
                    self.diagram_zoom = (self.diagram_zoom + 0.1).clamp(0.7, 1.5);
                }
                ui.label(
                    RichText::new(format!("{}%", (self.diagram_zoom * 100.0).round() as u16))
                        .small()
                        .color(self.theme.text_secondary),
                );
                if compact_button(ui, "−", self.theme).clicked() {
                    self.diagram_zoom = (self.diagram_zoom - 0.1).clamp(0.7, 1.5);
                }
                if secondary_button_with_icon(ui, Icon::Square, "Fit", self.theme).clicked() {
                    self.diagram_zoom = 1.0;
                    self.diagram_pan = egui::Vec2::ZERO;
                }
            });
        });
        ui.add_space(10.0);

        if tables.is_empty() {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(56.0);
                    ui.label(icon_text(
                        Icon::Workflow,
                        "No schema map yet",
                        self.theme.text_secondary,
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Connect to a database and load its tables to see the relationship map.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(56.0);
                });
            });
            return;
        }

        let grid_columns = visible_tables.clamp(1, 3);
        let grid_rows = visible_tables.div_ceil(grid_columns);
        let max_visible_columns = tables
            .iter()
            .take(ER_MAX_TABLES)
            .map(|table| table.columns.len().clamp(1, ER_MAX_COLUMNS))
            .max()
            .unwrap_or(1);
        let node_height = ER_HEADER_HEIGHT + ER_ROW_HEIGHT * max_visible_columns as f32;
        let canvas_width = (ER_CANVAS_MARGIN * 2.0
            + grid_columns as f32 * ER_NODE_WIDTH
            + (grid_columns.saturating_sub(1)) as f32 * ER_GAP_X)
            * self.diagram_zoom;
        let canvas_height =
            (ER_CANVAS_MARGIN * 2.0 + grid_rows as f32 * node_height + (grid_rows.saturating_sub(1)) as f32 * ER_GAP_Y)
                * self.diagram_zoom;
        let zoom = self.diagram_zoom;
        let pan = self.diagram_pan;
        let theme = self.theme;

        panel_frame(self.theme).show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(canvas_width.max(640.0), canvas_height.max(360.0)),
                    Sense::click_and_drag(),
                );
                let origin = response.rect.min + pan;
                let nodes = tables
                    .iter()
                    .take(ER_MAX_TABLES)
                    .enumerate()
                    .map(|(index, table)| {
                        let column = index % grid_columns;
                        let row = index / grid_columns;
                        let position = origin
                            + egui::vec2(
                                (ER_CANVAS_MARGIN + column as f32 * (ER_NODE_WIDTH + ER_GAP_X)) * zoom,
                                (ER_CANVAS_MARGIN + row as f32 * (node_height + ER_GAP_Y)) * zoom,
                            );
                        ErNode {
                            table: table.clone(),
                            rect: egui::Rect::from_min_size(
                                position,
                                egui::vec2(ER_NODE_WIDTH * zoom, node_height * zoom),
                            ),
                        }
                    })
                    .collect::<Vec<_>>();

                let mut drawn_edges = 0;
                for source_node in &nodes {
                    for foreign_key in &source_node.table.foreign_keys {
                        if drawn_edges >= ER_MAX_EDGES {
                            break;
                        }
                        let Some(target_node) = nodes.iter().find(|node| {
                            node.table.schema == foreign_key.to_schema && node.table.name == foreign_key.to_table
                        }) else {
                            continue;
                        };
                        let source_on_right = source_node.rect.center().x < target_node.rect.center().x;
                        let from = er_column_anchor(
                            source_node,
                            foreign_key.from_columns.first().map(String::as_str),
                            source_on_right,
                            zoom,
                        );
                        let to = er_column_anchor(
                            target_node,
                            foreign_key.to_columns.first().map(String::as_str),
                            !source_on_right,
                            zoom,
                        );
                        let bend_x = (from.x + to.x) / 2.0;
                        let bend_a = egui::pos2(bend_x, from.y);
                        let bend_b = egui::pos2(bend_x, to.y);
                        let stroke = egui::Stroke::new(1.2, theme.accent);
                        painter.line_segment([from, bend_a], stroke);
                        painter.line_segment([bend_a, bend_b], stroke);
                        painter.line_segment([bend_b, to], stroke);
                        let label = format!(
                            "{} → {}",
                            foreign_key.from_columns.first().map(String::as_str).unwrap_or("key"),
                            foreign_key.to_columns.first().map(String::as_str).unwrap_or("key"),
                        );
                        let label_position = egui::pos2(bend_x, (from.y + to.y) / 2.0);
                        let label_galley =
                            painter.layout_no_wrap(label.clone(), FontId::proportional(10.0), theme.text_secondary);
                        let label_rect =
                            egui::Rect::from_center_size(label_position, label_galley.size() + egui::vec2(8.0, 4.0));
                        painter.rect_filled(label_rect, egui::Rounding::same(3.0), theme.surface_panel);
                        painter.text(
                            label_position,
                            egui::Align2::CENTER_CENTER,
                            label,
                            FontId::proportional(10.0),
                            theme.text_secondary,
                        );
                        drawn_edges += 1;
                    }
                }

                for node in &nodes {
                    let selected = self.selected_table.as_deref() == Some(node.table.name.as_str());
                    painter.rect_filled(node.rect, egui::Rounding::same(8.0), theme.surface_panel);
                    painter.rect_stroke(
                        node.rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(1.0, if selected { theme.accent } else { theme.border_default }),
                    );
                    let header_rect = egui::Rect::from_min_max(
                        node.rect.min,
                        egui::pos2(node.rect.max.x, node.rect.min.y + ER_HEADER_HEIGHT * zoom),
                    );
                    painter.rect_filled(
                        header_rect,
                        egui::Rounding::same(8.0),
                        if selected {
                            theme.accent_soft
                        } else {
                            theme.surface_hover
                        },
                    );
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(header_rect.min.x, header_rect.max.y - 8.0 * zoom),
                            header_rect.max,
                        ),
                        egui::Rounding::ZERO,
                        if selected {
                            theme.accent_soft
                        } else {
                            theme.surface_hover
                        },
                    );
                    painter.text(
                        node.rect.min + egui::vec2(14.0 * zoom, 20.0 * zoom),
                        egui::Align2::LEFT_CENTER,
                        format!("{}.{}", node.table.schema, node.table.name),
                        FontId::proportional(13.0 * zoom),
                        theme.text_primary,
                    );
                    painter.text(
                        egui::pos2(node.rect.max.x - 12.0 * zoom, node.rect.min.y + 20.0 * zoom),
                        egui::Align2::RIGHT_CENTER,
                        "TABLE",
                        FontId::proportional(9.0 * zoom),
                        theme.text_muted,
                    );

                    for (index, column) in node.table.columns.iter().take(ER_MAX_COLUMNS).enumerate() {
                        let row_top = node.rect.min.y + (ER_HEADER_HEIGHT + index as f32 * ER_ROW_HEIGHT) * zoom;
                        let row_rect = egui::Rect::from_min_max(
                            egui::pos2(node.rect.min.x, row_top),
                            egui::pos2(node.rect.max.x, row_top + ER_ROW_HEIGHT * zoom),
                        );
                        if index % 2 == 0 {
                            painter.rect_filled(row_rect, egui::Rounding::ZERO, theme.surface_app);
                        }
                        let is_foreign_key = node
                            .table
                            .foreign_keys
                            .iter()
                            .any(|foreign_key| foreign_key.from_columns.iter().any(|name| name == &column.name));
                        let marker_color = if column.is_primary_key {
                            theme.warning
                        } else if is_foreign_key {
                            theme.accent
                        } else {
                            theme.border_strong
                        };
                        painter.circle_filled(
                            egui::pos2(row_rect.min.x + 13.0 * zoom, row_rect.center().y),
                            3.0 * zoom,
                            marker_color,
                        );
                        painter.text(
                            egui::pos2(row_rect.min.x + 23.0 * zoom, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &column.name,
                            FontId::proportional(11.0 * zoom),
                            theme.text_primary,
                        );
                        painter.text(
                            egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            &column.data_type,
                            FontId::monospace(10.0 * zoom),
                            theme.text_secondary,
                        );
                    }
                    if node.table.columns.len() > ER_MAX_COLUMNS {
                        painter.text(
                            egui::pos2(
                                node.rect.min.x + 14.0 * zoom,
                                node.rect.min.y + (ER_HEADER_HEIGHT + ER_ROW_HEIGHT * 7.5) * zoom,
                            ),
                            egui::Align2::LEFT_CENTER,
                            format!("+ {} more columns", node.table.columns.len() - ER_MAX_COLUMNS),
                            FontId::proportional(10.0 * zoom),
                            theme.text_muted,
                        );
                    }
                }

                if response.drag_started() {
                    self.diagram_pan_origin = Some(self.diagram_pan);
                }
                if response.dragged() {
                    if let Some(origin) = self.diagram_pan_origin {
                        self.diagram_pan = origin + response.drag_delta();
                    }
                }
                if response.drag_stopped() {
                    self.diagram_pan_origin = None;
                }

                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        if let Some(node) = nodes.iter().find(|node| node.rect.contains(pointer)) {
                            self.selected_table = Some(node.table.name.clone());
                            self.selected_schema_object = None;
                            self.schema_object_view = SchemaObjectView::Definition;
                            self.table_info = None;
                            self.table_ddl = None;
                            self.table_info_error = None;
                            self.table_ddl_error = None;
                            self.ddl_execute_confirmation = false;
                            self.ddl_execution_request = None;
                            self.table_data_result = None;
                            self.table_data_total_rows = None;
                            self.table_data_offset = 0;
                            self.table_data_filter_column.clear();
                            self.table_data_filter_value.clear();
                            self.table_data_sort_column = None;
                            self.table_data_sort_desc = false;
                            self.table_data_error = None;
                            self.table_info_request = None;
                            self.table_ddl_request = None;
                            self.table_data_request = None;
                            self.selected_cell = None;
                            self.selected_row = None;
                            self.data_editing_cell = None;
                            self.data_edit_value.clear();
                            self.data_delete_confirmation = false;
                            self.table_view = TableView::Structure;
                            self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", node.table.name);
                            self.request_table_info();
                            self.activity = Activity::Explorer;
                            self.sidebar_open = true;
                            self.active_tab = WorkspaceTab::Table;
                        }
                    }
                }
            });
        });
    }

    fn draw_schema_object_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(selection) = self.selected_schema_object.clone() else {
            self.active_tab = WorkspaceTab::Welcome;
            return;
        };
        let is_view = matches!(selection, SchemaObjectSelection::View(_));

        let (icon, kind, name, schema, definition, metadata, query) = match selection {
            SchemaObjectSelection::View(name) => {
                let Some(view) = self.schema.views.iter().find(|view| view.name == name).cloned() else {
                    return;
                };
                (
                    Icon::Eye,
                    "VIEW".to_owned(),
                    view.name.clone(),
                    view.schema.clone(),
                    view.definition.clone(),
                    None,
                    format!("SELECT *\nFROM \"{}\".\"{}\"\nLIMIT 100;", view.schema, view.name),
                )
            }
            SchemaObjectSelection::Trigger(name) => {
                let Some(trigger) = self
                    .schema
                    .triggers
                    .iter()
                    .find(|trigger| trigger.name == name)
                    .cloned()
                else {
                    return;
                };
                (
                    Icon::Zap,
                    "TRIGGER".to_owned(),
                    trigger.name.clone(),
                    trigger.schema.clone(),
                    trigger.definition.clone(),
                    Some(format!(
                        "{} · {} · {}",
                        trigger.table_name, trigger.timing, trigger.event
                    )),
                    trigger.definition.clone(),
                )
            }
            SchemaObjectSelection::Function(name) => {
                let Some(function) = self
                    .schema
                    .functions
                    .iter()
                    .find(|function| function.name == name)
                    .cloned()
                else {
                    return;
                };
                (
                    Icon::Code2,
                    function.routine_type,
                    function.name.clone(),
                    function.schema.clone(),
                    function.definition.clone(),
                    Some(format!("returns {}", function.data_type)),
                    function.definition.clone(),
                )
            }
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(icon, &kind, self.theme.text_primary));
                ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
                ui.label(
                    RichText::new(format!("{schema}.{name}"))
                        .strong()
                        .color(self.theme.accent),
                );
                if let Some(metadata) = metadata.as_deref() {
                    badge(ui, metadata, self.theme.surface_active, self.theme.text_secondary);
                }
                if is_view {
                    for (view, icon, label) in [
                        (SchemaObjectView::Definition, Icon::Code2, "Definition"),
                        (SchemaObjectView::Data, Icon::Table2, "Data"),
                    ] {
                        let selected = self.schema_object_view == view;
                        let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                            ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
                        });
                        if tab.inner.clicked() {
                            self.schema_object_view = view;
                            if view == SchemaObjectView::Data {
                                self.table_data_result = None;
                                self.table_data_total_rows = None;
                                self.table_data_error = None;
                                self.table_data_offset = 0;
                                self.request_table_data();
                            }
                        }
                    }
                }
                if secondary_button_with_icon(ui, Icon::FileCode2, "Open in Query", self.theme).clicked() {
                    self.query_text = query;
                    self.active_tab = WorkspaceTab::Query;
                }
            });
        });
        ui.add_space(12.0);
        if is_view && self.schema_object_view == SchemaObjectView::Data {
            if self.table_data_result.is_none() && self.table_data_request.is_none() && self.table_data_error.is_none()
            {
                self.request_table_data();
            }
            self.draw_table_data(ui, &name);
        } else {
            self.draw_schema_definition(ui, &kind, &definition);
        }
    }

    fn draw_schema_definition(&self, ui: &mut egui::Ui, kind: &str, definition: &str) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, format!("{kind} DEFINITION"), self.theme);
            ui.add_space(8.0);
            editor_frame(self.theme).show(ui, |ui| {
                let mut script = definition.to_owned();
                ui.add(
                    TextEdit::multiline(&mut script)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(18)
                        .interactive(false),
                );
            });
        });
    }

    fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(56.0);
            card_frame(self.theme).show(ui, |ui| {
                ui.set_max_width(680.0);
                section_label(ui, "DATABASE WORKSPACE", self.theme);
                ui.add_space(12.0);
                ui.label(RichText::new("A calmer way to work with databases").size(26.0).strong());
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Connect, explore, and query with confidence.").color(self.theme.text_secondary),
                );
                ui.add_space(22.0);
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                        self.new_query_document();
                    }
                    if secondary_button_with_icon(ui, Icon::Database, "New connection", self.theme).clicked() {
                        self.editing_connection_id = None;
                        self.connection_draft = UiConnectionDraft::default();
                        self.connection_error.clear();
                        self.connection_dialog_open = true;
                    }
                });
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Command, "", self.theme.accent));
                    ui.label(
                        RichText::new("⌘ P to open anything")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.separator();
                    ui.label(icon_text(Icon::PanelLeft, "", self.theme.accent));
                    ui.label(
                        RichText::new("⌘ B to toggle explorer")
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        });
    }

    fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.selected_table.clone() else {
            self.active_tab = WorkspaceTab::Welcome;
            return;
        };
        if self.table_view == TableView::Ddl && self.table_ddl.is_none() && self.table_ddl_request.is_none() {
            self.request_table_ddl();
        }
        if self.table_view == TableView::Data
            && self.table_info.is_some()
            && self.table_data_result.is_none()
            && self.table_data_request.is_none()
            && self.table_data_error.is_none()
        {
            self.request_table_data();
        }

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(Icon::Table2, "Table", self.theme.text_primary));
                ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
                ui.label(
                    RichText::new(format!("{}.{table_name}", self.active_schema()))
                        .strong()
                        .color(self.theme.accent),
                );
                ui.separator();
                for (view, icon, label) in [
                    (TableView::Structure, Icon::Columns3, "Structure"),
                    (TableView::Data, Icon::Table2, "Data"),
                    (TableView::Ddl, Icon::Code2, "DDL"),
                ] {
                    let selected = self.table_view == view;
                    let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
                    });
                    if tab.inner.clicked() {
                        self.table_view = view;
                    }
                }
                if secondary_button_with_icon(ui, Icon::FileCode2, "Open in Query", self.theme).clicked() {
                    self.active_tab = WorkspaceTab::Query;
                }
            });
        });
        ui.add_space(12.0);

        match self.table_view {
            TableView::Structure => self.draw_table_structure(ui),
            TableView::Data => self.draw_table_data(ui, &table_name),
            TableView::Ddl => self.draw_table_ddl(ui, &table_name),
        }
    }

    fn draw_table_structure(&self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_info_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::LoaderCircle
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            "Table structure could not be loaded"
                        } else {
                            "Loading table structure…"
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    ui.label(
                        RichText::new(failed.unwrap_or("Columns, keys and indexes will appear here."))
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    ui.add_space(28.0);
                });
            });
            return;
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                badge(
                    ui,
                    &format!("{} columns", info.columns.len()),
                    self.theme.accent_soft,
                    self.theme.accent,
                );
                badge(
                    ui,
                    &format!("{} indexes", info.indexes.len()),
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                badge(
                    ui,
                    &format!("{} foreign keys", info.foreign_keys.len()),
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                if let Some(row_count) = info.row_count {
                    badge(
                        ui,
                        &format!("{row_count} rows"),
                        self.theme.surface_active,
                        self.theme.text_secondary,
                    );
                }
            });
        });
        ui.add_space(10.0);

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, "COLUMNS", self.theme);
            ui.add_space(8.0);
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                egui::Grid::new("table-structure-columns")
                    .num_columns(5)
                    .spacing([18.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for header in ["Column", "Type", "Nullable", "Key", "Default"] {
                            ui.label(RichText::new(header).small().strong().color(self.theme.text_secondary));
                        }
                        ui.end_row();
                        for column in &info.columns {
                            ui.label(RichText::new(&column.name).color(self.theme.text_primary));
                            ui.label(
                                RichText::new(&column.data_type)
                                    .monospace()
                                    .color(self.theme.code_keyword),
                            );
                            ui.label(
                                RichText::new(if column.nullable { "yes" } else { "no" })
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            if column.is_primary_key {
                                ui.label(icon_text(Icon::KeyRound, "PK", self.theme.warning));
                            } else {
                                ui.label(RichText::new("—").small().color(self.theme.text_muted));
                            }
                            ui.label(
                                RichText::new(column.default.as_deref().unwrap_or("—"))
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            ui.end_row();
                        }
                    });
            });
        });
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            card_frame(self.theme).show(&mut columns[0], |ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(6.0);
                if info.indexes.is_empty() {
                    ui.label(RichText::new("No indexes").small().color(self.theme.text_muted));
                }
                for index in &info.indexes {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(icon_text(
                            if index.unique { Icon::BadgeCheck } else { Icon::List },
                            "",
                            self.theme.accent,
                        ));
                        ui.label(RichText::new(&index.name).color(self.theme.text_primary));
                        ui.label(
                            RichText::new(index.columns.join(", "))
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    });
                }
            });
            card_frame(self.theme).show(&mut columns[1], |ui| {
                section_label(ui, "FOREIGN KEYS", self.theme);
                ui.add_space(6.0);
                if info.foreign_keys.is_empty() {
                    ui.label(RichText::new("No foreign keys").small().color(self.theme.text_muted));
                }
                for foreign_key in &info.foreign_keys {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                        ui.label(RichText::new(&foreign_key.name).color(self.theme.text_primary));
                        ui.label(
                            RichText::new(format!(
                                "{} → {}.{}",
                                foreign_key.from_columns.join(", "),
                                foreign_key.to_schema,
                                foreign_key.to_table,
                            ))
                            .small()
                            .color(self.theme.text_secondary),
                        );
                    });
                }
            });
        });
    }

    fn draw_table_data(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(result) = self.table_data_result.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_data_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::LoaderCircle
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            format!("Data for {table_name} could not be loaded")
                        } else {
                            format!("Loading data for {table_name}…")
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    if let Some(error) = failed {
                        ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                        ui.add_space(12.0);
                        if secondary_button_with_icon(ui, Icon::RotateCcw, "Retry", self.theme).clicked() {
                            self.table_data_error = None;
                            self.request_table_data();
                        }
                    } else {
                        ui.label(
                            RichText::new("Rows will appear here with the shared result-grid controls.")
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    }
                    ui.add_space(28.0);
                });
            });
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_rows = self.table_data_total_rows.unwrap_or(result.row_count);
        let page = self.table_data_offset / TABLE_PAGE_SIZE + 1;
        let total_pages = total_rows.div_ceil(TABLE_PAGE_SIZE).max(1);
        let page_range = if total_rows > 0 {
            let start = self.table_data_offset + 1;
            let end = (self.table_data_offset + result.row_count).min(total_rows);
            format!("{start}–{end} of {total_rows}")
        } else {
            "0 rows".to_owned()
        };
        let has_next = self.table_data_offset.saturating_add(TABLE_PAGE_SIZE) < total_rows;
        let has_previous = self.table_data_offset > 0;
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(Icon::Table2, "DATA EDITOR", self.theme.text_primary));
                badge(ui, table_name, self.theme.accent_soft, self.theme.accent);
                ui.label(RichText::new(page_range).small().color(self.theme.text_muted));
                ui.separator();
                if compact_button_with_icon(ui, Icon::RotateCcw, "Refresh", self.theme)
                    .on_hover_text("Reload the current page")
                    .clicked()
                {
                    self.table_data_result = None;
                    self.table_data_total_rows = None;
                    self.table_data_error = None;
                    self.selected_cell = None;
                    self.selected_row = None;
                    self.data_delete_confirmation = false;
                    self.request_table_data();
                }
                if can_mutate {
                    if compact_button_with_icon(ui, Icon::Plus, "New row", self.theme).clicked() {
                        self.open_insert_row();
                    }
                    if compact_button_with_icon(ui, Icon::Pencil, "Edit cell", self.theme).clicked() {
                        if let Some((row_index, column_index)) = self.selected_cell {
                            if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                                self.begin_data_cell_edit(row_index, column_index, cell);
                            }
                        } else {
                            self.runtime_message = "Select a cell before editing".to_owned();
                        }
                    }
                    if compact_button_with_icon(ui, Icon::Trash2, "Delete row", self.theme).clicked() {
                        self.request_delete_selected_data_row(&result);
                    }
                    if self.data_delete_confirmation {
                        ui.colored_label(self.theme.warning, "Delete selected row?");
                        if danger_button(ui, "Confirm", self.theme).clicked() {
                            self.submit_delete_selected_data_row(&result);
                        }
                        if compact_button(ui, "Cancel", self.theme).clicked() {
                            self.data_delete_confirmation = false;
                        }
                    }
                } else if self.connected {
                    ui.label(RichText::new("Read-only connection").small().color(self.theme.warning));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if compact_icon_button_enabled(ui, Icon::ChevronRight, has_next, self.theme)
                        .on_hover_text("Next page")
                        .clicked()
                    {
                        self.table_data_result = None;
                        self.table_data_error = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_delete_confirmation = false;
                        self.table_data_offset = self.table_data_offset.saturating_add(TABLE_PAGE_SIZE);
                        self.request_table_data();
                    }
                    if compact_icon_button_enabled(ui, Icon::ChevronLeft, has_previous, self.theme)
                        .on_hover_text("Previous page")
                        .clicked()
                    {
                        self.table_data_result = None;
                        self.table_data_error = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_delete_confirmation = false;
                        self.table_data_offset = self.table_data_offset.saturating_sub(TABLE_PAGE_SIZE);
                        self.request_table_data();
                    }
                    ui.label(
                        RichText::new(format!("Page {page} of {total_pages}"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        });
        ui.add_space(8.0);
        let column_names = result
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<Vec<_>>();
        let requires_order = !self.active_driver().eq_ignore_ascii_case("sqlite");
        if !column_names.is_empty() {
            toolbar_frame(self.theme).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    section_label(ui, "DATABASE FILTER", self.theme);
                    ui.label(RichText::new("Column").small().color(self.theme.text_muted));
                    egui::ComboBox::from_id_salt(("table-data-filter-column", table_name))
                        .selected_text(if self.table_data_filter_column.is_empty() {
                            "Column"
                        } else {
                            self.table_data_filter_column.as_str()
                        })
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            for column in &column_names {
                                ui.selectable_value(
                                    &mut self.table_data_filter_column,
                                    column.clone(),
                                    column.as_str(),
                                );
                            }
                        });
                    input(ui, &mut self.table_data_filter_value, "contains…", 180.0, self.theme);
                    if compact_button_with_icon(ui, Icon::Filter, "Apply", self.theme).clicked() {
                        self.reload_table_data_from_start();
                    }
                    if compact_button_with_icon(ui, Icon::FilterX, "Clear", self.theme).clicked() {
                        self.table_data_filter_value.clear();
                        self.reload_table_data_from_start();
                    }
                    ui.separator();
                    section_label(ui, "ORDER BY", self.theme);
                    egui::ComboBox::from_id_salt(("table-data-sort-column", table_name))
                        .selected_text(self.table_data_sort_column.as_deref().unwrap_or("None"))
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            if !requires_order {
                                ui.selectable_value(&mut self.table_data_sort_column, None, "None");
                            }
                            for column in &column_names {
                                ui.selectable_value(
                                    &mut self.table_data_sort_column,
                                    Some(column.clone()),
                                    column.as_str(),
                                );
                            }
                        });
                    if self.table_data_sort_column.is_some() {
                        let direction = if self.table_data_sort_desc { "DESC" } else { "ASC" };
                        if compact_button_with_icon(ui, Icon::ArrowDownUp, direction, self.theme).clicked() {
                            self.table_data_sort_desc = !self.table_data_sort_desc;
                            self.reload_table_data_from_start();
                        }
                    }
                });
            });
            ui.add_space(8.0);
        }
        let data_width = ui.max_rect().width();
        panel_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((data_width - 24.0).max(0.0));
            self.draw_result_grid(ui, &result);
        });
    }

    fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.insert_row_values = vec![String::new(); info.columns.len()];
        self.insert_row_error.clear();
        self.insert_row_open = true;
    }

    fn parse_insert_value(raw: &str, data_type: &str) -> Result<Option<UiCell>, String> {
        let value = raw.trim();
        if value.is_empty() {
            return Ok(None);
        }
        if value.eq_ignore_ascii_case("null") {
            return Ok(Some(UiCell::Null));
        }
        let normalized_type = data_type.to_ascii_lowercase();
        if normalized_type.contains("bool") {
            return match value.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Some(UiCell::Boolean(true))),
                "false" | "0" | "no" => Ok(Some(UiCell::Boolean(false))),
                _ => Err("expected true or false".to_owned()),
            };
        }
        if normalized_type.contains("int") || normalized_type.contains("serial") {
            return value
                .parse::<i64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid integer"));
        }
        if normalized_type.contains("real") || normalized_type.contains("float") || normalized_type.contains("double") {
            return value
                .parse::<f64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid floating-point number"));
        }
        if Self::is_decimal_type(&normalized_type) {
            return Self::parse_decimal_value(value, data_type).map(Some);
        }
        if normalized_type.contains("json") {
            return serde_json::from_str::<serde_json::Value>(value)
                .map(|_| Some(UiCell::Json(value.to_owned())))
                .map_err(|_| "expected valid JSON".to_owned());
        }
        Ok(Some(UiCell::Text(value.to_owned())))
    }

    fn parse_update_value(raw: &str, data_type: &str) -> Result<UiCell, String> {
        if raw.trim().is_empty() {
            if Self::is_decimal_type(&data_type.to_ascii_lowercase()) {
                return Err("enter a decimal value or the literal NULL".to_owned());
            }
            return Ok(UiCell::Text(String::new()));
        }
        Self::parse_insert_value(raw, data_type).map(|value| value.unwrap_or_else(|| UiCell::Text(String::new())))
    }

    fn is_decimal_type(normalized_type: &str) -> bool {
        normalized_type
            .split('(')
            .next()
            .is_some_and(|name| matches!(name.trim(), "numeric" | "decimal"))
    }

    fn decimal_constraints(data_type: &str) -> Option<(u64, i64)> {
        let normalized_type = data_type.to_ascii_lowercase();
        if !Self::is_decimal_type(&normalized_type) {
            return None;
        }
        let arguments = normalized_type.split_once('(')?.1.split_once(')')?.0;
        let mut parts = arguments.split(',').map(str::trim);
        let precision = parts.next()?.parse::<u64>().ok()?;
        let scale = parts.next().and_then(|part| part.parse::<i64>().ok()).unwrap_or(0);
        Some((precision, scale))
    }

    fn parse_decimal_value(value: &str, data_type: &str) -> Result<UiCell, String> {
        let decimal = value
            .parse::<BigDecimal>()
            .map_err(|_| format!("{value} is not a valid exact decimal"))?;
        let actual_scale = decimal.fractional_digit_count();
        if let Some((precision, declared_scale)) = Self::decimal_constraints(data_type) {
            if actual_scale > declared_scale {
                return Err(format!("{value} has more than {declared_scale} fractional digits"));
            }
            let effective_precision = if actual_scale < 0 {
                decimal.digits().saturating_add((-actual_scale) as u64)
            } else {
                decimal.digits()
            };
            if effective_precision > precision {
                return Err(format!("{value} exceeds NUMERIC precision {precision}"));
            }
        }
        Ok(UiCell::Text(value.to_owned()))
    }

    fn submit_insert_row(&mut self) {
        let Some(table) = self.selected_table.clone() else {
            self.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for (column, raw) in info.columns.iter().zip(&self.insert_row_values) {
            match Self::parse_insert_value(raw, &column.data_type) {
                Ok(Some(value)) => {
                    columns.push(column.name.clone());
                    values.push(value);
                }
                Ok(None) => {}
                Err(error) => {
                    self.insert_row_error = format!("{}: {error}", column.name);
                    return;
                }
            }
        }
        if columns.is_empty() {
            self.insert_row_error = "Enter at least one value; leave defaulted columns empty".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.insert_row_error = "Connect to a database before inserting a row".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::InsertTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            columns,
            values,
        });
        self.table_mutation_request = Some(request_id);
        self.insert_row_open = false;
        self.insert_row_error.clear();
        self.runtime_message = "Inserting row…".to_owned();
    }

    fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table_info.clone() else {
            self.insert_row_open = false;
            return;
        };
        if self.insert_row_values.len() != info.columns.len() {
            self.insert_row_values = vec![String::new(); info.columns.len()];
        }
        let mut open = self.insert_row_open;
        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("Insert row")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Add a row to {}.{}", info.schema, info.name))
                        .color(self.theme.text_secondary),
                );
                ui.label(
                    RichText::new("Empty fields use the database default. Type NULL for a null value.")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.add_space(10.0);
                for (index, column) in info.columns.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [108.0, 24.0],
                            egui::Label::new(
                                RichText::new(format!("{} · {}", column.name, column.data_type))
                                    .small()
                                    .strong()
                                    .color(self.theme.text_secondary),
                            ),
                        );
                        input(
                            ui,
                            &mut self.insert_row_values[index],
                            &column.data_type,
                            300.0,
                            self.theme,
                        );
                    });
                }
                if !self.insert_row_error.is_empty() {
                    ui.add_space(6.0);
                    ui.colored_label(self.theme.danger, self.insert_row_error.as_str());
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Plus, "Insert row", self.theme).clicked() {
                        submit = true;
                    }
                    if ghost_button(ui, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
        if submit {
            self.submit_insert_row();
        }
        if cancel || !open {
            self.insert_row_open = false;
            self.insert_row_error.clear();
        }
    }

    fn submit_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to execute DDL".to_owned();
            return;
        }
        if self.ddl_execution_request.is_some() {
            return;
        }
        let Some(sql) = self.table_ddl.clone() else {
            self.runtime_message = "Load the table DDL before executing it".to_owned();
            return;
        };
        if sql.trim().is_empty() {
            self.runtime_message = "DDL cannot be empty".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.runtime_message = "Connect to a database before executing DDL".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::ExecuteDdl {
            request_id,
            connection_id: connection.id,
            sql,
        });
        self.ddl_execution_request = Some(request_id);
        self.ddl_execute_confirmation = false;
        self.runtime_message = "Executing DDL…".to_owned();
    }

    fn draw_table_ddl(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(mut ddl) = self.table_ddl.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_ddl_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::Code2
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            format!("DDL for {table_name} could not be loaded")
                        } else {
                            format!("Loading DDL for {table_name}…")
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    if let Some(error) = failed {
                        ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                    }
                    ui.add_space(28.0);
                });
            });
            return;
        };

        let writable = self.can_mutate_active_connection();
        let mut request_execution = false;
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "CREATE SCRIPT", self.theme);
                ui.label(
                    RichText::new(if writable {
                        "Editable preview · execution is confirmation-gated"
                    } else {
                        "Read-only preview"
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
                if writable
                    && self.ddl_execution_request.is_none()
                    && primary_button_with_icon(ui, Icon::Play, "Apply DDL", self.theme).clicked()
                {
                    request_execution = true;
                }
            });
            ui.add_space(8.0);
            if let Some(error) = self.table_ddl_error.as_deref() {
                ui.label(
                    RichText::new(format!("DDL execution failed · {error}"))
                        .small()
                        .color(self.theme.danger),
                );
                ui.add_space(6.0);
            }
            editor_frame(self.theme).show(ui, |ui| {
                let response = ui.add(
                    TextEdit::multiline(&mut ddl)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(18)
                        .interactive(writable),
                );
                if response.changed() {
                    self.table_ddl_error = None;
                }
            });
        });
        self.table_ddl = Some(ddl);
        if request_execution {
            self.ddl_execute_confirmation = true;
        }
        if self.ddl_execute_confirmation {
            let mut execute = false;
            let mut cancel = false;
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(icon_text(
                        Icon::TriangleAlert,
                        "Review DDL before applying",
                        self.theme.warning,
                    ));
                    ui.label(
                        RichText::new("This changes the connected database and refreshes the Explorer.")
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    if primary_button_with_icon(ui, Icon::Check, "Execute", self.theme).clicked() {
                        execute = true;
                    }
                    if ghost_button(ui, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
            if execute {
                self.submit_ddl();
            }
            if cancel {
                self.ddl_execute_confirmation = false;
            }
        }
    }

    fn sql_layouter(ui: &egui::Ui, text: &str, wrap_width: f32, theme: DbProTheme) -> Arc<egui::Galley> {
        let keywords = [
            "select",
            "from",
            "where",
            "and",
            "or",
            "join",
            "left",
            "right",
            "inner",
            "group",
            "by",
            "order",
            "limit",
            "offset",
            "insert",
            "into",
            "values",
            "update",
            "set",
            "delete",
            "create",
            "table",
            "alter",
            "drop",
            "as",
            "on",
            "is",
            "null",
            "not",
            "returning",
            "with",
            "explain",
        ];
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let mut current = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let flush = |job: &mut LayoutJob, value: &mut String, color: Color32| {
            if !value.is_empty() {
                job.append(
                    value,
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color,
                        ..Default::default()
                    },
                );
                value.clear();
            }
        };
        let chars: Vec<char> = text.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            if !in_string && !in_comment && ch == '-' && chars.get(index + 1) == Some(&'-') {
                flush(&mut job, &mut current, theme.text_secondary);
                in_comment = true;
                current.push(ch);
            } else if in_comment {
                current.push(ch);
                if ch == '\n' {
                    flush(&mut job, &mut current, theme.code_comment);
                    in_comment = false;
                }
            } else if ch == '\'' {
                current.push(ch);
                if in_string {
                    flush(&mut job, &mut current, theme.code_string);
                    in_string = false;
                } else {
                    flush(&mut job, &mut current, theme.code_string);
                    in_string = true;
                }
            } else if in_string || ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
            } else {
                let word = current.to_lowercase();
                let color = if keywords.contains(&word.as_str()) {
                    theme.code_keyword
                } else if current.chars().all(|value| value.is_ascii_digit()) && !current.is_empty() {
                    theme.code_number
                } else {
                    theme.text_primary
                };
                flush(&mut job, &mut current, color);
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: theme.text_primary,
                        ..Default::default()
                    },
                );
            }
            index += 1;
        }
        if in_string {
            flush(&mut job, &mut current, theme.code_string);
        } else if in_comment {
            flush(&mut job, &mut current, theme.code_comment);
        } else {
            let word = current.to_lowercase();
            let color = if keywords.contains(&word.as_str()) {
                theme.code_keyword
            } else {
                theme.text_primary
            };
            flush(&mut job, &mut current, color);
        }
        ui.fonts(|fonts| fonts.layout_job(job))
    }

    fn format_sql(sql: &str) -> String {
        let keywords = [
            "select", "from", "where", "group by", "order by", "limit", "values", "set",
        ];
        let mut formatted = sql.trim().to_owned();
        for keyword in keywords {
            formatted = formatted.replace(keyword, &keyword.to_uppercase());
        }
        formatted = formatted
            .replace(" FROM ", "\nFROM ")
            .replace(" WHERE ", "\nWHERE ")
            .replace(" GROUP BY ", "\nGROUP BY ")
            .replace(" ORDER BY ", "\nORDER BY ")
            .replace(" LIMIT ", "\nLIMIT ");
        formatted
    }

    fn parse_sql_diagnostics(sql: &str, driver: &str) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let parse_result = if driver.eq_ignore_ascii_case("sqlite") {
            Parser::parse_sql(&SQLiteDialect {}, sql)
        } else if driver.eq_ignore_ascii_case("postgres") {
            Parser::parse_sql(&PostgreSqlDialect {}, sql)
        } else {
            Parser::parse_sql(&GenericDialect {}, sql)
        };
        if let Err(error) = parse_result {
            diagnostics.push(format!("SQL parser: {error}"));
        }
        if sql.trim().is_empty() {
            diagnostics.push("Query is empty".to_owned());
            return diagnostics;
        }
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut parentheses = 0i32;
        for ch in sql.chars() {
            if ch == '\'' {
                in_string = !in_string;
                current.push(ch);
            } else if in_string {
                current.push(ch);
            } else if ch == '(' {
                parentheses += 1;
                tokens.push(current.to_lowercase());
                current.clear();
            } else if ch == ')' {
                parentheses -= 1;
                tokens.push(current.to_lowercase());
                current.clear();
                if parentheses < 0 {
                    diagnostics.push("Unexpected closing parenthesis".to_owned());
                    parentheses = 0;
                }
            } else if ch.is_whitespace() || ch == ';' || ch == ',' {
                if !current.is_empty() {
                    tokens.push(current.to_lowercase());
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            tokens.push(current.to_lowercase());
        }
        if in_string {
            diagnostics.push("Unclosed string literal".to_owned());
        }
        if parentheses > 0 {
            diagnostics.push("Unclosed parenthesis".to_owned());
        }
        if tokens.first().map(String::as_str) == Some("select") && !tokens.iter().any(|token| token == "from") {
            diagnostics.push("SELECT statement is missing FROM".to_owned());
        }
        if tokens.first().map(String::as_str) == Some("update") && !tokens.iter().any(|token| token == "where") {
            diagnostics.push("UPDATE without WHERE will affect every row".to_owned());
        }
        let lower = sql.to_lowercase();
        if driver.eq_ignore_ascii_case("sqlite") && tokens.iter().any(|token| token == "ilike") {
            diagnostics.push("SQLite does not support ILIKE; use LIKE or lower()".to_owned());
        }
        if driver.eq_ignore_ascii_case("postgres") && tokens.iter().any(|token| token == "glob") {
            diagnostics.push("GLOB is SQLite-specific; use LIKE for PostgreSQL".to_owned());
        }
        if lower.contains("select * from") && lower.contains("select * from select") {
            diagnostics.push("Subquery must be enclosed in parentheses".to_owned());
        }
        diagnostics
    }

    fn refresh_diagnostics(&mut self) {
        let driver = self.active_driver();
        self.diagnostics = Self::parse_sql_diagnostics(&self.query_text, driver);
    }

    fn insert_snippet(&mut self, snippet: &str) {
        if !self.query_text.trim().is_empty() {
            self.query_text.push_str("\n\n");
        }
        self.query_text.push_str(snippet);
    }

    fn draw_query(&mut self, ui: &mut egui::Ui) {
        self.refresh_diagnostics();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Query").strong().color(self.theme.text_primary));
            ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
            ui.label(RichText::new(self.active_connection_name()).color(self.theme.accent));
            let running = self.next_query_request.is_some();
            let run_button = if running {
                secondary_button_with_icon(ui, Icon::Square, "Stop  Esc", self.theme)
            } else {
                primary_button_with_icon(ui, Icon::Play, "Run  ⌘↵", self.theme)
            };
            if run_button.clicked() {
                if let Some(request_id) = self.next_query_request {
                    self.cancel_query(request_id);
                } else {
                    self.dispatch_query();
                }
            }
            if secondary_button_with_icon(ui, Icon::WandSparkles, "Format", self.theme).clicked() {
                self.query_text = Self::format_sql(&self.query_text);
            }
            input(ui, &mut self.query_folder, "folder (optional)", 150.0, self.theme);
            if ghost_button(ui, "New folder", self.theme).clicked() {
                if let Some(connection) = self.connections.first() {
                    if !self.query_folder.trim().is_empty() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::CreateQueryFolder {
                            request_id,
                            connection_id: connection.id.clone(),
                            name: self.query_folder.trim().to_owned(),
                        });
                        self.runtime_message = "Creating query folder…".to_owned();
                    }
                }
            }
            if secondary_button_with_icon(ui, Icon::Save, "Save", self.theme).clicked() {
                if let Some(connection) = self.connections.first() {
                    let request_id = self.task_bridge.next_request_id();
                    let name = self
                        .query_documents
                        .get(self.active_query_document)
                        .map(|document| document.title.clone())
                        .unwrap_or_else(|| "Saved query".to_owned());
                    let _ = self.task_bridge.send(UiCommand::SaveQuery {
                        request_id,
                        connection_id: connection.id.clone(),
                        name,
                        sql: self.query_text.clone(),
                        folder: (!self.query_folder.trim().is_empty()).then(|| self.query_folder.trim().to_owned()),
                    });
                    self.runtime_message = "Saving query…".to_owned();
                }
            }
            if ghost_button(
                ui,
                if self.selected_query.is_empty() {
                    "Run statement"
                } else {
                    "Run selection"
                },
                self.theme,
            )
            .clicked()
            {
                if self.selected_query.is_empty() {
                    let statement = self.query_text.split(';').next().unwrap_or_default().trim().to_owned();
                    if !statement.is_empty() {
                        self.query_text = statement;
                        self.dispatch_query();
                    }
                } else {
                    self.dispatch_query();
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ghost_button_with_icon(ui, Icon::Search, "Search", self.theme).clicked() {
                self.editor_search_open = !self.editor_search_open;
            }
            if ghost_button(ui, "A−", self.theme).clicked() {
                self.editor_font_size = (self.editor_font_size - 1.0).max(10.0);
            }
            if ghost_button(ui, "A+", self.theme).clicked() {
                self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
            }
            if ghost_button(ui, "Completion", self.theme).clicked() {
                self.completion_open = !self.completion_open;
            }
            if ghost_button(ui, "Snippets", self.theme).clicked() {
                self.snippets_open = !self.snippets_open;
            }
            ui.label(
                RichText::new(format!("{} px", self.editor_font_size))
                    .small()
                    .color(self.theme.text_muted),
            );
            if self.editor_search_open {
                input(ui, &mut self.editor_search, "Find in SQL…", 240.0, self.theme);
                if !self.editor_search.is_empty() {
                    let matches = self.query_text.matches(&self.editor_search).count();
                    ui.label(
                        RichText::new(format!("{matches} matches"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
            }
        });
        let editor_width = ui.max_rect().width();
        ui.allocate_ui_with_layout(egui::vec2(editor_width, 300.0), Layout::top_down(Align::Min), |ui| {
            editor_frame(self.theme).show(ui, |ui| {
                ui.set_min_width((editor_width - 24.0).max(0.0));
                ui.horizontal_top(|ui| {
                    let line_count = self.query_text.lines().count().max(1);
                    ui.vertical(|ui| {
                        for line in 1..=line_count {
                            ui.label(
                                RichText::new(format!("{line:>3}"))
                                    .monospace()
                                    .color(self.theme.text_muted),
                            );
                        }
                    });
                    ui.separator();
                    let editor_text_width = (editor_width - 72.0).max(280.0);
                    let editor_size = egui::vec2(editor_text_width, 260.0);
                    let theme = self.theme;
                    let output = ui.allocate_ui(editor_size, |ui| {
                        TextEdit::multiline(&mut self.query_text)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .min_size(editor_size)
                            .desired_rows(14)
                            .layouter(&mut |ui, text, wrap_width| Self::sql_layouter(ui, text, wrap_width, theme))
                            .lock_focus(true)
                            .show(ui)
                    });
                    if let Some(cursor_range) = output.inner.cursor_range {
                        let range = cursor_range.as_sorted_char_range();
                        if range.start < range.end && range.end <= self.query_text.len() {
                            self.selected_query = self
                                .query_text
                                .chars()
                                .skip(range.start)
                                .take(range.end - range.start)
                                .collect();
                        } else {
                            self.selected_query.clear();
                        }
                    }
                });
            });
        });
        if self.completion_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.label(RichText::new("SQL completion").strong());
                let is_sqlite = self
                    .connections
                    .first()
                    .map(|connection| connection.driver.eq_ignore_ascii_case("sqlite"))
                    .unwrap_or(false);
                let mut candidates = vec![
                    "SELECT".to_owned(),
                    "FROM".to_owned(),
                    "WHERE".to_owned(),
                    "JOIN".to_owned(),
                    "GROUP BY".to_owned(),
                    "ORDER BY".to_owned(),
                    "LIMIT".to_owned(),
                    "COUNT(*)".to_owned(),
                ];
                if is_sqlite {
                    candidates.extend(["GLOB", "strftime", "WITHOUT ROWID"].into_iter().map(str::to_owned));
                } else {
                    candidates.extend(
                        ["ILIKE", "RETURNING", "jsonb_build_object"]
                            .into_iter()
                            .map(str::to_owned),
                    );
                }
                candidates.extend(self.schema.tables.iter().cloned());
                candidates.extend(self.schema.columns.iter().cloned());
                candidates.extend(self.schema.views.iter().map(|view| view.name.clone()));
                candidates.extend(self.schema.functions.iter().map(|function| function.name.clone()));
                for keyword in candidates.iter() {
                    if ui
                        .selectable_label(false, keyword)
                        .on_hover_text("Insert SQL keyword or expression")
                        .clicked()
                    {
                        self.query_text.push_str(keyword);
                        self.completion_open = false;
                    }
                }
            });
        }
        if self.snippets_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.label(RichText::new("SQL snippets").strong());
                if compact_button(ui, "SELECT table", self.theme).clicked() {
                    self.insert_snippet("SELECT *\nFROM table_name\nLIMIT 100;");
                    self.snippets_open = false;
                }
                if compact_button(ui, "UPDATE by primary key", self.theme).clicked() {
                    self.insert_snippet("UPDATE table_name\nSET column_name = value\nWHERE id = 1;");
                    self.snippets_open = false;
                }
            });
        }
        if !self.diagnostics.is_empty() {
            ui.colored_label(self.theme.warning, format!("Diagnostics · {}", self.diagnostics.len()));
            for diagnostic in &self.diagnostics {
                ui.colored_label(self.theme.warning, format!("• {diagnostic}"));
            }
        }
        ui.add_space(12.0);
        let result = self.query_result.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Results").strong().color(self.theme.text_primary));
            let row_label = result
                .as_ref()
                .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                .unwrap_or_else(|| "No result".to_owned());
            ui.label(RichText::new(row_label).small().color(self.theme.text_muted));
            ui.label(
                RichText::new(self.runtime_message.as_str())
                    .small()
                    .color(self.theme.text_muted),
            );
            if result.is_some() && compact_button(ui, "Export", self.theme).clicked() {
                self.export_open = true;
            }
        });
        if self.export_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Export results");
                    ui.selectable_value(&mut self.export_format, "CSV".to_owned(), "CSV");
                    ui.selectable_value(&mut self.export_format, "TSV".to_owned(), "TSV");
                    input(ui, &mut self.export_path, "output path", 260.0, self.theme);
                    if compact_button(ui, "Export", self.theme).clicked() {
                        if let Some(result) = result.as_ref() {
                            self.export_result(result);
                        }
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.export_open = false;
                    }
                });
            });
        }
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                section_label(ui, "ROW MUTATION", self.theme);
                ui.label(
                    RichText::new("Requires a primary key")
                        .small()
                        .color(self.theme.text_muted),
                );
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                input(ui, &mut self.mutation_table, "table", 104.0, self.theme);
                input(ui, &mut self.mutation_pk_column, "pk column", 104.0, self.theme);
                input(ui, &mut self.mutation_pk_value, "pk value", 104.0, self.theme);
                input(ui, &mut self.mutation_column, "column", 104.0, self.theme);
                input(ui, &mut self.mutation_value, "new value", 104.0, self.theme);
                if compact_button(ui, "Update", self.theme).clicked() {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::UpdateTableRow {
                            request_id,
                            connection_id: connection.id.clone(),
                            schema: self.active_schema().to_owned(),
                            table: self.mutation_table.clone(),
                            column: self.mutation_column.clone(),
                            value: UiCell::Text(self.mutation_value.clone()),
                            pk_columns: vec![self.mutation_pk_column.clone()],
                            pk_values: vec![UiCell::Text(self.mutation_pk_value.clone())],
                        });
                    }
                }
                if compact_button(ui, "Delete", self.theme).clicked() {
                    self.mutation_delete_confirmation = true;
                }
                if self.mutation_delete_confirmation {
                    ui.colored_label(self.theme.warning, "Confirm delete?");
                    if compact_button(ui, "Yes", self.theme).clicked() {
                        if let Some(connection) = self.active_connection().cloned() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::DeleteTableRow {
                                request_id,
                                connection_id: connection.id.clone(),
                                schema: self.active_schema().to_owned(),
                                table: self.mutation_table.clone(),
                                pk_columns: vec![self.mutation_pk_column.clone()],
                                pk_values: vec![UiCell::Text(self.mutation_pk_value.clone())],
                            });
                        }
                        self.mutation_delete_confirmation = false;
                    }
                    if compact_button(ui, "No", self.theme).clicked() {
                        self.mutation_delete_confirmation = false;
                    }
                }
            });
        });
        ui.add_space(8.0);
        let results_width = ui.max_rect().width();
        panel_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((results_width - 24.0).max(0.0));
            if let Some(result) = result {
                self.draw_result_grid(ui, &result);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Run a query to see results").color(self.theme.text_muted));
                });
            }
        });
    }

    fn export_result(&mut self, result: &UiQueryResult) {
        let path = self.export_path.trim();
        if path.is_empty() {
            self.runtime_message = "Choose an export path first".to_owned();
            return;
        }
        let delimiter = if self.export_format == "CSV" { "," } else { "\t" };
        let mut output = result
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<Vec<_>>()
            .join(delimiter);
        output.push('\n');
        for row in &result.rows {
            output.push_str(
                &row.iter()
                    .map(|cell| match cell {
                        UiCell::Null => String::new(),
                        UiCell::Boolean(v) => v.to_string(),
                        UiCell::Number(v) | UiCell::Text(v) | UiCell::Json(v) | UiCell::Bytes(v) => v.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(delimiter),
            );
            output.push('\n');
        }
        match std::fs::write(path, output) {
            Ok(()) => self.runtime_message = format!("Exported {} rows to {path}", result.rows.len()),
            Err(error) => self.runtime_message = format!("Export failed: {error}"),
        }
        self.export_open = false;
    }

    fn draw_result_grid(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if result.columns.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Statement completed without rows").color(self.theme.text_muted));
            });
            return;
        }

        let editable = self.active_tab == WorkspaceTab::Table
            && self.table_view == TableView::Data
            && self.can_mutate_active_connection();

        if ui.input(|input| input.key_pressed(egui::Key::C) && input.modifiers.command) {
            self.copy_selected_cell(ui, result);
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_secondary));
            input(ui, &mut self.grid_filter, "Search visible rows…", 240.0, self.theme);
            if compact_button(ui, "Clear", self.theme).clicked() {
                self.grid_filter.clear();
            }
            if compact_button(ui, "Copy cell", self.theme).clicked() {
                self.copy_selected_cell(ui, result);
            }
            if compact_button(ui, "Copy row", self.theme).clicked() {
                self.copy_selected_row(ui, result);
            }
            if !self.copy_status.is_empty() {
                ui.label(
                    RichText::new(self.copy_status.as_str())
                        .small()
                        .color(self.theme.success),
                );
            }
            ui.label(
                RichText::new(if editable {
                    "Double-click a cell to edit · drag the divider to resize"
                } else {
                    "Click a cell to select · drag the divider to resize"
                })
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(6.0);

        let indexes =
            crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);
        ui.label(
            RichText::new(format!("{} matching rows · virtualized", indexes.len()))
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);

        let widths = self.column_widths(result.columns.len());
        let row_offset = if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            self.table_data_offset
        } else {
            0
        };
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.set_min_width(GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>());
            self.draw_grid_header(ui, result, &widths);
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show_rows(ui, 24.0, indexes.len(), |ui, range| {
                    for position in range {
                        let row_index = indexes[position];
                        let row = &result.rows[row_index];
                        let row_selected = self.selected_row == Some(row_index);
                        ui.horizontal(|ui| {
                            let row_number = crate::displayed_row_number(row_offset, row_index);
                            let row_response = ui.add_sized(
                                [GRID_ROW_NUMBER_WIDTH, 24.0],
                                egui::SelectableLabel::new(
                                    row_selected,
                                    RichText::new(row_number.to_string())
                                        .monospace()
                                        .small()
                                        .color(if row_selected {
                                            self.theme.accent
                                        } else {
                                            self.theme.text_muted
                                        }),
                                ),
                            );
                            if row_response.clicked() {
                                self.selected_cell = None;
                                self.selected_row = Some(row_index);
                                self.data_editing_cell = None;
                                self.data_edit_value.clear();
                                self.copy_status.clear();
                            }
                            for (column_index, cell) in row.iter().enumerate().take(result.columns.len()) {
                                let width = widths.get(column_index).copied().unwrap_or(180.0);
                                let fill = if row_selected {
                                    self.theme.surface_active
                                } else if position % 2 == 0 {
                                    self.theme.surface_panel
                                } else {
                                    self.theme.surface_elevated
                                };
                                egui::Frame::default().fill(fill).show(ui, |ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(width, 24.0),
                                        Layout::left_to_right(Align::Center),
                                        |ui| {
                                            ui.add_space(8.0);
                                            let selected =
                                                row_selected || self.selected_cell == Some((row_index, column_index));
                                            let editing =
                                                editable && self.data_editing_cell == Some((row_index, column_index));
                                            if editing {
                                                let response = ui.add_sized(
                                                    [width - 12.0, 22.0],
                                                    TextEdit::singleline(&mut self.data_edit_value)
                                                        .margin(egui::Margin::symmetric(6.0, 2.0))
                                                        .text_color(self.theme.text_primary),
                                                );
                                                response.request_focus();
                                                let commit = response.lost_focus()
                                                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                                                if commit {
                                                    self.submit_data_cell_edit(result, row_index, column_index);
                                                } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                                                    self.data_editing_cell = None;
                                                    self.data_edit_value.clear();
                                                }
                                            } else {
                                                let response = ui.add_sized(
                                                    [width - 12.0, 22.0],
                                                    egui::SelectableLabel::new(selected, Self::cell_label(cell)),
                                                );
                                                if response.double_clicked() && editable {
                                                    self.begin_data_cell_edit(row_index, column_index, cell);
                                                } else if response.clicked() {
                                                    self.selected_cell = Some((row_index, column_index));
                                                    self.selected_row = Some(row_index);
                                                    self.copy_status.clear();
                                                }
                                            }
                                        },
                                    );
                                });
                            }
                        });
                    }
                });
        });
    }

    fn copy_selected_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some((row_index, column_index)) = self.selected_cell else {
            self.copy_status = "Select a cell first".to_owned();
            return;
        };
        let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) else {
            self.copy_status = "Selected cell is no longer available".to_owned();
            return;
        };
        ui.output_mut(|output| output.copied_text = crate::cell_text(cell));
        self.copy_status = "Cell copied".to_owned();
    }

    fn copy_selected_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some(row_index) = self.selected_row else {
            self.copy_status = "Select a row first".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            self.copy_status = "Selected row is no longer available".to_owned();
            return;
        };
        let row_text = row.iter().map(crate::cell_text).collect::<Vec<_>>().join("\t");
        ui.output_mut(|output| output.copied_text = row_text);
        self.copy_status = "Row copied".to_owned();
    }

    fn column_widths(&mut self, count: usize) -> Vec<f32> {
        if self.grid_column_widths.len() != count {
            self.grid_column_widths = vec![180.0; count];
        }
        self.grid_column_widths.clone()
    }

    fn draw_grid_header(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, widths: &[f32]) {
        ui.horizontal(|ui| {
            ui.add_sized(
                [GRID_ROW_NUMBER_WIDTH, 28.0],
                egui::Button::new(RichText::new("#").strong().color(self.theme.text_muted))
                    .fill(self.theme.surface_hover),
            );
            for (index, column) in result.columns.iter().enumerate() {
                let width = widths.get(index).copied().unwrap_or(180.0);
                let sort_marker = match self.grid_sort_column {
                    Some(active) if active == index && self.grid_sort_desc => " ↓",
                    Some(active) if active == index => " ↑",
                    _ => "",
                };
                let response = ui.add_sized(
                    [width, 28.0],
                    egui::Button::new(
                        RichText::new(format!("{}{}", column.name, sort_marker))
                            .strong()
                            .color(self.theme.text_primary),
                    )
                    .fill(self.theme.surface_hover),
                );
                if response.clicked() {
                    if self.grid_sort_column == Some(index) {
                        self.grid_sort_desc = !self.grid_sort_desc;
                    } else {
                        self.grid_sort_column = Some(index);
                        self.grid_sort_desc = false;
                    }
                }
                let (_divider_rect, divider) = ui.allocate_exact_size(egui::vec2(4.0, 28.0), Sense::drag());
                if divider.drag_started() {
                    self.grid_resize_start = Some((index, width));
                }
                if divider.dragged() {
                    let start_width = self
                        .grid_resize_start
                        .filter(|(column, _)| *column == index)
                        .map(|(_, start)| start)
                        .unwrap_or(width);
                    self.grid_column_widths[index] = (start_width + divider.drag_delta().x).clamp(90.0, 520.0);
                }
            }
        });
    }

    fn cell_label(cell: &crate::UiCell) -> RichText {
        match cell {
            crate::UiCell::Null => RichText::new("NULL").italics(),
            crate::UiCell::Boolean(value) => RichText::new(value.to_string()),
            crate::UiCell::Number(value) => RichText::new(value.as_str()).monospace(),
            crate::UiCell::Text(value) => RichText::new(value.as_str()),
            crate::UiCell::Json(value) => RichText::new(value.as_str()).monospace(),
            crate::UiCell::Bytes(value) => RichText::new(value.as_str()).monospace(),
        }
    }

    fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        self.editing_connection_id = Some(connection.id.clone());
        self.connection_draft = UiConnectionDraft {
            name: connection.name.clone(),
            host: connection.host.clone(),
            port: connection.port.to_string(),
            database: connection.database.clone(),
            username: connection.username.clone(),
            password: String::new(),
            driver: if connection.driver == "SQLite" {
                UiDriver::Sqlite
            } else {
                UiDriver::Postgres
            },
            ssl_mode: UiSslMode::Disable,
            readonly: connection.readonly,
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
        };
        self.connection_error = "Enter the password again to save changes".to_owned();
        self.connection_dialog_open = true;
    }

    fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(connection_id) = self.delete_confirmation_id.clone() else {
            return;
        };
        let name = self
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "this connection".to_owned());
        egui::Window::new("Delete connection")
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(format!("Delete {name} and its saved credentials?"));
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "This action cannot be undone.");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Delete", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteConnection {
                            request_id,
                            connection_id: connection_id.clone(),
                        });
                        self.runtime_message = format!("Deleting {name}…");
                        self.delete_confirmation_id = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            });
    }

    fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(folder_id) = self.folder_delete_confirmation.clone() else {
            return;
        };
        let folder_name = self
            .query_folders
            .iter()
            .find(|folder| folder.id == folder_id)
            .map(|folder| folder.name.clone())
            .unwrap_or_else(|| "this folder".to_owned());
        let mut open = true;
        egui::Window::new("Delete query folder")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(format!("Delete {folder_name} and its saved-query links?"));
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "Saved queries in this folder will become unfiled.");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Delete folder", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteQueryFolder {
                            request_id,
                            id: folder_id.clone(),
                        });
                        self.folder_delete_confirmation = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.folder_delete_confirmation = None;
                    }
                });
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }

    fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let title = if self.editing_connection_id.is_some() {
            "Edit connection"
        } else {
            "New connection"
        };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(if self.editing_connection_id.is_some() {
                        "Update a safe database connection"
                    } else {
                        "Create a safe database connection"
                    })
                    .color(self.theme.text_secondary),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Driver");
                    ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Postgres, "PostgreSQL");
                    ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Sqlite, "SQLite");
                });
                ui.add_space(6.0);
                Self::form_row(ui, "Name", &mut self.connection_draft.name, "Production DB", self.theme);
                if self.connection_draft.driver == UiDriver::Postgres {
                    Self::form_row(ui, "Host", &mut self.connection_draft.host, "localhost", self.theme);
                    Self::form_row(ui, "Port", &mut self.connection_draft.port, "5432", self.theme);
                    Self::form_row(ui, "Database", &mut self.connection_draft.database, "app", self.theme);
                    Self::form_row(
                        ui,
                        "Username",
                        &mut self.connection_draft.username,
                        "postgres",
                        self.theme,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Password");
                        password_input(ui, &mut self.connection_draft.password, "Password", 300.0, self.theme);
                    });
                    ui.horizontal(|ui| {
                        ui.label("SSL");
                        for (mode, label) in [
                            (UiSslMode::Disable, "Disable"),
                            (UiSslMode::Require, "Require"),
                            (UiSslMode::VerifyFull, "Verify full"),
                        ] {
                            ui.selectable_value(&mut self.connection_draft.ssl_mode, mode, label);
                        }
                    });
                    egui::CollapsingHeader::new("SSH tunnel").show(ui, |ui| {
                        ui.checkbox(&mut self.connection_draft.ssh_tunnel_enabled, "Use SSH tunnel");
                        if self.connection_draft.ssh_tunnel_enabled {
                            Self::form_row(
                                ui,
                                "SSH host",
                                &mut self.connection_draft.ssh_host,
                                "bastion.example.com",
                                self.theme,
                            );
                            Self::form_row(ui, "SSH port", &mut self.connection_draft.ssh_port, "22", self.theme);
                            Self::form_row(
                                ui,
                                "SSH user",
                                &mut self.connection_draft.ssh_user,
                                "ubuntu",
                                self.theme,
                            );
                            ui.horizontal(|ui| {
                                Self::form_row(
                                    ui,
                                    "Private key",
                                    &mut self.connection_draft.ssh_private_key,
                                    "/home/me/.ssh/id_ed25519",
                                    self.theme,
                                );
                                if compact_button(ui, "Browse…", self.theme).clicked() {
                                    let request_id = self.task_bridge.next_request_id();
                                    let _ = self.task_bridge.send(UiCommand::PickSshPrivateKey { request_id });
                                }
                            });
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        Self::form_row(
                            ui,
                            "SQLite file",
                            &mut self.connection_draft.database,
                            "/path/to/db.sqlite",
                            self.theme,
                        );
                        if compact_button(ui, "Browse…", self.theme).clicked() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::PickSqliteFile { request_id });
                        }
                    });
                }
                ui.checkbox(&mut self.connection_draft.readonly, "Read-only connection");
                if !self.connection_error.is_empty() {
                    ui.colored_label(self.theme.danger, self.connection_error.as_str());
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if compact_button(ui, "Test connection", self.theme).clicked() {
                        self.dispatch_connection_command(false);
                    }
                    if primary_button(ui, "Save connection", self.theme).clicked() {
                        self.dispatch_connection_command(true);
                    }
                    if ghost_button(ui, "Cancel", self.theme).clicked() {
                        self.connection_dialog_open = false;
                    }
                });
            });
        self.connection_dialog_open = open && self.connection_dialog_open;
    }

    fn form_row(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str, theme: DbProTheme) {
        ui.horizontal(|ui| {
            ui.label(label);
            input(ui, value, hint, 300.0, theme);
        });
    }

    fn dispatch_connection_command(&mut self, save: bool) {
        if self.connection_draft.name.trim().is_empty() || self.connection_draft.database.trim().is_empty() {
            self.connection_error = "Name and database are required".to_owned();
            return;
        }
        if save && self.connection_draft.driver == UiDriver::Postgres && self.connection_draft.password.is_empty() {
            self.connection_error = "Password is required for PostgreSQL".to_owned();
            return;
        }
        if self.connection_draft.driver == UiDriver::Postgres && self.connection_draft.port.parse::<u16>().is_err() {
            self.connection_error = "Port must be a number between 1 and 65535".to_owned();
            return;
        }
        if self.connection_draft.ssh_tunnel_enabled
            && (self.connection_draft.ssh_host.trim().is_empty()
                || self.connection_draft.ssh_user.trim().is_empty()
                || self.connection_draft.ssh_private_key.trim().is_empty())
        {
            self.connection_error = "SSH host, user and private key are required".to_owned();
            return;
        }
        if self.connection_draft.ssh_tunnel_enabled && self.connection_draft.ssh_port.parse::<u16>().is_err() {
            self.connection_error = "SSH port must be a number between 1 and 65535".to_owned();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        let draft = self.connection_draft.clone();
        let command = if save {
            if let Some(connection_id) = self.editing_connection_id.clone() {
                UiCommand::UpdateConnection {
                    request_id,
                    connection_id,
                    draft,
                }
            } else {
                UiCommand::CreateConnection { request_id, draft }
            }
        } else {
            UiCommand::TestConnection { request_id, draft }
        };
        let _ = self.task_bridge.send(command);
        self.connection_error.clear();
        self.runtime_message = if save {
            "Saving connection…"
        } else {
            "Testing connection…"
        }
        .to_owned();
    }

    fn draw_agent_panel(&mut self, ctx: &egui::Context) {
        let mut submit = false;
        let mut copy_sql = None;
        egui::SidePanel::right("agent_panel")
            .default_width(360.0)
            .min_width(300.0)
            .max_width(380.0)
            .frame(card_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if !self.agent_messages.is_empty()
                            && compact_icon_button(ui, Icon::RotateCcw, self.theme).clicked()
                        {
                            self.agent_messages.clear();
                        }
                        if compact_icon_button(ui, Icon::X, self.theme).clicked() {
                            self.set_agent_open(false, ctx);
                        }
                    });
                });
                ui.add_space(6.0);
                let context = self.agent_context();
                toolbar_frame(self.theme).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        badge(
                            ui,
                            &self.agent_provider_label,
                            self.theme.accent_soft,
                            self.theme.accent,
                        );
                        badge(
                            ui,
                            context.connection_name.as_deref().unwrap_or("No connection"),
                            self.theme.accent_soft,
                            self.theme.accent,
                        );
                        badge(
                            ui,
                            &context.driver,
                            self.theme.surface_active,
                            self.theme.text_secondary,
                        );
                        badge(
                            ui,
                            &format!("{} tables", context.tables.len()),
                            self.theme.surface_active,
                            self.theme.text_secondary,
                        );
                    });
                    ui.label(
                        RichText::new(&self.agent_provider_detail)
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
                ui.add_space(8.0);
                let messages_height = (ui.available_height() - 86.0).max(160.0);
                egui::ScrollArea::vertical()
                    .max_height(messages_height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.agent_messages.is_empty() {
                            ui.add_space(18.0);
                            ui.vertical_centered(|ui| {
                                ui.label(icon_text(Icon::Bot, "", self.theme.accent));
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("Database copilot")
                                        .strong()
                                        .color(self.theme.text_primary),
                                );
                                ui.label(
                                    RichText::new("Ask for an overview, a read-only query, or a query plan.")
                                        .small()
                                        .color(self.theme.text_secondary),
                                );
                            });
                            ui.add_space(18.0);
                            for suggestion in [
                                "Show me the schema overview",
                                "Count rows in customers",
                                "Explain customers query performance",
                            ] {
                                if ghost_button_with_icon(ui, Icon::WandSparkles, suggestion, self.theme).clicked() {
                                    self.agent_input = suggestion.to_owned();
                                    submit = true;
                                }
                            }
                        } else {
                            let messages = self.agent_messages.clone();
                            for message in messages {
                                let is_user = message.role == AgentRole::User;
                                if is_user {
                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        card_frame(self.theme).show(ui, |ui| {
                                            ui.label(RichText::new(message.content).color(self.theme.text_primary));
                                        });
                                    });
                                } else {
                                    card_frame(self.theme).show(ui, |ui| {
                                        ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                                        ui.add_space(4.0);
                                        ui.label(RichText::new(message.content).color(self.theme.text_primary));
                                        if message.requires_confirmation {
                                            ui.add_space(8.0);
                                            ui.label(icon_text(
                                                Icon::TriangleAlert,
                                                "Review carefully before running",
                                                self.theme.warning,
                                            ));
                                        }
                                        if let Some(sql) = message.sql {
                                            ui.add_space(8.0);
                                            editor_frame(self.theme).show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    section_label(ui, "SQL draft", self.theme);
                                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                        if compact_icon_button(ui, Icon::Copy, self.theme).clicked() {
                                                            copy_sql = Some(sql.clone());
                                                        }
                                                    });
                                                });
                                                ui.add_space(4.0);
                                                ui.label(
                                                    RichText::new(sql.as_str())
                                                        .monospace()
                                                        .color(self.theme.code_keyword),
                                                );
                                            });
                                            ui.add_space(6.0);
                                            if secondary_button_with_icon(
                                                ui,
                                                Icon::ArrowUp,
                                                "Insert into Query",
                                                self.theme,
                                            )
                                            .clicked()
                                            {
                                                self.insert_agent_sql(&sql);
                                            }
                                            if !message.requires_confirmation
                                                && self.connected
                                                && self.active_connection_id.is_some()
                                                && secondary_button_with_icon(
                                                    ui,
                                                    Icon::Play,
                                                    "Run read-only",
                                                    self.theme,
                                                )
                                                .clicked()
                                            {
                                                self.run_agent_read_only(&sql);
                                            }
                                        }
                                    });
                                }
                                ui.add_space(8.0);
                            }
                        }
                        if self.agent_request.is_some() {
                            card_frame(self.theme).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(RichText::new("Thinking with Codex…").color(self.theme.text_secondary));
                                });
                            });
                            ui.add_space(8.0);
                        }
                    });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                toolbar_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let input_width = (ui.available_width() - 34.0).max(120.0);
                        let response = input(
                            ui,
                            &mut self.agent_input,
                            "Ask about schema or draft SQL…",
                            input_width,
                            self.theme,
                        );
                        let send =
                            compact_icon_button_enabled(ui, Icon::Send, self.agent_request.is_none(), self.theme);
                        if send.clicked()
                            || (self.agent_request.is_none()
                                && response.has_focus()
                                && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.command))
                        {
                            submit = true;
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} · Cmd/Ctrl+Enter to send · writes stay unexecuted",
                            self.agent_provider_label
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                });
            });
        if submit {
            self.submit_agent_prompt();
        }
        if let Some(sql) = copy_sql {
            ctx.output_mut(|output| output.copied_text = sql);
            self.copy_status = "Agent SQL copied".to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result() -> UiQueryResult {
        UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "name".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![
                vec![
                    crate::UiCell::Number("2".to_owned()),
                    crate::UiCell::Text("Beta".to_owned()),
                ],
                vec![
                    crate::UiCell::Number("1".to_owned()),
                    crate::UiCell::Text("Alpha".to_owned()),
                ],
                vec![
                    crate::UiCell::Number("3".to_owned()),
                    crate::UiCell::Text("Gamma".to_owned()),
                ],
            ],
            row_count: 3,
            duration_ms: 2,
        }
    }

    #[test]
    fn filter_returns_original_row_indexes() {
        let app = DbProApp {
            grid_filter: "gamma".to_owned(),
            ..Default::default()
        };
        let value = result();
        assert_eq!(
            crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
            vec![2]
        );
    }

    #[test]
    fn sort_is_stable_over_filtered_indexes() {
        let mut app = DbProApp {
            grid_sort_column: Some(0),
            ..Default::default()
        };
        let value = result();
        assert_eq!(
            crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
            vec![1, 0, 2]
        );
        app.grid_sort_desc = true;
        assert_eq!(
            crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
            vec![2, 0, 1]
        );
    }

    #[test]
    fn cell_text_keeps_null_and_json_visible() {
        assert_eq!(crate::cell_text(&crate::UiCell::Null), "NULL");
        assert_eq!(
            crate::cell_text(&crate::UiCell::Json("{\"ok\":true}".to_owned())),
            "{\"ok\":true}"
        );
    }

    #[test]
    fn displayed_row_number_tracks_database_page_offset() {
        assert_eq!(crate::displayed_row_number(0, 0), 1);
        assert_eq!(crate::displayed_row_number(100, 0), 101);
        assert_eq!(crate::displayed_row_number(100, 49), 150);
    }

    #[test]
    fn composite_primary_key_identity_preserves_each_cell_type() {
        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "tenant_id".to_owned(),
                    data_type: "INTEGER".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "item_id".to_owned(),
                    data_type: "TEXT".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![vec![UiCell::Number("7".to_owned()), UiCell::Text("sku-42".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };
        let info = UiTableInfo {
            schema: "main".to_owned(),
            name: "inventory".to_owned(),
            row_count: Some(1),
            columns: Vec::new(),
            primary_key: Some(vec!["tenant_id".to_owned(), "item_id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
        };

        let (columns, values) = DbProApp::row_identity(&result, &info, 0).expect("row identity expected");

        assert_eq!(columns, vec!["tenant_id", "item_id"]);
        assert_eq!(
            values,
            vec![UiCell::Number("7".to_owned()), UiCell::Text("sku-42".to_owned())]
        );
    }

    #[test]
    fn update_value_keeps_empty_text_and_parses_typed_values() {
        assert_eq!(
            DbProApp::parse_update_value("", "TEXT").unwrap(),
            UiCell::Text(String::new())
        );
        assert_eq!(
            DbProApp::parse_update_value("false", "BOOLEAN").unwrap(),
            UiCell::Boolean(false)
        );
        assert!(DbProApp::parse_update_value("not-an-int", "INTEGER").is_err());
    }

    #[test]
    fn insert_value_respects_column_types() {
        assert_eq!(
            DbProApp::parse_insert_value("42", "INTEGER").unwrap(),
            Some(crate::UiCell::Number("42".to_owned()))
        );
        assert_eq!(
            DbProApp::parse_insert_value("true", "BOOLEAN").unwrap(),
            Some(crate::UiCell::Boolean(true))
        );
        assert_eq!(
            DbProApp::parse_insert_value("{\"active\":true}", "JSONB").unwrap(),
            Some(crate::UiCell::Json("{\"active\":true}".to_owned()))
        );
        assert_eq!(
            DbProApp::parse_insert_value("12.50", "NUMERIC(10,2)").unwrap(),
            Some(crate::UiCell::Text("12.50".to_owned()))
        );
        assert_eq!(
            DbProApp::parse_insert_value("1.20e1", "DECIMAL(10,2)").unwrap(),
            Some(crate::UiCell::Text("1.20e1".to_owned()))
        );
    }

    #[test]
    fn insert_value_rejects_invalid_typed_input() {
        assert!(DbProApp::parse_insert_value("maybe", "BOOLEAN").is_err());
        assert!(DbProApp::parse_insert_value("not-json", "JSON").is_err());
        assert!(DbProApp::parse_insert_value("4.2", "INTEGER").is_err());
        assert!(DbProApp::parse_insert_value("12.345", "NUMERIC(10,2)").is_err());
        assert!(DbProApp::parse_insert_value("123456789.01", "NUMERIC(10,2)").is_err());
        assert!(DbProApp::parse_update_value("", "DECIMAL(10,2)").is_err());
    }

    #[test]
    fn closing_agent_restores_sidebar_state_after_narrow_window() {
        let mut app = DbProApp::default();
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
            ..Default::default()
        });

        app.set_agent_open(true, &ctx);
        assert!(!app.sidebar_open);

        app.set_agent_open(false, &ctx);
        assert!(app.sidebar_open);
        let _ = ctx.end_pass();
    }

    #[test]
    fn selected_connection_is_not_shown_as_connected() {
        let connection = UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            readonly: false,
        };
        let mut app = DbProApp {
            connections: vec![connection],
            active_connection_id: Some("conn-1".to_owned()),
            connected: false,
            ..Default::default()
        };

        assert_eq!(app.active_connection_name(), "Local");
        assert_eq!(app.connection_indicator(&app.connections[0]).1, app.theme.accent);
        app.connected = true;
        assert_eq!(app.connection_indicator(&app.connections[0]).1, app.theme.success);
    }

    #[test]
    fn closing_query_document_restores_the_next_valid_document() {
        let mut app = DbProApp::default();
        app.new_query_document();
        app.query_text = "select 2".to_owned();
        app.persist_active_query_document();

        app.close_query_document(0);

        assert_eq!(app.query_documents.len(), 1);
        assert_eq!(app.active_query_document, 0);
        assert_eq!(app.query_text, "select 2");
        assert_eq!(app.runtime_message, "Closed Query 2");
    }

    #[test]
    fn quick_open_filters_workspaces_by_title_and_description() {
        let app = DbProApp {
            palette_query: "relationship".to_owned(),
            ..Default::default()
        };
        let items = app.filtered_palette_items(PaletteMode::QuickOpen);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "ER diagram");
    }

    #[test]
    fn command_palette_new_query_keeps_a_query_entry_point() {
        let mut app = DbProApp::default();
        let ctx = egui::Context::default();
        app.execute_palette_action(PaletteAction::NewQuery, &ctx);

        assert_eq!(app.active_tab, WorkspaceTab::Query);
        assert_eq!(app.query_documents.len(), 2);
        assert!(app.palette_mode.is_none());
    }

    #[test]
    fn command_palette_refresh_schema_bypasses_the_metadata_cache() {
        let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connections = vec![UiConnectionSummary {
            id: "active".to_owned(),
            name: "Active".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "active".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            readonly: false,
        }];
        app.active_connection_id = Some("active".to_owned());
        app.connected = true;
        let ctx = egui::Context::default();

        app.execute_palette_action(PaletteAction::RefreshSchema, &ctx);

        let UiCommand::IntrospectSchema {
            connection_id,
            force_refresh,
            ..
        } = command_rx.try_recv().expect("schema refresh command expected")
        else {
            panic!("expected IntrospectSchema command");
        };
        assert_eq!(connection_id, "active");
        assert!(force_refresh);
        assert_eq!(app.runtime_message, "Refreshing schema…");
    }

    #[test]
    fn schema_refresh_reloads_the_selected_table_after_summary_completion() {
        let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connections = vec![UiConnectionSummary {
            id: "active".to_owned(),
            name: "Active".to_owned(),
            host: String::new(),
            port: 0,
            database: "active".to_owned(),
            username: String::new(),
            driver: "SQLite".to_owned(),
            readonly: false,
        }];
        app.active_connection_id = Some("active".to_owned());
        app.selected_table = Some("customers".to_owned());
        app.active_tab = WorkspaceTab::Table;
        app.refresh_table_info_after_schema = true;
        event_tx
            .send(UiEvent::SchemaLoaded {
                request_id: crate::RequestId(1),
                schema: UiSchemaSummary {
                    tables: vec!["customers".to_owned()],
                    columns: vec!["id".to_owned()],
                    table_details: Vec::new(),
                    views: Vec::new(),
                    triggers: Vec::new(),
                    functions: Vec::new(),
                },
            })
            .expect("schema event should be queued");

        app.apply_runtime_events();

        let UiCommand::LoadTableInfo {
            connection_id,
            schema,
            table,
            ..
        } = command_rx.try_recv().expect("table metadata refresh expected")
        else {
            panic!("expected LoadTableInfo command");
        };
        assert_eq!(connection_id, "active");
        assert_eq!(schema, "main");
        assert_eq!(table, "customers");
        assert!(!app.refresh_table_info_after_schema);
    }

    #[test]
    fn query_dispatch_uses_the_active_connection_not_the_first_connection() {
        let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connections = vec![
            UiConnectionSummary {
                id: "first".to_owned(),
                name: "First".to_owned(),
                host: "localhost".to_owned(),
                port: 5432,
                database: "first".to_owned(),
                username: "postgres".to_owned(),
                driver: "PostgreSQL".to_owned(),
                readonly: false,
            },
            UiConnectionSummary {
                id: "active".to_owned(),
                name: "Active".to_owned(),
                host: "localhost".to_owned(),
                port: 5432,
                database: "active".to_owned(),
                username: "postgres".to_owned(),
                driver: "PostgreSQL".to_owned(),
                readonly: false,
            },
        ];
        app.active_connection_id = Some("active".to_owned());
        app.connected = true;
        app.dispatch_query();

        let UiCommand::RunQuery { connection_id, .. } = command_rx.try_recv().expect("query command expected") else {
            panic!("expected RunQuery command");
        };
        assert_eq!(connection_id, "active");
    }

    #[test]
    fn ddl_apply_dispatch_requires_an_explicit_request_and_uses_active_connection() {
        let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connections = vec![UiConnectionSummary {
            id: "active".to_owned(),
            name: "Active".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "active".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            readonly: false,
        }];
        app.active_connection_id = Some("active".to_owned());
        app.connected = true;
        app.table_ddl = Some("CREATE TABLE \"public\".\"audit\" (id INTEGER)".to_owned());

        app.submit_ddl();

        let UiCommand::ExecuteDdl { connection_id, sql, .. } = command_rx.try_recv().expect("DDL command expected")
        else {
            panic!("expected ExecuteDdl command");
        };
        assert_eq!(connection_id, "active");
        assert_eq!(sql, "CREATE TABLE \"public\".\"audit\" (id INTEGER)");
        assert!(app.ddl_execution_request.is_some());
    }
}
