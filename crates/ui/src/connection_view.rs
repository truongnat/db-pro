use super::*;
use egui::{pos2, vec2, Align2, FontFamily, FontId, Margin, Rect, Rounding, Stroke};
use lucide_icons::Icon;

struct DriverCardProps<'a> {
    icon: Icon,
    name: &'a str,
    subtitle: &'a str,
    badge: &'a str,
    is_selected: bool,
    is_disabled: bool,
    width: f32,
}

fn draw_driver_card(ui: &mut egui::Ui, props: DriverCardProps<'_>, theme: &DbProTheme) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(props.width, 54.0), egui::Sense::click());
    let is_hovered = resp.hovered() && !props.is_disabled;
    let painter = ui.painter();

    let bg_fill = if props.is_selected {
        theme.accent_soft
    } else if props.is_disabled {
        theme.surface_editor
    } else if is_hovered {
        theme.surface_hover
    } else {
        theme.surface_panel
    };

    let border_stroke = if props.is_selected {
        Stroke::new(1.5, theme.accent)
    } else if props.is_disabled {
        Stroke::new(1.0, theme.border_subtle)
    } else if is_hovered {
        Stroke::new(1.0, theme.border_strong)
    } else {
        Stroke::new(1.0, theme.border_subtle)
    };

    painter.rect(rect, Rounding::same(RADIUS_CARD), bg_fill, border_stroke);

    // Left Icon (17px)
    let icon_color = if props.is_selected {
        theme.accent
    } else if props.is_disabled {
        theme.text_disabled
    } else {
        theme.text_secondary
    };
    painter.text(
        pos2(rect.min.x + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        char::from(props.icon).to_string(),
        FontId::new(17.0, FontFamily::Name("lucide".into())),
        icon_color,
    );

    // Title and Subtitle
    let text_x = rect.min.x + 36.0;
    let title_color = if props.is_selected {
        theme.text_primary
    } else if props.is_disabled {
        theme.text_disabled
    } else {
        theme.text_secondary
    };
    painter.text(
        pos2(text_x, rect.center().y - 7.0),
        Align2::LEFT_CENTER,
        props.name,
        DbProTheme::ui_medium_font(12.0),
        title_color,
    );

    let sub_color = if props.is_disabled {
        theme.text_disabled
    } else {
        theme.text_muted
    };
    painter.text(
        pos2(text_x, rect.center().y + 8.0),
        Align2::LEFT_CENTER,
        props.subtitle,
        FontId::proportional(10.0),
        sub_color,
    );

    // Right Badge / Check
    if props.is_selected {
        painter.text(
            pos2(rect.max.x - 10.0, rect.center().y),
            Align2::RIGHT_CENTER,
            char::from(Icon::Check).to_string(),
            FontId::new(13.0, FontFamily::Name("lucide".into())),
            theme.accent,
        );
    } else {
        let badge_w = (props.badge.len() as f32) * 5.5 + 8.0;
        let badge_rect = Rect::from_min_size(
            pos2(rect.max.x - badge_w - 6.0, rect.center().y - 7.0),
            vec2(badge_w, 14.0),
        );
        let badge_bg = if props.is_disabled {
            theme.surface_panel
        } else {
            theme.surface_hover
        };
        let badge_fg = if props.is_disabled {
            theme.text_disabled
        } else {
            theme.text_muted
        };
        painter.rect_filled(badge_rect, Rounding::same(3.0), badge_bg);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            props.badge,
            FontId::proportional(8.5),
            badge_fg,
        );
    }

    if props.is_disabled {
        resp.on_hover_cursor(egui::CursorIcon::NotAllowed)
            .on_hover_text(format!("{} provider is coming soon in a future release", props.name))
    } else {
        resp
    }
}

