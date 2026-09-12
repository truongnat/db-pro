use super::*;
use crate::components::{kbd_badge, Dialog};

impl DbProApp {
    /// Build the full palette item list for the given mode, including the
    /// schema-table and connection shortcuts appended per mode.
    fn palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let mut items = match mode {
            PaletteMode::QuickOpen => Self::quick_open_items(),
            PaletteMode::Commands => Self::command_items(),
        };
        if mode == PaletteMode::QuickOpen {
            items.extend(self.schema_table_items());
        }
        if mode == PaletteMode::Commands {
            items.extend(self.connection_items());
        }
        items
    }

    fn quick_open_items() -> Vec<PaletteItem> {
        vec![
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
                shortcut: Some(format!("{}K", Self::primary_modifier_label())),
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
                subtitle: "Open the database copilot".to_owned(),
                shortcut: None,
                action: PaletteAction::Agent,
            },
        ]
    }

    fn command_items() -> Vec<PaletteItem> {
        vec![
            PaletteItem {
                icon: Icon::Plus,
                title: "New query".to_owned(),
                subtitle: "Create a fresh SQL document".to_owned(),
                shortcut: None,
                action: PaletteAction::NewQuery,
            },
            PaletteItem {
                icon: Icon::Play,
                title: "Run query".to_owned(),
                subtitle: "Execute the current SQL or selection".to_owned(),
                shortcut: Some(format!("{}↵", Self::primary_modifier_label())),
                action: PaletteAction::RunQuery,
            },
            PaletteItem {
                icon: Icon::WandSparkles,
                title: "Format SQL".to_owned(),
                subtitle: "Format the active SQL document".to_owned(),
                shortcut: None,
                action: PaletteAction::FormatSql,
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
                subtitle: "Ask Agent about the active schema".to_owned(),
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
            PaletteItem {
                icon: Icon::ChartNoAxesCombined,
                title: "Explain query".to_owned(),
                subtitle: "Inspect a read-only query plan".to_owned(),
                shortcut: None,
                action: PaletteAction::ExplainQuery,
            },
            PaletteItem {
                icon: Icon::Download,
                title: "Export results".to_owned(),
                subtitle: "Open export options for the current result".to_owned(),
                shortcut: None,
                action: PaletteAction::ExportResults,
            },
            PaletteItem {
                icon: Icon::Palette,
                title: "Open Component Gallery".to_owned(),
                subtitle: "Preview DB Pro common UI design system".to_owned(),
                shortcut: None,
                action: PaletteAction::ComponentGallery,
            },
        ]
    }

    fn schema_table_items(&self) -> Vec<PaletteItem> {
        self.active_schema_table_names()
            .iter()
            .take(EXPLORER_MAX_TABLES)
            .cloned()
            .map(|table| PaletteItem {
                icon: Icon::Table2,
                title: table.clone(),
                subtitle: format!("Open table in {}", self.active_schema()),
                shortcut: None,
                action: PaletteAction::OpenTable(table),
            })
            .collect()
    }

    fn connection_items(&self) -> Vec<PaletteItem> {
        self.connections
            .iter()
            .cloned()
            .map(|connection| PaletteItem {
                icon: Icon::Database,
                title: format!("Switch to {}", connection.name),
                subtitle: format!("{} · {}", connection.driver, connection.database),
                shortcut: None,
                action: PaletteAction::SwitchConnection(connection.id),
            })
            .collect()
    }

    pub(crate) fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let query = self.palette_query.trim().to_lowercase();
        self.palette_items(mode)
            .into_iter()
            .filter(|item| {
                if query.is_empty() {
                    true
                } else {
                    item.title.to_lowercase().contains(&query)
                        || item.subtitle.to_lowercase().contains(&query)
                }
            })
            .collect()
    }

    pub(crate) fn execute_palette_action(&mut self, action: PaletteAction, _ctx: &egui::Context) {
        self.palette_mode = None;
        match action {
            PaletteAction::Welcome => self.active_tab = WorkspaceTab::Welcome,
            PaletteAction::Query => {
                self.active_tab = WorkspaceTab::Query;
            }
            PaletteAction::History => {
                self.activity = Activity::History;
                self.sidebar_open = true;
            }
            PaletteAction::Diagram => self.active_tab = WorkspaceTab::Diagram,
            PaletteAction::Settings => {
                self.activity = Activity::Settings;
                self.sidebar_open = true;
            }
            PaletteAction::Agent => {
                self.agent_open = true;
            }
            PaletteAction::NewQuery => {
                self.active_tab = WorkspaceTab::Query;
                self.new_query_document();
                self.runtime_message = "New query ready".to_owned();
            }
            PaletteAction::NewConnection => {
                self.open_new_connection();
            }
            PaletteAction::RefreshSchema => self.refresh_schema_palette(),
            PaletteAction::ToggleExplorer => self.sidebar_open = !self.sidebar_open,
            PaletteAction::OpenTable(table) => self.open_table_from_palette(table),
            PaletteAction::ExplainQuery => self.explain_query(),
            PaletteAction::ExportResults => self.export_results_from_palette(),
            PaletteAction::RunQuery => {
                self.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
            }
            PaletteAction::FormatSql => {
                self.active_tab = WorkspaceTab::Query;
                self.query_text = Self::format_sql(&self.query_text);
                self.runtime_message = "SQL formatted".to_owned();
            }
            PaletteAction::SwitchConnection(connection_id) => {
                self.switch_connection_from_palette(connection_id);
            }
            PaletteAction::ComponentGallery => {
                self.active_tab = WorkspaceTab::ComponentGallery;
            }
        }
    }

    fn refresh_schema_palette(&mut self) {
        if let Some(connection_id) = self.active_connection_id.clone() {
            self.refresh_table_info_after_schema = self.selected_table.is_some();
            self.request_schema_introspection(connection_id, true);
        } else {
            self.runtime_message = "Connect to a database before refreshing schema".to_owned();
        }
    }

    fn open_table_from_palette(&mut self, table: String) {
        self.selected_table = Some(table.clone());
        self.selected_schema_object = None;
        self.table_view = TableView::Structure;
        self.table_info = None;
        self.table_ddl = None;
        self.table_data_result = None;
        self.request_table_info();
        self.active_tab = WorkspaceTab::Table;
        self.runtime_message = format!("Opening table {table}");
    }

    fn export_results_from_palette(&mut self) {
        if self.query_result.is_some() {
            self.output_tab = OutputTab::Results;
            self.export_open = true;
            self.active_tab = WorkspaceTab::Query;
        } else {
            self.runtime_message = "Run a query before exporting results".to_owned();
        }
    }

    fn switch_connection_from_palette(&mut self, connection_id: String) {
        if let Some(connection) = self.connections.iter().find(|item| item.id == connection_id).cloned() {
            self.active_connection_id = Some(connection.id.clone());
            self.connected = false;
            let request_id = self.task_bridge.next_request_id();
            self.pending_connection_request = Some(request_id);
            let _ = self.task_bridge.send(UiCommand::Connect {
                request_id,
                connection_id: connection.id,
            });
            self.runtime_message = format!("Connecting to {}…", connection.name);
        }
    }

    pub(super) fn draw_palette(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.palette_mode else {
            return;
        };
        let items = self.filtered_palette_items(mode);
        self.clamp_palette_selection(&items);
        let mut activate = false;
        let mut open = true;
        let title = if mode == PaletteMode::QuickOpen {
            "Quick Open"
        } else {
            "Command Palette"
        };
        let description = if mode == PaletteMode::QuickOpen {
            "Switch workspaces, tabs, or open editors"
        } else {
            "Search commands, actions, and database tools"
        };

        egui::Area::new(egui::Id::new("palette_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, title, self.theme)
                    .description(description)
                    .width(580.0)
                    .id_salt("palette_dialog")
                    .show(ui, |ui| {
                        let response = ui.add(
                            TextEdit::singleline(&mut self.palette_query)
                                .hint_text(RichText::new("Type a command or search…").color(self.theme.text_muted))
                                .desired_width(ui.available_width())
                                .margin(egui::Margin::symmetric(12.0, 8.0))
                                .font(egui::FontId::proportional(13.5))
                                .text_color(self.theme.text_primary),
                        );
                        if self.palette_focus_requested {
                            response.request_focus();
                            self.palette_focus_requested = false;
                        }

                        if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) && !items.is_empty() {
                            self.palette_selected = (self.palette_selected + 1) % items.len();
                        }
                        if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) && !items.is_empty() {
                            self.palette_selected = if self.palette_selected == 0 {
                                items.len() - 1
                            } else {
                                self.palette_selected - 1
                            };
                        }
                        if ui.input(|input| input.key_pressed(egui::Key::Enter)) && !items.is_empty() {
                            activate = true;
                        }

                        ui.add_space(8.0);
                        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                            if items.is_empty() {
                                ui.add_space(16.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("No matching commands found").color(self.theme.text_muted));
                                });
                                ui.add_space(16.0);
                            }
                            for (index, item) in items.iter().enumerate() {
                                let selected = index == self.palette_selected;
                                let item_fill = if selected {
                                    self.theme.surface_hover
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                let (rect, item_resp) = ui
                                    .allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
                                if item_resp.hovered() {
                                    self.palette_selected = index;
                                }
                                if item_resp.clicked() {
                                    self.palette_selected = index;
                                    activate = true;
                                }

                                if selected || item_resp.hovered() {
                                    ui.painter().rect_filled(rect, egui::Rounding::same(6.0), item_fill);
                                    if selected {
                                        ui.painter().rect_stroke(
                                            rect,
                                            egui::Rounding::same(6.0),
                                            egui::Stroke::new(1.0, self.theme.border_subtle),
                                        );
                                    }
                                }

                                // Icon
                                let icon_char = char::from(item.icon).to_string();
                                ui.painter().text(
                                    egui::pos2(rect.left() + 12.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    icon_char,
                                    egui::FontId::new(15.0, egui::FontFamily::Name("lucide".into())),
                                    if selected {
                                        self.theme.text_primary
                                    } else {
                                        self.theme.text_secondary
                                    },
                                );

                                // Title and Subtitle
                                let text_x = rect.left() + 38.0;
                                ui.painter().text(
                                    egui::pos2(text_x, rect.center().y - 8.0),
                                    egui::Align2::LEFT_CENTER,
                                    &item.title,
                                    crate::DbProTheme::ui_medium_font(13.0),
                                    self.theme.text_primary,
                                );
                                ui.painter().text(
                                    egui::pos2(text_x, rect.center().y + 8.0),
                                    egui::Align2::LEFT_CENTER,
                                    &item.subtitle,
                                    egui::FontId::proportional(11.5),
                                    self.theme.text_muted,
                                );

                                if let Some(shortcut) = &item.shortcut {
                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                                            egui::pos2(rect.right() - 80.0, rect.top()),
                                            rect.right_bottom(),
                                        )),
                                        |ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.add_space(8.0);
                                                kbd_badge(ui, shortcut, self.theme);
                                            });
                                        },
                                    );
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("↑↓ Navigate · ↵ Select · Esc Close")
                                    .size(11.0)
                                    .color(self.theme.text_muted),
                            );
                        });
                    });
            });

        if !open {
            self.palette_mode = None;
        }

        if activate {
            if let Some(item) = items.get(self.palette_selected) {
                self.execute_palette_action(item.action.clone(), ctx);
            }
        }
    }

    fn clamp_palette_selection(&mut self, items: &[PaletteItem]) {
        if items.is_empty() {
            self.palette_selected = 0;
        } else {
            self.palette_selected = self.palette_selected.min(items.len() - 1);
        }
    }
}
