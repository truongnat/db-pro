use crate::{
    DbProTheme, TaskBridge, UiCommand, UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiQueryResult, UiSavedQuerySummary, UiSchemaSummary, UiSslMode,
};
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Sense, TextEdit, TextFormat, TopBottomPanel};
use eframe::egui::text::LayoutJob;
use std::sync::Arc;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Welcome,
    Query,
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
    agent_input: String,
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
    copy_status: String,
    connections: Vec<UiConnectionSummary>,
    saved_queries: Vec<UiSavedQuerySummary>,
    schema: UiSchemaSummary,
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
}

impl DbProApp {
    pub fn with_task_bridge(task_bridge: TaskBridge) -> Self {
        Self::with_task_bridge_and_storage(task_bridge, None)
    }

    pub fn with_task_bridge_and_storage(
        task_bridge: TaskBridge,
        storage: Option<&dyn eframe::Storage>,
    ) -> Self {
        let mut app = Self {
            task_bridge,
            ..Self::default()
        };
        if let Some(storage) = storage {
            if let Some(widths) = storage.get_string("dbpro.native.grid-widths") {
                if let Ok(widths) = serde_json::from_str::<Vec<f32>>(&widths) {
                    app.grid_column_widths = widths
                        .into_iter()
                        .map(|width| width.clamp(90.0, 520.0))
                        .collect();
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
        Self {
            theme: DbProTheme::default(),
            activity: Activity::Explorer,
            active_tab: WorkspaceTab::Welcome,
            sidebar_open: true,
            agent_open: false,
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
            agent_input: String::new(),
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
            copy_status: String::new(),
            connections: Vec::new(),
            saved_queries: Vec::new(),
            schema: UiSchemaSummary { tables: Vec::new(), columns: Vec::new() },
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

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.theme.surface_app))
            .show(ctx, |ui| {
                self.draw_workspace(ui);
            });

        if self.agent_open {
            self.draw_agent_panel(ctx);
        }
        if self.connection_dialog_open {
            self.draw_connection_dialog(ctx);
        }
        if self.delete_confirmation_id.is_some() {
            self.draw_delete_confirmation(ctx);
        }
    }
}

impl DbProApp {
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
                        let _ = self.task_bridge.send(UiCommand::ListSavedQueries { request_id, connection_id: connection_id.clone() });
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::IntrospectSchema { request_id, connection_id });
                    }
                }
                UiEvent::SavedQueriesLoaded { queries, .. } => {
                    self.saved_queries = queries;
                }
                UiEvent::SchemaLoaded { schema, .. } => {
                    self.schema = schema;
                    self.runtime_message = format!("Schema loaded · {} tables · {} columns", self.schema.tables.len(), self.schema.columns.len());
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
                UiEvent::BackupCompleted { output_path, size_bytes, .. } => {
                    self.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
                }
                UiEvent::OperationCompleted { operation, .. } => {
                    self.runtime_message = operation.clone();
                    self.connections_requested = false;
                    if operation == "connection.created" || operation == "connection.updated" {
                        self.connection_dialog_open = false;
                        self.editing_connection_id = None;
                    }
                    if operation.starts_with("query") || operation.starts_with("query-folder") {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::ListSavedQueries { request_id, connection_id });
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
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query failed · {message}");
                        self.next_query_request = None;
                    }
                }
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.command) {
            self.active_tab = WorkspaceTab::Query;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.command) {
            self.sidebar_open = !self.sidebar_open;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) {
            self.editor_search_open = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5) || (i.key_pressed(egui::Key::Enter) && i.modifiers.command)) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.next_query_request {
                self.cancel_query(request_id);
            } else if self.editor_search_open {
                self.editor_search_open = false;
            } else {
                self.agent_open = false;
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
        let Some(connection) = self.connections.first() else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() { self.query_text.clone() } else { self.selected_query.clone() };
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
            connection_id: connection.id.clone(),
            sql,
        });
    }

    fn draw_topbar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("topbar")
            .exact_height(48.0)
            .frame(egui::Frame::default().fill(self.theme.surface_panel))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("DB").strong().color(self.theme.accent));
                    ui.label(RichText::new("PRO").strong().color(self.theme.text_primary));
                    ui.separator();
                    ui.label(RichText::new("Workspace").color(self.theme.text_secondary));
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if ui.small_button("Choose…").clicked() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::PickBackupFile { request_id });
                        }
                        ui.add(TextEdit::singleline(&mut self.backup_output_path).hint_text("backup path").desired_width(150.0));
                        if ui.small_button("Backup").clicked() {
                            if let Some(connection) = self.connections.first() {
                                let request_id = self.task_bridge.next_request_id();
                                let _ = self.task_bridge.send(UiCommand::Backup { request_id, connection_id: connection.id.clone(), output_path: self.backup_output_path.clone(), custom_format: false });
                            }
                        }
                        if ui.small_button("Choose…").clicked() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::PickRestoreFile { request_id });
                        }
                        ui.add(TextEdit::singleline(&mut self.restore_input_path).hint_text("restore path").desired_width(150.0));
                        if ui.small_button("Restore").clicked() {
                            self.restore_confirmation = true;
                        }
                        if self.restore_confirmation {
                            ui.colored_label(self.theme.warning, "Overwrite database?");
                            if ui.small_button("Confirm").clicked() {
                                if let Some(connection) = self.connections.first() {
                                    let request_id = self.task_bridge.next_request_id();
                                    let _ = self.task_bridge.send(UiCommand::Restore { request_id, connection_id: connection.id.clone(), input_path: self.restore_input_path.clone(), custom_format: false });
                                }
                                self.restore_confirmation = false;
                            }
                            if ui.small_button("Cancel").clicked() { self.restore_confirmation = false; }
                        }
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(RichText::new("✦  Agent").color(self.theme.text_secondary))
                            .clicked()
                        {
                            self.agent_open = !self.agent_open;
                        }
                        ui.button(RichText::new("⌘ P  Quick Open").color(self.theme.text_muted));
                        ui.label(RichText::new("v0.1 native preview").small().color(self.theme.text_muted));
                    });
                });
            });
    }

    fn draw_statusbar(&self, ctx: &egui::Context) {
        TopBottomPanel::bottom("statusbar")
            .exact_height(26.0)
            .frame(egui::Frame::default().fill(self.theme.surface_panel))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    let (color, label) = if self.connected {
                        (self.theme.success, "Connected")
                    } else {
                        (self.theme.warning, "Not connected")
                    };
                    ui.colored_label(color, "●");
                    ui.label(RichText::new(label).small().color(self.theme.text_secondary));
                    ui.separator();
                    ui.label(RichText::new("PostgreSQL").small().color(self.theme.text_muted));
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
            .frame(egui::Frame::default().fill(self.theme.surface_panel))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    for (activity, icon, hint) in [
                        (Activity::Explorer, "⌘", "Explorer"),
                        (Activity::History, "◷", "History"),
                        (Activity::Settings, "⚙", "Settings"),
                    ] {
                        let active = self.activity == activity;
                        let response = ui.add_sized(
                            [36.0, 36.0],
                            egui::Button::new(RichText::new(icon).size(18.0).color(if active {
                                self.theme.accent
                            } else {
                                self.theme.text_muted
                            }))
                            .fill(if active { self.theme.surface_hover } else { Color32::TRANSPARENT }),
                        );
                        if response.on_hover_text(hint).clicked() {
                            self.activity = activity;
                            self.sidebar_open = true;
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
            .frame(egui::Frame::default().fill(self.theme.surface_panel))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(match self.activity {
                        Activity::Explorer => "EXPLORER",
                        Activity::History => "QUERY HISTORY",
                        Activity::Settings => "SETTINGS",
                    }).small().strong().color(self.theme.text_secondary));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("‹").on_hover_text("Hide sidebar (⌘B)").clicked() {
                            self.sidebar_open = false;
                        }
                    });
                });
                ui.add_space(12.0);

                match self.activity {
                    Activity::Explorer => self.draw_explorer(ui),
                    Activity::History => self.draw_history(ui),
                    Activity::Settings => self.draw_settings(ui),
                }
            });
    }

    fn draw_explorer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⌄").color(self.theme.text_muted));
            ui.label(RichText::new(self.connection_name.as_str()).strong());
            ui.label(RichText::new("PG").small().color(self.theme.accent));
        });
        ui.add_space(8.0);
        if self.connections.is_empty() {
            ui.label(RichText::new("No saved connections").color(self.theme.text_muted));
            ui.label(RichText::new("Create one from the next backend slice.").small().color(self.theme.text_muted));
        } else {
            for connection in self.connections.clone() {
                let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if is_active { "●" } else { "○" }).color(if connection.readonly {
                        self.theme.warning
                    } else {
                        self.theme.success
                    }));
                    if ui
                        .selectable_label(is_active, RichText::new(connection.name.as_str()).strong())
                        .clicked()
                    {
                        self.active_connection_id = Some(connection.id.clone());
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::Connect {
                            request_id,
                            connection_id: connection.id.clone(),
                        });
                        self.runtime_message = format!("Connecting to {}…", connection.name);
                    }
                    ui.label(RichText::new(connection.driver.as_str()).small().color(self.theme.accent));
                    if is_active && ui.small_button("Edit").clicked() {
                        self.open_edit_connection(&connection);
                    }
                    if is_active && ui.small_button("×").on_hover_text("Delete connection").clicked() {
                        self.delete_confirmation_id = Some(connection.id.clone());
                    }
                });
            }
        }
        ui.add_space(12.0);
        ui.collapsing("◫  Schemas", |ui| {
            ui.label(RichText::new("public").small().color(self.theme.text_secondary));
        });
        let tables = self.schema.tables.clone();
        ui.collapsing(format!("▦  Tables ({})", tables.len()), |ui| {
            for table in tables.iter().take(100) {
                if ui.selectable_label(false, format!("  {table}")).clicked() {
                    self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
                    self.active_tab = WorkspaceTab::Query;
                }
            }
        });
        ui.collapsing(format!("◌  Columns ({})", self.schema.columns.len()), |ui| {
            for column in self.schema.columns.iter().take(100) {
                ui.label(RichText::new(format!("  {column}")).small().color(self.theme.text_muted));
            }
        });
        ui.add_space(16.0);
        if ui.button("＋  New connection").clicked() {
            self.editing_connection_id = None;
            self.connection_draft = UiConnectionDraft::default();
            self.connection_error.clear();
            self.connection_dialog_open = true;
        }
    }

    fn draw_history(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Saved queries").small().strong().color(self.theme.text_muted));
        if self.saved_queries.is_empty() {
            ui.label(RichText::new("No saved queries").small().color(self.theme.text_muted));
        } else {
            let saved = self.saved_queries.clone();
            let mut folders = Vec::new();
            for query in &saved {
                let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
                if !folders.contains(&folder) {
                    folders.push(folder.clone());
                    ui.label(RichText::new(format!("▾ {folder}")).strong().color(self.theme.text_secondary));
                }
                ui.horizontal(|ui| {
                    if ui.selectable_label(false, &query.name).clicked() {
                        self.query_text = query.sql.clone();
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if ui.small_button("rename").clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let name = if self.query_folder.trim().is_empty() { format!("{} (renamed)", query.name) } else { self.query_folder.trim().to_owned() };
                        let _ = self.task_bridge.send(UiCommand::RenameSavedQuery { request_id, id: query.id.clone(), name });
                    }
                    if ui.small_button("delete").clicked() {
                        self.delete_confirmation_id = Some(query.id.clone());
                    }
                });
            }
            if let Some(id) = self.delete_confirmation_id.clone() {
                ui.colored_label(self.theme.warning, "Delete this saved query?");
                ui.horizontal(|ui| {
                    if ui.button("Confirm delete").clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteSavedQuery { request_id, id });
                        self.delete_confirmation_id = None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            }
        }
        ui.separator();
        ui.label(RichText::new("Local history").small().strong().color(self.theme.text_muted));
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

    fn draw_settings(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Appearance").strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Dark theme").color(self.theme.text_secondary));
        ui.label(RichText::new("Native renderer").small().color(self.theme.text_muted));
    }

    fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let welcome = ui.selectable_label(self.active_tab == WorkspaceTab::Welcome, "⌂  Welcome");
            if welcome.clicked() {
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
                if ui.selectable_label(selected, format!("◉  {title}  ×")).clicked() {
                    self.switch_query_document(index);
                    self.active_tab = WorkspaceTab::Query;
                }
            }
            if ui.small_button("＋").on_hover_text("New query").clicked() {
                self.new_query_document();
            }
        });
        ui.separator();
        ui.add_space(14.0);

        match self.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
        }
    }

    fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(90.0);
            ui.label(RichText::new("A calmer way to work with databases").size(26.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new("Connect, explore, and query with confidence.").color(self.theme.text_secondary));
            ui.add_space(24.0);
            if ui.button(RichText::new("＋  New query").color(self.theme.text_primary)).clicked() {
                self.new_query_document();
            }
            ui.add_space(10.0);
            ui.label(RichText::new("⌘ P to open anything  ·  ⌘ B to toggle explorer").small().color(self.theme.text_muted));
        });
    }

    fn sql_layouter(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
        let keywords = [
            "select", "from", "where", "and", "or", "join", "left", "right", "inner", "group", "by",
            "order", "limit", "offset", "insert", "into", "values", "update", "set", "delete", "create",
            "table", "alter", "drop", "as", "on", "is", "null", "not", "returning", "with", "explain",
        ];
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let mut current = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let flush = |job: &mut LayoutJob, value: &mut String, color: Color32| {
            if !value.is_empty() {
                job.append(value, 0.0, TextFormat {
                    font_id: FontId::monospace(14.0),
                    color,
                    ..Default::default()
                });
                value.clear();
            }
        };
        let chars: Vec<char> = text.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            if !in_string && !in_comment && ch == '-' && chars.get(index + 1) == Some(&'-') {
                flush(&mut job, &mut current, Color32::LIGHT_GRAY);
                in_comment = true;
                current.push(ch);
            } else if in_comment {
                current.push(ch);
                if ch == '\n' {
                    flush(&mut job, &mut current, Color32::from_rgb(105, 117, 134));
                    in_comment = false;
                }
            } else if ch == '\'' {
                current.push(ch);
                if in_string {
                    flush(&mut job, &mut current, Color32::from_rgb(231, 182, 90));
                    in_string = false;
                } else {
                    flush(&mut job, &mut current, Color32::from_rgb(231, 182, 90));
                    in_string = true;
                }
            } else if in_string {
                current.push(ch);
            } else if ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
            } else {
                let word = current.to_lowercase();
                let color = if keywords.contains(&word.as_str()) {
                    Color32::from_rgb(139, 140, 255)
                } else if current.chars().all(|value| value.is_ascii_digit()) && !current.is_empty() {
                    Color32::from_rgb(53, 196, 138)
                } else {
                    Color32::from_rgb(243, 245, 247)
                };
                flush(&mut job, &mut current, color);
                job.append(&ch.to_string(), 0.0, TextFormat {
                    font_id: FontId::monospace(14.0),
                    color: Color32::from_rgb(243, 245, 247),
                    ..Default::default()
                });
            }
            index += 1;
        }
        if in_string {
            flush(&mut job, &mut current, Color32::from_rgb(231, 182, 90));
        } else if in_comment {
            flush(&mut job, &mut current, Color32::from_rgb(105, 117, 134));
        } else {
            let word = current.to_lowercase();
            let color = if keywords.contains(&word.as_str()) {
                Color32::from_rgb(139, 140, 255)
            } else {
                Color32::from_rgb(243, 245, 247)
            };
            flush(&mut job, &mut current, color);
        }
        ui.fonts(|fonts| fonts.layout_job(job))
    }

    fn format_sql(sql: &str) -> String {
        let keywords = ["select", "from", "where", "group by", "order by", "limit", "values", "set"];
        let mut formatted = sql.trim().to_owned();
        for keyword in keywords {
            formatted = formatted.replace(keyword, &keyword.to_uppercase());
        }
        formatted = formatted.replace(" FROM ", "\nFROM ")
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
        if !current.is_empty() { tokens.push(current.to_lowercase()); }
        if in_string { diagnostics.push("Unclosed string literal".to_owned()); }
        if parentheses > 0 { diagnostics.push("Unclosed parenthesis".to_owned()); }
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
        let driver = self.connections.first().map(|connection| connection.driver.as_str()).unwrap_or("postgres");
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
        ui.horizontal(|ui| {
            ui.label(RichText::new("Query").strong());
            ui.label(RichText::new("›  ").color(self.theme.text_muted));
            ui.label(RichText::new(self.connection_name.as_str()).color(self.theme.accent));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let running = self.next_query_request.is_some();
                if ui.button(if running { "Stop  Esc" } else { "Run  ⌘↵" }).clicked() {
                    if let Some(request_id) = self.next_query_request {
                        self.cancel_query(request_id);
                    } else {
                        self.dispatch_query();
                    }
                }
                if ui.button("Format").clicked() {
                    self.query_text = Self::format_sql(&self.query_text);
                }
                ui.add(egui::TextEdit::singleline(&mut self.query_folder).hint_text("folder (optional)").desired_width(120.0));
                if ui.button("New folder").clicked() {
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
                if ui.button("Save").clicked() {
                    if let Some(connection) = self.connections.first() {
                        let request_id = self.task_bridge.next_request_id();
                        let name = self.query_documents.get(self.active_query_document).map(|document| document.title.clone()).unwrap_or_else(|| "Saved query".to_owned());
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
                if ui.button(if self.selected_query.is_empty() { "Run statement" } else { "Run selection" }).clicked() {
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
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.small_button("⌕ Search").clicked() {
                self.editor_search_open = !self.editor_search_open;
            }
            if ui.small_button("A−").clicked() {
                self.editor_font_size = (self.editor_font_size - 1.0).max(10.0);
            }
            if ui.small_button("A+").clicked() {
                self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
            }
            if ui.small_button("Completion").clicked() {
                self.completion_open = !self.completion_open;
            }
            if ui.small_button("Snippets").clicked() {
                self.snippets_open = !self.snippets_open;
            }
            ui.label(RichText::new(format!("{} px", self.editor_font_size)).small().color(self.theme.text_muted));
            if self.editor_search_open {
                ui.add_sized([220.0, 24.0], TextEdit::singleline(&mut self.editor_search).hint_text("Find in SQL…"));
                if !self.editor_search.is_empty() {
                    let matches = self.query_text.matches(&self.editor_search).count();
                    ui.label(RichText::new(format!("{matches} matches")).small().color(self.theme.text_muted));
                }
            }
        });
        egui::Frame::default().fill(self.theme.surface_panel).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let line_count = self.query_text.lines().count().max(1);
                ui.vertical(|ui| {
                    for line in 1..=line_count {
                        ui.label(RichText::new(format!("{line:>3}")).monospace().color(self.theme.text_muted));
                    }
                });
                ui.separator();
                let editor_size = egui::vec2(ui.available_width(), 220.0);
                let output = ui.allocate_ui(editor_size, |ui| {
                    TextEdit::multiline(&mut self.query_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(10)
                        .layouter(&mut |ui, text, wrap_width| Self::sql_layouter(ui, text, wrap_width))
                        .lock_focus(true)
                        .show(ui)
                });
                if let Some(cursor_range) = output.inner.cursor_range {
                    let range = cursor_range.as_sorted_char_range();
                    if range.start < range.end && range.end <= self.query_text.len() {
                        self.selected_query = self.query_text.chars().skip(range.start).take(range.end - range.start).collect();
                    } else {
                        self.selected_query.clear();
                    }
                }
            });
        });
        if self.completion_open {
            egui::Frame::default().fill(self.theme.surface_elevated).show(ui, |ui| {
                ui.label(RichText::new("SQL completion").strong());
                let is_sqlite = self.connections.first().map(|connection| connection.driver.eq_ignore_ascii_case("sqlite")).unwrap_or(false);
                let mut candidates = vec!["SELECT".to_owned(), "FROM".to_owned(), "WHERE".to_owned(), "JOIN".to_owned(), "GROUP BY".to_owned(), "ORDER BY".to_owned(), "LIMIT".to_owned(), "COUNT(*)".to_owned()];
                if is_sqlite {
                    candidates.extend(["GLOB", "strftime", "WITHOUT ROWID"].into_iter().map(str::to_owned));
                } else {
                    candidates.extend(["ILIKE", "RETURNING", "jsonb_build_object"].into_iter().map(str::to_owned));
                }
                candidates.extend(self.schema.tables.iter().cloned());
                candidates.extend(self.schema.columns.iter().cloned());
                for keyword in candidates.iter() {
                    if ui.selectable_label(false, keyword).on_hover_text("Insert SQL keyword or expression").clicked() {
                        self.query_text.push_str(keyword);
                        self.completion_open = false;
                    }
                }
            });
        }
        if self.snippets_open {
            egui::Frame::default().fill(self.theme.surface_elevated).show(ui, |ui| {
                ui.label(RichText::new("SQL snippets").strong());
                if ui.button("SELECT table").clicked() {
                    self.insert_snippet("SELECT *\nFROM table_name\nLIMIT 100;");
                    self.snippets_open = false;
                }
                if ui.button("UPDATE by primary key").clicked() {
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
        ui.horizontal(|ui| {
            ui.label(RichText::new("Results").strong());
            let row_label = result
                .as_ref()
                .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                .unwrap_or_else(|| "No result".to_owned());
            ui.label(RichText::new(row_label).small().color(self.theme.text_muted));
            ui.label(RichText::new(self.runtime_message.as_str()).small().color(self.theme.text_muted));
        });
        ui.add_space(8.0);
        egui::Frame::default().fill(self.theme.surface_panel).show(ui, |ui| {
            if let Some(result) = result {
                self.draw_result_grid(ui, &result);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Run a query to see results").color(self.theme.text_muted));
                });
            }
        });
    }

    fn draw_result_grid(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if result.columns.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Statement completed without rows").color(self.theme.text_muted));
            });
            return;
        }

        if ui.input(|input| input.key_pressed(egui::Key::C) && input.modifiers.command) {
            self.copy_selected_cell(ui, result);
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_secondary));
            ui.add_sized(
                [220.0, 24.0],
                egui::TextEdit::singleline(&mut self.grid_filter).hint_text("Search visible rows…"),
            );
            if ui.small_button("Clear").clicked() {
                self.grid_filter.clear();
            }
            if ui.small_button("Copy cell").clicked() {
                self.copy_selected_cell(ui, result);
            }
            if ui.small_button("Copy row").clicked() {
                self.copy_selected_row(ui, result);
            }
            if !self.copy_status.is_empty() {
                ui.label(RichText::new(self.copy_status.as_str()).small().color(self.theme.success));
            }
            ui.label(RichText::new("Click a cell to select · drag the divider to resize").small().color(self.theme.text_muted));
        });
        ui.add_space(6.0);

        let indexes = self.filtered_sorted_indexes(result);
        ui.label(RichText::new(format!("{} matching rows · virtualized", indexes.len())).small().color(self.theme.text_muted));
        ui.add_space(4.0);

        let widths = self.column_widths(result.columns.len());
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.set_min_width(widths.iter().sum());
            self.draw_grid_header(ui, result, &widths);
            egui::ScrollArea::vertical().max_height(250.0).show_rows(ui, 24.0, indexes.len(), |ui, range| {
                for position in range {
                    let row_index = indexes[position];
                    let row = &result.rows[row_index];
                    ui.horizontal(|ui| {
                        for (column_index, cell) in row.iter().enumerate().take(result.columns.len()) {
                            let width = widths.get(column_index).copied().unwrap_or(180.0);
                            let fill = if position % 2 == 0 { self.theme.surface_panel } else { self.theme.surface_elevated };
                            egui::Frame::default().fill(fill).show(ui, |ui| {
                                ui.allocate_ui_with_layout(egui::vec2(width, 24.0), Layout::left_to_right(Align::Center), |ui| {
                                    ui.add_space(8.0);
                                    let selected = self.selected_cell == Some((row_index, column_index));
                                    let response = ui.add_sized(
                                        [width - 12.0, 22.0],
                                        egui::SelectableLabel::new(selected, Self::cell_label(cell)),
                                    );
                                    if response.clicked() {
                                        self.selected_cell = Some((row_index, column_index));
                                        self.selected_row = Some(row_index);
                                        self.copy_status.clear();
                                    }
                                });
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
        ui.output_mut(|output| output.copied_text = Self::cell_text(cell));
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
        let row_text = row.iter().map(Self::cell_text).collect::<Vec<_>>().join("\t");
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
            for (index, column) in result.columns.iter().enumerate() {
                let width = widths.get(index).copied().unwrap_or(180.0);
                let sort_marker = match self.grid_sort_column {
                    Some(active) if active == index && self.grid_sort_desc => " ↓",
                    Some(active) if active == index => " ↑",
                    _ => "",
                };
                let response = ui.add_sized(
                    [width - 4.0, 28.0],
                    egui::Button::new(RichText::new(format!("{}{}", column.name, sort_marker)).strong())
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
                let (_divider_rect, divider) = ui.allocate_exact_size(egui::vec2(8.0, 28.0), Sense::drag());
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

    fn filtered_sorted_indexes(&self, result: &UiQueryResult) -> Vec<usize> {
        let filter = self.grid_filter.to_lowercase();
        let mut indexes: Vec<usize> = result
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| filter.is_empty() || row.iter().any(|cell| Self::cell_text(cell).to_lowercase().contains(&filter)))
            .map(|(index, _)| index)
            .collect();
        if let Some(column) = self.grid_sort_column {
            indexes.sort_by(|left, right| {
                let left_value = result.rows[*left].get(column).map(Self::cell_text).unwrap_or_default();
                let right_value = result.rows[*right].get(column).map(Self::cell_text).unwrap_or_default();
                let ordering = left_value.cmp(&right_value);
                if self.grid_sort_desc { ordering.reverse() } else { ordering }
            });
        }
        indexes
    }

    fn cell_text(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "NULL".to_owned(),
            crate::UiCell::Boolean(value) => value.to_string(),
            crate::UiCell::Number(value) | crate::UiCell::Text(value) | crate::UiCell::Json(value) | crate::UiCell::Bytes(value) => value.clone(),
        }
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
            driver: if connection.driver == "SQLite" { UiDriver::Sqlite } else { UiDriver::Postgres },
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
        let Some(connection_id) = self.delete_confirmation_id.clone() else { return };
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
                    if ui.button("Delete").clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteConnection {
                            request_id,
                            connection_id: connection_id.clone(),
                        });
                        self.runtime_message = format!("Deleting {name}…");
                        self.delete_confirmation_id = None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            });
    }

    fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let title = if self.editing_connection_id.is_some() { "Edit connection" } else { "New connection" };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(if self.editing_connection_id.is_some() { "Update a safe database connection" } else { "Create a safe database connection" }).color(self.theme.text_secondary));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Driver");
                    ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Postgres, "PostgreSQL");
                    ui.selectable_value(&mut self.connection_draft.driver, UiDriver::Sqlite, "SQLite");
                });
                ui.add_space(6.0);
                Self::form_row(ui, "Name", &mut self.connection_draft.name, "Production DB");
                if self.connection_draft.driver == UiDriver::Postgres {
                    Self::form_row(ui, "Host", &mut self.connection_draft.host, "localhost");
                    Self::form_row(ui, "Port", &mut self.connection_draft.port, "5432");
                    Self::form_row(ui, "Database", &mut self.connection_draft.database, "app");
                    Self::form_row(ui, "Username", &mut self.connection_draft.username, "postgres");
                    ui.horizontal(|ui| {
                        ui.label("Password");
                        ui.add_sized([300.0, 24.0], egui::TextEdit::singleline(&mut self.connection_draft.password).password(true));
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
                            Self::form_row(ui, "SSH host", &mut self.connection_draft.ssh_host, "bastion.example.com");
                            Self::form_row(ui, "SSH port", &mut self.connection_draft.ssh_port, "22");
                            Self::form_row(ui, "SSH user", &mut self.connection_draft.ssh_user, "ubuntu");
                            ui.horizontal(|ui| {
                                Self::form_row(ui, "Private key", &mut self.connection_draft.ssh_private_key, "/home/me/.ssh/id_ed25519");
                                if ui.small_button("Browse…").clicked() {
                                    let request_id = self.task_bridge.next_request_id();
                                    let _ = self.task_bridge.send(UiCommand::PickSshPrivateKey { request_id });
                                }
                            });
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        Self::form_row(ui, "SQLite file", &mut self.connection_draft.database, "/path/to/db.sqlite");
                        if ui.small_button("Browse…").clicked() {
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
                    if ui.button("Test connection").clicked() {
                        self.dispatch_connection_command(false);
                    }
                    if ui.button(RichText::new("Save connection").color(self.theme.text_primary)).clicked() {
                        self.dispatch_connection_command(true);
                    }
                    if ui.button("Cancel").clicked() {
                        self.connection_dialog_open = false;
                    }
                });
            });
        self.connection_dialog_open = open && self.connection_dialog_open;
    }

    fn form_row(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add_sized([300.0, 24.0], egui::TextEdit::singleline(value).hint_text(hint));
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
        self.runtime_message = if save { "Saving connection…" } else { "Testing connection…" }.to_owned();
    }

    fn draw_agent_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("agent_panel")
            .default_width(320.0)
            .min_width(280.0)
            .frame(egui::Frame::default().fill(self.theme.surface_elevated))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Agent workspace").strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("×").clicked() {
                            self.agent_open = false;
                        }
                    });
                });
                ui.separator();
                ui.add_space(8.0);
                ui.label(RichText::new("Ask about your database or draft a safe query.").color(self.theme.text_secondary));
                ui.add_space(16.0);
                ui.label(RichText::new("Preview").small().strong().color(self.theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("The agent will always show its target and require confirmation before writes.").small().color(self.theme.text_muted));
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.add(TextEdit::singleline(&mut self.agent_input).hint_text("Ask the agent…"));
                });
            });
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
                vec![crate::UiCell::Number("2".to_owned()), crate::UiCell::Text("Beta".to_owned())],
                vec![crate::UiCell::Number("1".to_owned()), crate::UiCell::Text("Alpha".to_owned())],
                vec![crate::UiCell::Number("3".to_owned()), crate::UiCell::Text("Gamma".to_owned())],
            ],
            row_count: 3,
            duration_ms: 2,
        }
    }

    #[test]
    fn filter_returns_original_row_indexes() {
        let mut app = DbProApp::default();
        app.grid_filter = "gamma".to_owned();
        let value = result();
        assert_eq!(app.filtered_sorted_indexes(&value), vec![2]);
    }

    #[test]
    fn sort_is_stable_over_filtered_indexes() {
        let mut app = DbProApp::default();
        app.grid_sort_column = Some(0);
        let value = result();
        assert_eq!(app.filtered_sorted_indexes(&value), vec![1, 0, 2]);
        app.grid_sort_desc = true;
        assert_eq!(app.filtered_sorted_indexes(&value), vec![2, 0, 1]);
    }

    #[test]
    fn cell_text_keeps_null_and_json_visible() {
        assert_eq!(DbProApp::cell_text(&crate::UiCell::Null), "NULL");
        assert_eq!(
            DbProApp::cell_text(&crate::UiCell::Json("{\"ok\":true}".to_owned())),
            "{\"ok\":true}"
        );
    }
}
