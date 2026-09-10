use crate::{
    DbProTheme, TaskBridge, UiCommand, UiConnectionSummary, UiEvent, UiQueryResult,
};
use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, TextEdit, TopBottomPanel};

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
    connections: Vec<UiConnectionSummary>,
    active_connection_id: Option<String>,
    connections_requested: bool,
}

impl DbProApp {
    pub fn with_task_bridge(task_bridge: TaskBridge) -> Self {
        Self {
            task_bridge,
            ..Self::default()
        }
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
            connections: Vec::new(),
            active_connection_id: None,
            connections_requested: false,
        }
    }
}

impl eframe::App for DbProApp {
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
    }
}

impl DbProApp {
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
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.next_query_request {
                self.cancel_query(request_id);
            } else {
                self.agent_open = false;
            }
        }
    }

    fn cancel_query(&mut self, request_id: crate::RequestId) {
        let _ = self.task_bridge.send(UiCommand::CancelQuery { request_id });
        self.runtime_message = "Cancelling query…".to_owned();
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
                });
            }
        }
        ui.add_space(12.0);
        for (label, icon) in [("Schemas", "◫"), ("Tables", "▦"), ("Views", "◌"), ("Functions", "ƒ")]
        {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(RichText::new("›").color(self.theme.text_muted));
                ui.label(RichText::new(icon).color(self.theme.text_secondary));
                ui.label(RichText::new(label).color(self.theme.text_secondary));
            });
            ui.add_space(5.0);
        }
        ui.add_space(16.0);
        if ui.button("＋  New connection").clicked() {
            self.connected = !self.connected;
        }
    }

    fn draw_history(&self, ui: &mut egui::Ui) {
        for (title, meta) in [("select customers", "2 min ago"), ("show active users", "1 hour ago"), ("explain orders", "yesterday")] {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).color(self.theme.text_secondary));
                ui.label(RichText::new(meta).small().color(self.theme.text_muted));
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
            let query = ui.selectable_label(self.active_tab == WorkspaceTab::Query, "◉  Query  ×");
            if query.clicked() {
                self.active_tab = WorkspaceTab::Query;
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
                self.active_tab = WorkspaceTab::Query;
            }
            ui.add_space(10.0);
            ui.label(RichText::new("⌘ P to open anything  ·  ⌘ B to toggle explorer").small().color(self.theme.text_muted));
        });
    }

    fn draw_query(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Query").strong());
            ui.label(RichText::new("›  ").color(self.theme.text_muted));
            ui.label(RichText::new(self.connection_name.as_str()).color(self.theme.accent));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let running = self.next_query_request.is_some();
                if ui.button(if running { "Stop  Esc" } else { "Run  ⌘↵" }).clicked() {
                    if let Some(request_id) = self.next_query_request {
                        self.cancel_query(request_id);
                    } else if let Some(connection) = self.connections.first() {
                        let request_id = self.task_bridge.next_request_id();
                        self.next_query_request = Some(request_id);
                        self.runtime_message = "Sending query to runtime…".to_owned();
                        let _ = self.task_bridge.send(UiCommand::RunQuery {
                            request_id,
                            connection_id: connection.id.clone(),
                            sql: self.query_text.clone(),
                        });
                    } else {
                        self.runtime_message = "Create or select a connection first".to_owned();
                    }
                }
                ui.button("Format");
            });
        });
        ui.add_space(10.0);
        egui::Frame::default().fill(self.theme.surface_panel).show(ui, |ui| {
            ui.add_sized(
                [ui.available_width(), 180.0],
                TextEdit::multiline(&mut self.query_text)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(8)
                    .lock_focus(true),
            );
        });
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

        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_secondary));
            ui.add_sized(
                [220.0, 24.0],
                egui::TextEdit::singleline(&mut self.grid_filter).hint_text("Search visible rows…"),
            );
            if ui.small_button("Clear").clicked() {
                self.grid_filter.clear();
            }
            ui.label(RichText::new("Click a column to sort · drag the divider to resize").small().color(self.theme.text_muted));
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
                        for (column_index, cell) in row.0.iter().enumerate().take(result.columns.len()) {
                            let width = widths.get(column_index).copied().unwrap_or(180.0);
                            let fill = if position % 2 == 0 { self.theme.surface_panel } else { self.theme.surface_elevated };
                            egui::Frame::default().fill(fill).show(ui, |ui| {
                                ui.allocate_ui_with_layout(egui::vec2(width, 24.0), Layout::left_to_right(Align::Center), |ui| {
                                    ui.add_space(8.0);
                                    ui.label(Self::cell_label(cell));
                                });
                            });
                        }
                    });
                }
            });
        });
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
            .filter(|(_, row)| filter.is_empty() || row.0.iter().any(|cell| Self::cell_text(cell).to_lowercase().contains(&filter)))
            .map(|(index, _)| index)
            .collect();
        if let Some(column) = self.grid_sort_column {
            indexes.sort_by(|left, right| {
                let left_value = result.rows[*left].0.get(column).map(Self::cell_text).unwrap_or_default();
                let right_value = result.rows[*right].0.get(column).map(Self::cell_text).unwrap_or_default();
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
