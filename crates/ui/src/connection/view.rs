use super::config::*;
use super::layout::*;
use super::logic::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::tokens::*;
use crate::{DbProApp, DbProTheme, UiCommand, UiDriver, UiSslMode};
use egui::{pos2, vec2, Align2, FontFamily, FontId, Frame, Margin, Rect, RichText, Rounding, Stroke};
use lucide_icons::Icon;

/// In-UI qualification caveat for the SSH tunnel control (#239).
pub const SSH_QUALIFICATION_HINT: &str =
    "Unqualified in v0.1: the tunnel has not been end-to-end tested and may not work reliably.";

/// Per-mode guidance for the SSL selector (#144 locked contract).
pub fn ssl_mode_guidance(mode: UiSslMode) -> std::borrow::Cow<'static, str> {
    match mode {
        UiSslMode::Disable => t!("ssl_guidance.disable"),
        UiSslMode::Require => t!("ssl_guidance.require"),
        UiSslMode::VerifyCa => t!("ssl_guidance.verify_ca"),
        UiSslMode::VerifyFull => t!("ssl_guidance.verify_full"),
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
    let (rect, resp) = ui.allocate_exact_size(vec2(props.width, 52.0), egui::Sense::click());
    let is_hovered = resp.hovered() && !props.is_disabled;
    let painter = ui.painter();

    let bg_fill = if props.is_selected {
        theme.accent_soft
    } else if props.is_disabled {
        theme.surface_panel.linear_multiply(0.6)
    } else if is_hovered {
        theme.surface_hover
    } else {
        theme.surface_panel
    };

    let border_stroke = if props.is_selected {
        Stroke::new(1.5, theme.accent)
    } else if props.is_disabled {
        Stroke::new(1.0, theme.border_subtle.linear_multiply(0.5))
    } else if is_hovered {
        Stroke::new(1.0, theme.border_strong)
    } else {
        Stroke::new(1.0, theme.border_subtle)
    };

    painter.rect(rect, Rounding::same(RADIUS_CARD), bg_fill, border_stroke);

    // Left Icon (18px)
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
        FontId::new(18.0, FontFamily::Name("lucide".into())),
        icon_color,
    );

    // Title and Subtitle
    let text_x = rect.min.x + 38.0;
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
        DbProTheme::ui_medium_font(12.5),
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
            pos2(rect.max.x - 12.0, rect.center().y),
            Align2::RIGHT_CENTER,
            char::from(Icon::Check).to_string(),
            FontId::new(14.0, FontFamily::Name("lucide".into())),
            theme.accent,
        );
    } else {
        let badge_w = (props.badge.len() as f32) * 5.5 + 8.0;
        let badge_rect = Rect::from_min_size(
            pos2(rect.max.x - badge_w - 8.0, rect.center().y - 7.0),
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
    pub(crate) fn draw_connection_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.connection_dialog_open;
        let draft_before = self.connection_draft.clone();
        let title = if self.editing_connection_id.is_some() {
            t!("connection.edit_connection")
        } else {
            t!("connection.new_connection")
        };
        let desc = if self.editing_connection_id.is_some() {
            t!("connection.edit_desc")
        } else {
            t!("connection.new_desc")
        };
        Dialog::new(&mut open, title, self.theme)
            .description(desc)
            .width(820.0)
            .id_salt("connection_form_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    self.draw_connection_form(ui);
                });
                frame.footer(|ui| {
                    self.draw_connection_footer(ui);
                });
            });
        self.connection_dialog_open = open && self.connection_dialog_open;
        if self.connection_draft != draft_before {
            self.connection_test_valid = false;
            self.connection_error.clear();
            self.runtime_message = t!("status.connection_changed").to_string();
        }
        if !self.connection_dialog_open {
            self.pending_connection_request = None;
        }
    }

    pub(crate) fn draw_connection_form(&mut self, ui: &mut egui::Ui) {
        // ── 1. Database Engine Selection Cards (Grid: 4 cols) ─────────
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t!("connection.database_engine"))
                    .font(DbProTheme::ui_medium_font(10.5))
                    .color(self.theme.text_muted),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(t!("connection.engine_hint"))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(SPACE_XS);

        let gap = 8.0;
        let card_w = calculate_engine_card_width(ui.available_width(), gap);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for spec in DRIVER_CARD_SPECS {
                let is_selected = self.connection_draft.driver == spec.driver;
                if draw_driver_card(
                    ui,
                    DriverCardProps {
                        icon: spec.icon,
                        name: spec.name,
                        subtitle: spec.subtitle,
                        badge: spec.badge,
                        is_selected,
                        is_disabled: spec.is_disabled,
                        width: card_w,
                    },
                    &self.theme,
                )
                .clicked()
                {
                    select_driver(&mut self.connection_draft, spec.driver);
                }
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
            Alert::new(t!("alerts.config_error"), &self.connection_error, self.theme)
                .variant(AlertVariant::Destructive)
                .show(ui);
        } else if self.connection_test_valid {
            ui.add_space(SPACE_XS);
            Alert::new(t!("alerts.verified"), t!("alerts.verified_desc"), self.theme)
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
                    RichText::new(format!("{mark} · {} · {}", stage.stage.label(), stage.message))
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
                .text(t!("connection.test_connection"))
                .variant(ButtonVariant::Secondary)
                .icon(Icon::Zap)
                .loading(is_testing)
                .show(ui);

            if test_btn.clicked() && !is_testing {
                self.dispatch_connection_command(false);
            }

            if is_testing {
                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(t!("status.connecting"))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            } else if self.connection_test_valid {
                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(char::from(Icon::Check).to_string())
                        .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.success),
                );
                ui.label(
                    RichText::new(t!("alerts.verified"))
                        .font(font_caption())
                        .color(self.theme.success),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let save_label = if self.editing_connection_id.is_some() {
                    t!("connection.update_connection")
                } else {
                    t!("connection.save_connection")
                };

                let save_btn = Button::new(self.theme)
                    .text(&*save_label)
                    .variant(ButtonVariant::Default)
                    .icon(Icon::Check)
                    .show(ui);

                if save_btn.clicked() {
                    self.dispatch_connection_command(true);
                }

                if Button::new(self.theme)
                    .text(t!("connection.cancel"))
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
        // General Profile
        self.draw_general_profile_fields(ui);

        // Server & Credentials
        self.draw_server_credentials_fields(ui);

        // Advanced Panels
        self.draw_cloud_presets_panel(ui);
        self.draw_ssl_certificates_panel(ui);
        self.draw_ssh_tunnel_panel(ui);
        self.draw_tags_metadata_panel(ui, "conn_tags_panel", t!("connection.tags_placeholder_pg").as_ref());
    }

    pub(crate) fn draw_sqlite_connection_fields(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_width();
        let gap = SPACE_SM;

        // General Profile
        self.draw_general_profile_fields(ui);

        // Database File Path
        ui.label(
            RichText::new(t!("connection.database_file"))
                .font(DbProTheme::ui_medium_font(10.5))
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XXS);

        let SqliteFileRowWidths { input_w, button_w } = calculate_sqlite_file_row_widths(avail, gap);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.vertical(|ui| {
                ui.set_width(input_w);
                ui.set_max_width(input_w);
                Input::new(&mut self.connection_draft.database, "/path/to/database.db", self.theme)
                    .label(t!("connection.database_file_path"))
                    .width(input_w)
                    .leading_icon(Icon::FolderArchive)
                    .clearable(true)
                    .show(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(button_w);
                ui.set_max_width(button_w);
                ui.add_space(20.0); // Align with input below label
                if Button::new(self.theme)
                    .icon(Icon::FolderOpen)
                    .text(t!("connection.browse_file"))
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
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            rounding: Rounding::same(RADIUS_CARD),
            inner_margin: Margin::same(10.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Info).to_string())
                        .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(t!("connection.sqlite_info_hint"))
                        .size(11.5)
                        .color(self.theme.text_muted),
                );
            });
        });

        // Panel: Tags & Metadata
        self.draw_tags_metadata_panel(
            ui,
            "sqlite_tags_panel",
            t!("connection.tags_placeholder_sqlite").as_ref(),
        );
    }
}
