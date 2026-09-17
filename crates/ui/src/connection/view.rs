use super::logic::*;
use super::mapper::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::input::{Input, PasswordInput};
use crate::components::tabs::SegmentedTabs;
use crate::tokens::*;
use crate::{DbProApp, DbProTheme, UiCommand, UiDriver, UiSslMode};
use egui::{pos2, vec2, Align2, FontFamily, FontId, Margin, Rect, Rounding, Stroke};
use lucide_icons::Icon;

/// In-UI qualification caveat for the SSH tunnel control (#239).
pub const SSH_QUALIFICATION_HINT: &str =
    "Unqualified in v0.1: the tunnel has not been end-to-end tested and may not work reliably.";

/// Per-mode guidance for the SSL selector (#144 locked contract).
pub fn ssl_mode_guidance(mode: UiSslMode) -> &'static str {
    match mode {
        UiSslMode::Disable => {
            "Plaintext — credentials and query traffic travel without TLS. Use only for localhost/dev."
        }
        UiSslMode::Require => "TLS encryption enabled, but server identity is not verified against a CA.",
        UiSslMode::VerifyCa => "CA validation enabled; hostname identity is weaker than Verify Full.",
        UiSslMode::VerifyFull => "Strongest available mode — recommended for remote production.",
    }
}

pub struct DriverCardProps<'a> {
    pub icon: Icon,
    pub name: &'a str,
    pub subtitle: &'a str,
    pub badge: &'a str,
    pub is_selected: bool,
    pub is_disabled: bool,
    pub width: f32,
}