impl DbProApp {
    pub(crate) fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        self.pending_connection_request = None;
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
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.connection_dialog_open = true;
    }

    pub(super) fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(connection_id) = self.delete_confirmation_id.clone() else {
            return;
        };
        let name = self
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "this connection".to_owned());
        let mut open = true;
        egui::Area::new(egui::Id::new("delete_conn_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, "Delete Connection", self.theme)
                    .width(420.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Delete \"{name}\" and its saved credentials?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "This action cannot be undone.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Delete Connection", self.theme).clicked() {
                                let request_id = self.task_bridge.next_request_id();
                                let _ = self.task_bridge.send(UiCommand::DeleteConnection {
                                    request_id,
                                    connection_id: connection_id.clone(),
                                });
                                self.pending_connection_request = Some(request_id);
                                self.runtime_message = format!("Deleting {name}…");
                                self.delete_confirmation_id = None;
                            }
                            if compact_button(ui, "Cancel", self.theme).clicked() {
                                self.delete_confirmation_id = None;
                            }
                        });
                    });
            });
        if !open {
            self.delete_confirmation_id = None;
        }
    }

    pub(super) fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
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
        egui::Area::new(egui::Id::new("delete_folder_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, "Delete Query Folder", self.theme)
                    .width(420.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Delete \"{folder_name}\" and its saved-query links?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "Saved queries in this folder will become unfiled.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Delete Folder", self.theme).clicked() {
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
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }

    pub(super) fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let draft_before = self.connection_draft.clone();
        let title = if self.editing_connection_id.is_some() {
            "Edit Connection"
        } else {
            "New Connection"
        };
        egui::Area::new(egui::Id::new("connection_dialog_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, title, self.theme)
                    .width(780.0)
                    .id_salt("connection_form_dialog")
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
                            self.draw_connection_form(ui);
                        });
                    });
            });
        self.connection_dialog_open = open && self.connection_dialog_open;
        if self.connection_draft != draft_before {
            self.connection_test_valid = false;
            self.connection_error.clear();
            self.runtime_message = "Connection changed · test again before saving".to_owned();
        }
        if !self.connection_dialog_open {
            self.pending_connection_request = None;
        }
    }

    fn draw_connection_form(&mut self, ui: &mut egui::Ui) {
        use crate::components::alert::{Alert, AlertVariant};
        use crate::components::button::{Button, ButtonVariant};
        use crate::components::input::Input;

        ui.label(
            RichText::new("Choose your database engine and configure connection credentials.")
                .font(font_caption())
                .color(self.theme.text_secondary),
        );
        ui.add_space(SPACE_MD);

        // ── 1. Database Engine Selection Cards (Grid: 4 cols x 2 rows) ─
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("DATABASE ENGINE")
                    .font(DbProTheme::ui_medium_font(10.5))
                    .color(self.theme.text_muted),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("2 active · 6 coming soon")
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(SPACE_XS);

        let gap = 8.0;
        let card_w = (ui.available_width() - 3.0 * gap) / 4.0;

        // Row 1: PostgreSQL, SQLite, MySQL, MariaDB
        ui.horizontal(|ui| {
            let is_pg = self.connection_draft.driver == UiDriver::Postgres;
            if draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Database,
                    name: "PostgreSQL",
                    subtitle: "Port 5432 · SQL",
                    badge: "Active",
                    is_selected: is_pg,
                    is_disabled: false,
                    width: card_w,
                },
                &self.theme,
            )
            .clicked()
            {
                self.connection_draft.driver = UiDriver::Postgres;
            }

            ui.add_space(gap);

            let is_sqlite = self.connection_draft.driver == UiDriver::Sqlite;
            if draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::FileCode,
                    name: "SQLite",
                    subtitle: "Embedded File",
                    badge: "Active",
                    is_selected: is_sqlite,
                    is_disabled: false,
                    width: card_w,
                },
                &self.theme,
            )
            .clicked()
            {
                self.connection_draft.driver = UiDriver::Sqlite;
            }

            ui.add_space(gap);

            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Database,
                    name: "MySQL",
                    subtitle: "Port 3306 · SQL",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );

            ui.add_space(gap);

            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Database,
                    name: "MariaDB",
                    subtitle: "Port 3306 · SQL",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );
        });

        ui.add_space(gap);

        // Row 2: Redis, MongoDB, ClickHouse, DuckDB
        ui.horizontal(|ui| {
            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Layers,
                    name: "Redis",
                    subtitle: "Port 6379 · KV",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );

            ui.add_space(gap);

            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Boxes,
                    name: "MongoDB",
                    subtitle: "Port 27017 · NoSQL",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );

            ui.add_space(gap);

            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::BarChart3,
                    name: "ClickHouse",
                    subtitle: "Port 8123 · OLAP",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );

            ui.add_space(gap);

            draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::FileSpreadsheet,
                    name: "DuckDB",
                    subtitle: "In-Process OLAP",
                    badge: "Soon",
                    is_selected: false,
                    is_disabled: true,
                    width: card_w,
                },
                &self.theme,
            );
        });

        ui.add_space(SPACE_LG);

        // ── 2. General Parameters ──────────────────────────────────────
        ui.label(
            RichText::new("CONNECTION SETTINGS")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);

        Input::new(&mut self.connection_draft.name, "e.g. Main Production DB", self.theme)
            .label("Connection Display Name")
            .leading_icon(Icon::Tag)
            .clearable(true)
            .show(ui);
        ui.add_space(SPACE_MD);

        // ── 3. Engine-specific Fields ─────────────────────────────────
        if self.connection_draft.driver == UiDriver::Postgres {
            self.draw_postgres_connection_fields(ui);
        } else {
            self.draw_sqlite_connection_fields(ui);
        }

        // ── 4. Safety & Read-only Options ─────────────────────────────
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.connection_draft.readonly, "Read-only connection");
            ui.label(
                RichText::new("(Disallows INSERT, UPDATE, DELETE, DDL mutations)")
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        });

        // ── 5. Feedback Alerts ─────────────────────────────────────────
        if !self.connection_error.is_empty() {
            ui.add_space(SPACE_SM);
            Alert::new("Configuration Error", &self.connection_error, self.theme)
                .variant(AlertVariant::Destructive)
                .show(ui);
        } else if self.connection_test_valid {
            ui.add_space(SPACE_SM);
            Alert::new(
                "Connection Verified",
                "Database server is reachable and validated.",
                self.theme,
            )
            .variant(AlertVariant::Success)
            .show(ui);
        }

        // ── 6. Action Footer ──────────────────────────────────────────
        ui.add_space(SPACE_MD);
        ui.separator();
        ui.add_space(SPACE_SM);

        ui.horizontal(|ui| {
            let is_testing = self.pending_connection_request.is_some();
            let test_btn = Button::new(self.theme)
                .text("Test Connection")
                .variant(ButtonVariant::Secondary)
                .icon(Icon::Zap)
                .loading(is_testing)
                .show(ui);

            if test_btn.clicked() && !is_testing {
                self.dispatch_connection_command(false);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let save_label = if self.editing_connection_id.is_some() {
                    "Update Connection"
                } else {
                    "Save Connection"
                };

                let save_btn = Button::new(self.theme)
                    .text(save_label)
                    .variant(ButtonVariant::Default)
                    .icon(Icon::Check)
                    .show(ui);

                if save_btn.clicked() {
                    self.dispatch_connection_command(true);
                }

                if Button::new(self.theme)
                    .text("Cancel")
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    self.connection_dialog_open = false;
                }
            });
        });
    }

    fn draw_postgres_connection_fields(&mut self, ui: &mut egui::Ui) {
        use crate::components::input::{Input, PasswordInput};
        use crate::components::tabs::SegmentedTabs;

        // Host & Port row
        ui.horizontal(|ui| {
            let avail = ui.available_width() - SPACE_SM;
            let host_w = avail * 0.72;
            let port_w = avail * 0.28;

            ui.vertical(|ui| {
                ui.set_width(host_w);
                Input::new(&mut self.connection_draft.host, "localhost", self.theme)
                    .label("Host / Server Address")
                    .leading_icon(Icon::Server)
                    .show(ui);
            });
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.set_width(port_w);
                Input::new(&mut self.connection_draft.port, "5432", self.theme)
                    .label("Port")
                    .leading_icon(Icon::Hash)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Database & Username row
        ui.horizontal(|ui| {
            let col_w = (ui.available_width() - SPACE_SM) * 0.5;

            ui.vertical(|ui| {
                ui.set_width(col_w);
                Input::new(&mut self.connection_draft.database, "postgres", self.theme)
                    .label("Database Name")
                    .leading_icon(Icon::Database)
                    .show(ui);
            });
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.set_width(col_w);
                Input::new(&mut self.connection_draft.username, "postgres", self.theme)
                    .label("Username")
                    .leading_icon(Icon::User)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Password with reveal toggle
        PasswordInput::new(
            &mut self.connection_draft.password,
            "Enter password",
            &mut self.connection_show_password,
            self.theme,
        )
        .label("Password")
        .show(ui);
        ui.add_space(SPACE_SM);

        // SSL Mode Segmented Tabs
        ui.label(
            RichText::new("SSL Mode")
                .size(12.0)
                .strong()
                .color(self.theme.text_secondary),
        );
        ui.add_space(SPACE_XXS);
        let mut ssl_idx = match self.connection_draft.ssl_mode {
            UiSslMode::Disable => 0,
            UiSslMode::Require => 1,
            UiSslMode::VerifyCa => 2,
            UiSslMode::VerifyFull => 3,
        };
        SegmentedTabs::new(
            &mut ssl_idx,
            &["Disable", "Require", "Verify CA", "Verify Full"],
            self.theme,
        )
        .show(ui);
        self.connection_draft.ssl_mode = match ssl_idx {
            0 => UiSslMode::Disable,
            1 => UiSslMode::Require,
            2 => UiSslMode::VerifyCa,
            _ => UiSslMode::VerifyFull,
        };
        ui.add_space(SPACE_MD);

        // SSH Bastion Tunnel Card
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::same(12.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Shield).to_string())
                        .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.add_space(SPACE_XS);
                ui.checkbox(
                    &mut self.connection_draft.ssh_tunnel_enabled,
                    RichText::new("Connect via SSH Bastion Tunnel")
                        .strong()
                        .color(self.theme.text_primary),
                );
            });

            if self.connection_draft.ssh_tunnel_enabled {
                ui.add_space(SPACE_SM);

                ui.horizontal(|ui| {
                    let avail = ui.available_width() - SPACE_SM;
                    ui.vertical(|ui| {
                        ui.set_width(avail * 0.72);
                        Input::new(&mut self.connection_draft.ssh_host, "bastion.example.com", self.theme)
                            .label("SSH Host")
                            .leading_icon(Icon::Server)
                            .show(ui);
                    });
                    ui.add_space(SPACE_SM);
                    ui.vertical(|ui| {
                        ui.set_width(avail * 0.28);
                        Input::new(&mut self.connection_draft.ssh_port, "22", self.theme)
                            .label("SSH Port")
                            .leading_icon(Icon::Hash)
                            .show(ui);
                    });
                });
                ui.add_space(SPACE_SM);

                Input::new(&mut self.connection_draft.ssh_user, "ubuntu", self.theme)
                    .label("SSH User")
                    .leading_icon(Icon::User)
                    .show(ui);
                ui.add_space(SPACE_SM);

                ui.label(
                    RichText::new("SSH Private Key")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    let btn_w = 96.0;
                    let input_w = (ui.available_width() - btn_w - SPACE_SM).max(120.0);
                    ui.vertical(|ui| {
                        ui.set_width(input_w);
                        Input::new(&mut self.connection_draft.ssh_private_key, "~/.ssh/id_rsa", self.theme)
                            .leading_icon(Icon::Key)
                            .show(ui);
                    });
                    ui.add_space(SPACE_SM);
                    if compact_button_with_icon(ui, Icon::FolderOpen, "Browse…", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::PickSshPrivateKey { request_id });
                    }
                });
            }
        });
    }

    fn draw_sqlite_connection_fields(&mut self, ui: &mut egui::Ui) {
        use crate::components::input::Input;

        ui.label(
            RichText::new("SQLite Database File")
                .size(12.0)
                .strong()
                .color(self.theme.text_secondary),
        );
        ui.add_space(SPACE_XXS);
        ui.horizontal(|ui| {
            let btn_w = 96.0;
            let input_w = (ui.available_width() - btn_w - SPACE_SM).max(120.0);
            ui.vertical(|ui| {
                ui.set_width(input_w);
                Input::new(&mut self.connection_draft.database, "/path/to/db.sqlite", self.theme)
                    .leading_icon(Icon::FolderArchive)
                    .show(ui);
            });
            ui.add_space(SPACE_SM);
            if compact_button_with_icon(ui, Icon::FolderOpen, "Browse…", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                let _ = self.task_bridge.send(UiCommand::PickSqliteFile { request_id });
            }
        });
    }

    pub(crate) fn dispatch_connection_command(&mut self, save: bool) {
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
        self.pending_connection_request = Some(request_id);
        if save {
            self.connection_test_valid = false;
        } else {
            self.connection_test_valid = false;
            self.connection_test_draft = Some(self.connection_draft.clone());
        }
        self.connection_error.clear();
        self.runtime_message = if save {
            "Saving connection…"
        } else {
            "Testing connection…"
        }
        .to_owned();
    }
}
