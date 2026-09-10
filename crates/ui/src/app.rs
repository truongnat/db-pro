use crate::{DbProTheme, TaskBridge, UiCommand, UiEvent};
use eframe::egui::{self, Align, Color32, Layout, RichText, TextEdit, TopBottomPanel};

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
/// a functional query surface. Backend commands are intentionally not wired
/// yet; this keeps the base independently reviewable and runnable.
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
        }
    }
}

impl eframe::App for DbProApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
    fn apply_runtime_events(&mut self) {
        let events: Vec<UiEvent> = self.task_bridge.drain_events().collect();
        for event in events {
            match event {
                UiEvent::QueryQueued { request_id } => {
                    self.next_query_request = Some(request_id);
                    self.runtime_message = format!("Query queued · request {}", request_id.0);
                }
                UiEvent::QueryCompleted { request_id, row_count } => {
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query completed · {row_count} rows");
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
            self.agent_open = false;
        }
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
            ui.label(RichText::new(&self.connection_name).strong());
            ui.label(RichText::new("PG").small().color(self.theme.accent));
        });
        ui.add_space(8.0);
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
            ui.label(RichText::new(&self.connection_name).color(self.theme.accent));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Run  ⌘↵").clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    self.next_query_request = Some(request_id);
                    self.runtime_message = "Sending query to runtime…".to_owned();
                    let _ = self.task_bridge.send(UiCommand::RunQuery {
                        request_id,
                        sql: self.query_text.clone(),
                    });
                    self.connected = true;
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
        ui.horizontal(|ui| {
            ui.label(RichText::new("Results").strong());
            ui.label(RichText::new("0 rows").small().color(self.theme.text_muted));
            ui.label(RichText::new(&self.runtime_message).small().color(self.theme.text_muted));
        });
        ui.add_space(8.0);
        egui::Frame::default().fill(self.theme.surface_panel).show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Run a query to see results").color(self.theme.text_muted));
            });
        });
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