pub fn draw_driver_card(ui: &mut egui::Ui, props: DriverCardProps<'_>, theme: &DbProTheme) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(props.width, 48.0), egui::Sense::click());
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

    // Left Icon (16px)
    let icon_color = if props.is_selected {
        theme.accent
    } else if props.is_disabled {
        theme.text_disabled
    } else {
        theme.text_secondary
    };
    painter.text(
        pos2(rect.min.x + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        char::from(props.icon).to_string(),
        FontId::new(16.0, FontFamily::Name("lucide".into())),
        icon_color,
    );

    // Title and Subtitle
    let text_x = rect.min.x + 34.0;
    let title_color = if props.is_selected {
        theme.text_primary
    } else if props.is_disabled {
        theme.text_disabled
    } else {
        theme.text_secondary
    };
    painter.text(
        pos2(text_x, rect.center().y - 6.0),
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
        FontId::proportional(9.5),
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
    pub(crate) fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
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
                            egui::RichText::new(format!("Delete \"{name}\" and its saved credentials?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "This action cannot be undone.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if Button::new(self.theme)
                                .text("Delete Connection")
                                .variant(ButtonVariant::Destructive)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::DeleteConnection {
                                    request_id,
                                    connection_id: connection_id.clone(),
                                });
                                self.pending_connection_request = Some(request_id);
                                self.runtime_message = format!("Deleting {name}…");
                                self.delete_confirmation_id = None;
                            }
                            if Button::new(self.theme)
                                .text("Cancel")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                self.delete_confirmation_id = None;
                            }
                        });
                    });
            });
        if !open {
            self.delete_confirmation_id = None;
        }
    }

    pub(crate) fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
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
                            egui::RichText::new(format!("Delete \"{folder_name}\" and its saved-query links?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "Saved queries in this folder will become unfiled.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if Button::new(self.theme)
                                .text("Delete Folder")
                                .variant(ButtonVariant::Destructive)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::DeleteQueryFolder {
                                    request_id,
                                    id: folder_id.clone(),
                                });
                                self.folder_delete_confirmation = None;
                            }
                            if Button::new(self.theme)
                                .text("Cancel")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                self.folder_delete_confirmation = None;
                            }
                        });
                    });
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }

    pub(crate) fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let draft_before = self.connection_draft.clone();
        let title = if self.editing_connection_id.is_some() {
            "Edit Connection"
        } else {
            "New Connection"
        };
        let desc = if self.editing_connection_id.is_some() {
            "Update database credentials, TLS parameters, and connection options."
        } else {
            "Select a database engine and configure connection parameters."
        };
        egui::Area::new(egui::Id::new("connection_dialog_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, title, self.theme)
                    .description(desc)
                    .width(820.0)
                    .id_salt("connection_form_dialog")
                    .show_framed(ui, |frame| {
                        frame.body(|ui| {
                            self.draw_connection_form(ui);
                        });
                        frame.footer(|ui| {
                            self.draw_connection_footer(ui);
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

    pub(crate) fn draw_connection_form(&mut self, ui: &mut egui::Ui) {
        // ── 1. Database Engine Selection Cards (Grid: 4 cols) ─────────
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("DATABASE ENGINE")
                    .font(DbProTheme::ui_medium_font(10.5))
                    .color(self.theme.text_muted),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("PostgreSQL · SQLite · MySQL · SQL Server")
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(SPACE_XS);

        let gap = 8.0;
        let card_w = (ui.available_width() - 3.0 * gap) / 4.0;

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
                select_driver(&mut self.connection_draft, UiDriver::Postgres);
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
                select_driver(&mut self.connection_draft, UiDriver::Sqlite);
            }

            ui.add_space(gap);

            let is_mysql = self.connection_draft.driver == UiDriver::Mysql;
            if draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Database,
                    name: "MySQL",
                    subtitle: "Port 3306 · SQL",
                    badge: "Active",
                    is_selected: is_mysql,
                    is_disabled: false,
                    width: card_w,
                },
                &self.theme,
            )
            .clicked()
            {
                select_driver(&mut self.connection_draft, UiDriver::Mysql);
            }

            ui.add_space(gap);

            if draw_driver_card(
                ui,
                DriverCardProps {
                    icon: Icon::Database,
                    name: "SQL Server",
                    subtitle: "Port 1433 · TDS",
                    badge: "Active",
                    is_selected: self.connection_draft.driver == UiDriver::SqlServer,
                    is_disabled: false,
                    width: card_w,
                },
                &self.theme,
            )
            .clicked()
            {
                select_driver(&mut self.connection_draft, UiDriver::SqlServer);
            }
        });

        ui.add_space(SPACE_MD);

        // ── 2. Engine-specific Fields ─────────────────────────────────
        match self.connection_draft.driver {
            UiDriver::Postgres | UiDriver::Mysql | UiDriver::SqlServer => self.draw_postgres_connection_fields(ui),
            UiDriver::Sqlite => self.draw_sqlite_connection_fields(ui),
        }

        // ── 3. Feedback Alerts ─────────────────────────────────────────
        if !self.connection_error.is_empty() {
            ui.add_space(SPACE_XS);
            Alert::new("Configuration Error", &self.connection_error, self.theme)
                .variant(AlertVariant::Destructive)
                .show(ui);
        } else if self.connection_test_valid {
            ui.add_space(SPACE_XS);
            Alert::new(
                "Connection Verified",
                "Database server is reachable and validated.",
                self.theme,
            )
            .variant(AlertVariant::Success)
            .show(ui);
        }
        if let Some(report) = &self.connection_diagnostics {
            ui.add_space(SPACE_XS);
            for stage in &report.stages {
                let mark = if stage.ok { "OK" } else { "FAIL" };
                let color = if stage.ok {
                    self.theme.success
                } else {
                    self.theme.danger
                };
                ui.label(
                    egui::RichText::new(format!("{mark} · {} · {}", stage.stage.label(), stage.message))
                        .small()
                        .monospace()
                        .color(color),
                );
            }
        }
    }

    pub(crate) fn draw_connection_footer(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn draw_postgres_connection_fields(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_width();
        let gap = SPACE_SM;

        // ── 1. General Profile Section ────────────────────────────────
        ui.label(
            egui::RichText::new("GENERAL")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        // Row 1: Connection Name (52%), Folder/Group (36%), Favorite (12%)
        let name_w = (avail - 2.0 * gap) * 0.52;
        let group_w = (avail - 2.0 * gap) * 0.36;
        let fav_w = (avail - 2.0 * gap) * 0.12;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(name_w);
                ui.set_max_width(name_w);
                Input::new(
                    &mut self.connection_draft.name,
                    "e.g. Production PostgreSQL",
                    self.theme,
                )
                .label("Connection Name")
                .width(name_w)
                .leading_icon(Icon::Tag)
                .clearable(true)
                .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(group_w);
                ui.set_max_width(group_w);
                Input::new(&mut self.connection_draft.group, "e.g. Acme / Local", self.theme)
                    .label("Folder / Group")
                    .width(group_w)
                    .leading_icon(Icon::Folder)
                    .clearable(true)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(fav_w);
                ui.set_max_width(fav_w);
                ui.label(
                    egui::RichText::new("Favorite")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
                let (fav_icon, fav_text) = if self.connection_draft.favorite {
                    (Icon::Star, "Saved")
                } else {
                    (Icon::Star, "Off")
                };
                if Button::new(self.theme)
                    .icon(fav_icon)
                    .text(fav_text)
                    .variant(if self.connection_draft.favorite {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.connection_draft.favorite = !self.connection_draft.favorite;
                }
            });
        });
        ui.add_space(SPACE_SM);

        // Row 2: Environment Selector & Read-only mode
        ui.horizontal(|ui| {
            let env_w = (avail - gap) * 0.55;
            let ro_w = (avail - gap) * 0.45;

            ui.vertical(|ui| {
                ui.set_width(env_w);
                ui.set_max_width(env_w);
                ui.label(
                    egui::RichText::new("Environment")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                let mut env_idx = match self.connection_draft.environment.as_str() {
                    "Staging" => 1,
                    "Production" => 2,
                    "Custom" => 3,
                    _ => 0,
                };
                SegmentedTabs::new(
                    &mut env_idx,
                    &["Development", "Staging", "Production", "Custom"],
                    self.theme,
                )
                .show(ui);
                self.connection_draft.environment = match env_idx {
                    1 => "Staging".to_owned(),
                    2 => "Production".to_owned(),
                    3 => "Custom".to_owned(),
                    _ => "Development".to_owned(),
                };
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(ro_w);
                ui.set_max_width(ro_w);
                ui.label(
                    egui::RichText::new("Safety Policy")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.connection_draft.readonly, "Read-only mode");
                    if self.connection_draft.environment == "Production" {
                        ui.label(
                            egui::RichText::new("⚠ Production guard")
                                .font(font_caption())
                                .color(self.theme.warning),
                        );
                    }
                });
            });
        });
        ui.add_space(SPACE_MD);

        // ── 2. Server & Credentials ──────────────────────────────────
        ui.label(
            egui::RichText::new("SERVER & CREDENTIALS")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        // Row 1: Host (68%) + Port (32%)
        let host_w = (avail - gap) * 0.68;
        let port_w = (avail - gap) * 0.32;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(host_w);
                ui.set_max_width(host_w);
                Input::new(&mut self.connection_draft.host, "localhost", self.theme)
                    .label("Host / Server Address")
                    .width(host_w)
                    .leading_icon(Icon::Server)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(port_w);
                ui.set_max_width(port_w);
                let default_port = match self.connection_draft.driver {
                    UiDriver::Mysql => "3306",
                    UiDriver::SqlServer => "1433",
                    _ => "5432",
                };
                Input::new(&mut self.connection_draft.port, default_port, self.theme)
                    .label("Port")
                    .width(port_w)
                    .leading_icon(Icon::Hash)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Row 2: Database Name (50%) + Username (50%)
        let half_w = (avail - gap) * 0.5;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let default_db = match self.connection_draft.driver {
                    UiDriver::Mysql => "mysql",
                    UiDriver::SqlServer => "master",
                    _ => "postgres",
                };
                Input::new(&mut self.connection_draft.database, default_db, self.theme)
                    .label("Database Name")
                    .width(half_w)
                    .leading_icon(Icon::Database)
                    .clearable(true)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let default_user = match self.connection_draft.driver {
                    UiDriver::Mysql => "root",
                    UiDriver::SqlServer => "sa",
                    _ => "postgres",
                };
                Input::new(&mut self.connection_draft.username, default_user, self.theme)
                    .label("Username")
                    .width(half_w)
                    .leading_icon(Icon::User)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        // Row 3: Password/Auth Token (50%) + SSL Mode (50%)
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                let pwd_placeholder = if self.connection_draft.auth_kind == "ephemeral_token" {
                    "Paste short-lived IAM/access token (not stored)"
                } else if self.editing_connection_id.is_some() {
                    "•••••••• (Leave blank to keep saved password)"
                } else {
                    "Optional (Leave blank if no password)"
                };
                PasswordInput::new(
                    &mut self.connection_draft.password,
                    pwd_placeholder,
                    &mut self.connection_show_password,
                    self.theme,
                )
                .label(if self.connection_draft.auth_kind == "ephemeral_token" {
                    "Access Token"
                } else {
                    "Password"
                })
                .width(half_w)
                .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(half_w);
                ui.set_max_width(half_w);
                ui.label(
                    egui::RichText::new("SSL / TLS Mode")
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
                ui.add_space(SPACE_XXS);
                let guidance_color = match self.connection_draft.ssl_mode {
                    UiSslMode::Disable => self.theme.danger,
                    UiSslMode::Require | UiSslMode::VerifyCa => self.theme.warning,
                    UiSslMode::VerifyFull => self.theme.text_muted,
                };
                ui.label(
                    egui::RichText::new(ssl_mode_guidance(self.connection_draft.ssl_mode))
                        .size(10.0)
                        .color(guidance_color),
                );
            });
        });

        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(char::from(Icon::ShieldCheck).to_string())
                    .font(FontId::new(11.0, FontFamily::Name("lucide".into())))
                    .color(self.theme.accent),
            );
            ui.label(
                egui::RichText::new("Credentials encrypted with AES-256-GCM in local vault.")
                    .size(11.0)
                    .color(self.theme.text_muted),
            );
        });

        ui.add_space(SPACE_MD);

        // ── 3. Advanced & Security Collapsibles ────────────────────────
        // Panel 1: Cloud Presets & URI Importer
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                egui::RichText::new("Cloud Presets & URI Importer")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            )
            .id_salt("conn_cloud_presets_panel")
            .show(ui, |ui| {
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    let selected = if self.connection_draft.cloud_preset.is_empty() {
                        "Select a cloud preset…".to_owned()
                    } else {
                        self.connection_draft.cloud_preset.clone()
                    };
                    egui::ComboBox::from_id_salt("cloud_preset")
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.connection_draft.cloud_preset,
                                String::new(),
                                "None (manual)",
                            );
                            for (key, label) in [
                                ("aws_rds:postgres", "AWS RDS · PostgreSQL"),
                                ("aws_rds:mysql", "AWS RDS · MySQL"),
                                ("aws_aurora:postgres", "AWS Aurora · PostgreSQL"),
                                ("gcp_cloudsql:postgres", "Cloud SQL · PostgreSQL"),
                                ("gcp_cloudsql:mysql", "Cloud SQL · MySQL"),
                                ("azure:postgres", "Azure Database · PostgreSQL"),
                                ("azure:mysql", "Azure Database · MySQL"),
                            ] {
                                ui.selectable_value(&mut self.connection_draft.cloud_preset, key.to_owned(), label);
                            }
                        });
                    if Button::new(self.theme)
                        .text("Apply Preset")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.apply_cloud_preset();
                    }
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    let mut auth_idx = if self.connection_draft.auth_kind == "ephemeral_token" {
                        1
                    } else {
                        0
                    };
                    SegmentedTabs::new(&mut auth_idx, &["Password", "Ephemeral token"], self.theme).show(ui);
                    self.connection_draft.auth_kind = if auth_idx == 1 {
                        "ephemeral_token".into()
                    } else {
                        "password".into()
                    };
                    if self.connection_draft.auth_kind == "ephemeral_token" {
                        ui.colored_label(
                            self.theme.warning,
                            "Token is session-only — never stored as a long-lived password",
                        );
                    }
                });
                ui.add_space(SPACE_XS);
                Input::new(
                    &mut self.connection_draft.cloud_snippet,
                    "postgresql://user:secret@host:5432/db?sslmode=verify-full",
                    self.theme,
                )
                .label("Paste Connection URI")
                .show(ui);
                ui.add_space(SPACE_XXS);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Import from URI")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        let snippet = self.connection_draft.cloud_snippet.clone();
                        match apply_connection_snippet(&mut self.connection_draft, &snippet) {
                            Ok(()) => self.connection_error.clear(),
                            Err(err) => self.connection_error = err,
                        }
                    }
                });
                if !self.connection_draft.cloud_guidance.is_empty() {
                    ui.add_space(SPACE_XXS);
                    ui.label(
                        egui::RichText::new(&self.connection_draft.cloud_guidance)
                            .small()
                            .color(self.theme.text_secondary),
                    );
                }
            });
        });

        // Panel 2: SSL / TLS Custom Certificates (if mode is CA or Full)
        if matches!(
            self.connection_draft.ssl_mode,
            UiSslMode::VerifyCa | UiSslMode::VerifyFull
        ) {
            ui.add_space(SPACE_SM);
            egui::Frame {
                fill: self.theme.surface_panel,
                stroke: Stroke::new(1.0, self.theme.border_subtle),
                rounding: Rounding::same(RADIUS_CARD),
                inner_margin: Margin::symmetric(12.0, 8.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("SSL Certificates")
                        .font(DbProTheme::ui_medium_font(11.5))
                        .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_XS);
                Input::new(
                    &mut self.connection_draft.ssl_root_cert_path,
                    "/path/to/ca.pem",
                    self.theme,
                )
                .label("Root CA Certificate Path")
                .leading_icon(Icon::FileCode)
                .show(ui);
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(half_w);
                        ui.set_max_width(half_w);
                        Input::new(
                            &mut self.connection_draft.ssl_client_cert_path,
                            "/path/to/client.crt",
                            self.theme,
                        )
                        .label("Client Certificate (optional)")
                        .width(half_w)
                        .show(ui);
                    });
                    ui.add_space(gap);
                    ui.vertical(|ui| {
                        ui.set_width(half_w);
                        ui.set_max_width(half_w);
                        Input::new(
                            &mut self.connection_draft.ssl_client_key_path,
                            "/path/to/client.key",
                            self.theme,
                        )
                        .label("Client Key (optional)")
                        .width(half_w)
                        .show(ui);
                    });
                });
            });
        }

        // Panel 3: SSH Bastion Tunnel (PostgreSQL, MySQL, SQL Server)
        ui.add_space(SPACE_SM);
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(char::from(Icon::Shield).to_string())
                        .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.add_space(SPACE_XXS);
                ui.checkbox(
                    &mut self.connection_draft.ssh_tunnel_enabled,
                    egui::RichText::new("Connect via SSH Bastion Tunnel")
                        .strong()
                        .color(self.theme.text_primary),
                );
            });

            ui.add_space(SPACE_XXS);
            ui.label(
                egui::RichText::new(SSH_QUALIFICATION_HINT)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );

            if self.connection_draft.ssh_tunnel_enabled {
                ui.add_space(SPACE_SM);

                ui.horizontal(|ui| {
                    let total = ui.available_width() - 3.0 * gap;
                    let ssh_host_w = total * 0.38;
                    let ssh_port_w = total * 0.14;
                    let ssh_user_w = total * 0.20;
                    let ssh_key_w = total * 0.28;

                    ui.vertical(|ui| {
                        ui.set_width(ssh_host_w);
                        ui.set_max_width(ssh_host_w);
                        Input::new(&mut self.connection_draft.ssh_host, "bastion.example.com", self.theme)
                            .label("SSH Host")
                            .width(ssh_host_w)
                            .leading_icon(Icon::Server)
                            .show(ui);
                    });
                    ui.add_space(gap);
                    ui.vertical(|ui| {
                        ui.set_width(ssh_port_w);
                        ui.set_max_width(ssh_port_w);
                        Input::new(&mut self.connection_draft.ssh_port, "22", self.theme)
                            .label("Port")
                            .width(ssh_port_w)
                            .leading_icon(Icon::Hash)
                            .show(ui);
                    });
                    ui.add_space(gap);
                    ui.vertical(|ui| {
                        ui.set_width(ssh_user_w);
                        ui.set_max_width(ssh_user_w);
                        Input::new(&mut self.connection_draft.ssh_user, "ubuntu", self.theme)
                            .label("SSH User")
                            .width(ssh_user_w)
                            .leading_icon(Icon::User)
                            .show(ui);
                    });
                    ui.add_space(gap);
                    ui.vertical(|ui| {
                        ui.set_width(ssh_key_w);
                        ui.set_max_width(ssh_key_w);
                        ui.label(
                            egui::RichText::new("Private Key")
                                .size(12.0)
                                .strong()
                                .color(self.theme.text_secondary),
                        );
                        ui.add_space(SPACE_XXS);
                        ui.horizontal(|ui| {
                            let browse_w = 68.0;
                            let key_input_w = (ssh_key_w - browse_w - SPACE_XS).max(50.0);
                            ui.vertical(|ui| {
                                ui.set_width(key_input_w);
                                ui.set_max_width(key_input_w);
                                Input::new(&mut self.connection_draft.ssh_private_key, "~/.ssh/id_rsa", self.theme)
                                    .width(key_input_w)
                                    .leading_icon(Icon::Key)
                                    .show(ui);
                            });
                            ui.add_space(SPACE_XS);
                            if Button::new(self.theme)
                                .icon(Icon::FolderOpen)
                                .text("Browse")
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::PickSshPrivateKey { request_id });
                            }
                        });
                    });
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Save as SSH profile")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.save_draft_as_ssh_profile();
                    }
                    if !self.ssh_profiles.is_empty() {
                        ui.label(egui::RichText::new("Use profile:").small().color(self.theme.text_muted));
                        for profile in self.ssh_profiles.clone() {
                            let selected = self.connection_draft.ssh_profile_id == profile.id;
                            if ui.selectable_label(selected, &profile.name).clicked() {
                                self.apply_ssh_profile(&profile.id);
                            }
                        }
                    }
                });
                if !self.connection_draft.ssh_profile_id.is_empty() {
                    ui.label(
                        egui::RichText::new(format!(
                            "Referenced SSH profile id `{}` (secrets stay in vault)",
                            self.connection_draft.ssh_profile_id
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
            }
        });

        // Panel 4: Tags & Metadata
        ui.add_space(SPACE_SM);
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                egui::RichText::new("Tags & Metadata")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            )
            .id_salt("conn_tags_panel")
            .show(ui, |ui| {
                ui.add_space(SPACE_XS);
                Input::new(
                    &mut self.connection_draft.tags,
                    "e.g. prod, analytics, reporting",
                    self.theme,
                )
                .label("Tags (comma separated)")
                .leading_icon(Icon::Tag)
                .show(ui);
            });
        });
    }

    pub(crate) fn draw_sqlite_connection_fields(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_width();
        let gap = SPACE_SM;

        // ── 1. General Profile Section ────────────────────────────────
        ui.label(
            egui::RichText::new("GENERAL")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        // Row 1: Connection Name (52%), Folder/Group (36%), Favorite (12%)
        let name_w = (avail - 2.0 * gap) * 0.52;
        let group_w = (avail - 2.0 * gap) * 0.36;
        let fav_w = (avail - 2.0 * gap) * 0.12;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(name_w);
                ui.set_max_width(name_w);
                Input::new(&mut self.connection_draft.name, "e.g. Local SQLite DB", self.theme)
                    .label("Connection Name")
                    .width(name_w)
                    .leading_icon(Icon::Tag)
                    .clearable(true)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(group_w);
                ui.set_max_width(group_w);
                Input::new(&mut self.connection_draft.group, "e.g. Local Projects", self.theme)
                    .label("Folder / Group")
                    .width(group_w)
                    .leading_icon(Icon::Folder)
                    .clearable(true)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(fav_w);
                ui.set_max_width(fav_w);
                ui.label(
                    egui::RichText::new("Favorite")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
                let (fav_icon, fav_text) = if self.connection_draft.favorite {
                    (Icon::Star, "Saved")
                } else {
                    (Icon::Star, "Off")
                };
                if Button::new(self.theme)
                    .icon(fav_icon)
                    .text(fav_text)
                    .variant(if self.connection_draft.favorite {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    })
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.connection_draft.favorite = !self.connection_draft.favorite;
                }
            });
        });
        ui.add_space(SPACE_SM);

        // Row 2: Environment Selector & Read-only mode
        ui.horizontal(|ui| {
            let env_w = (avail - gap) * 0.55;
            let ro_w = (avail - gap) * 0.45;

            ui.vertical(|ui| {
                ui.set_width(env_w);
                ui.set_max_width(env_w);
                ui.label(
                    egui::RichText::new("Environment")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                let mut env_idx = match self.connection_draft.environment.as_str() {
                    "Staging" => 1,
                    "Production" => 2,
                    "Custom" => 3,
                    _ => 0,
                };
                SegmentedTabs::new(
                    &mut env_idx,
                    &["Development", "Staging", "Production", "Custom"],
                    self.theme,
                )
                .show(ui);
                self.connection_draft.environment = match env_idx {
                    1 => "Staging".to_owned(),
                    2 => "Production".to_owned(),
                    3 => "Custom".to_owned(),
                    _ => "Development".to_owned(),
                };
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(ro_w);
                ui.set_max_width(ro_w);
                ui.label(
                    egui::RichText::new("Safety Policy")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(SPACE_XXS);
                ui.checkbox(&mut self.connection_draft.readonly, "Read-only mode");
            });
        });
        ui.add_space(SPACE_MD);

        // ── 2. Database File Path ─────────────────────────────────────
        ui.label(
            egui::RichText::new("DATABASE FILE")
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        let btn_w = 110.0;
        let input_w = (avail - btn_w - gap).max(120.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(input_w);
                ui.set_max_width(input_w);
                Input::new(&mut self.connection_draft.database, "/path/to/database.db", self.theme)
                    .label("Database File Path")
                    .width(input_w)
                    .leading_icon(Icon::FolderArchive)
                    .clearable(true)
                    .show(ui);
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(btn_w);
                ui.set_max_width(btn_w);
                ui.add_space(20.0); // Align with input below label
                if Button::new(self.theme)
                    .icon(Icon::FolderOpen)
                    .text("Browse File…")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::PickSqliteFile { request_id });
                }
            });
        });
        ui.add_space(SPACE_SM);

        // Embedded SQLite Engine Info Card
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::same(10.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(char::from(Icon::Info).to_string())
                        .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.add_space(SPACE_XS);
                ui.label(
                    egui::RichText::new(
                        "SQLite is embedded in-process. Tables, indexes, triggers, and foreign keys are introspected automatically.",
                    )
                    .size(11.5)
                    .color(self.theme.text_muted),
                );
            });
        });

        // Panel: Tags & Metadata
        ui.add_space(SPACE_SM);
        egui::Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::symmetric(12.0, 8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                egui::RichText::new("Tags & Metadata")
                    .font(DbProTheme::ui_medium_font(11.5))
                    .color(self.theme.text_primary),
            )
            .id_salt("sqlite_tags_panel")
            .show(ui, |ui| {
                ui.add_space(SPACE_XS);
                Input::new(&mut self.connection_draft.tags, "e.g. local, test, sqlite", self.theme)
                    .label("Tags (comma separated)")
                    .leading_icon(Icon::Tag)
                    .show(ui);
            });
        });
    }
}
