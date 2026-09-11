use super::*;

impl DbProApp {
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
                    shortcut: Some(format!("{}P", Self::primary_modifier_label())),
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
                    shortcut: Some(format!("{}B", Self::primary_modifier_label())),
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

    pub(crate) fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
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

    pub(crate) fn execute_palette_action(&mut self, action: PaletteAction, ctx: &egui::Context) {
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
            PaletteAction::NewConnection => self.open_new_connection(),
            PaletteAction::RefreshSchema => {
                if let Some(connection_id) = self.active_connection_id.clone() {
                    self.refresh_table_info_after_schema = self.selected_table.is_some();
                    self.request_schema_introspection(connection_id, true);
                } else {
                    self.runtime_message = "Connect to a database before refreshing schema".to_owned();
                }
            }
            PaletteAction::ToggleExplorer => self.sidebar_open = !self.sidebar_open,
        }
    }

    pub(super) fn draw_palette(&mut self, ctx: &egui::Context) {
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
                                        if let Some(shortcut) = &item.shortcut {
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(
                                                    RichText::new(shortcut.as_str())
                                                        .small()
                                                        .color(self.theme.text_muted),
                                                );
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
}
