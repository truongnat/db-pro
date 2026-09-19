//! Workspace start page — resume a connection or begin one.
//!
//! Guided by ui-ux-pro-max for a **Developer Tool / IDE**:
//! Minimalism & Swiss Style, Minimal & Direct launcher, density 7.
//! Connection is the unit of work — not a marketing brochure.
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::interact::paint_focus_ring;
use crate::components::kbd_combo;
use egui::{CursorIcon, Sense, Vec2};
use lucide_icons::Icon;

const CONTENT_MAX_WIDTH: f32 = 880.0;
const NARROW_BREAKPOINT: f32 = 640.0;
const CONNECTION_ROW_LIMIT: usize = 8;
/// Prefer ≥40px interactive rows (desktop hit comfort; skill recommends 44 for touch).
const ACTION_ROW_HEIGHT: f32 = 40.0;
const CONNECTION_ROW_HEIGHT: f32 = 44.0;

#[derive(Default)]
struct WelcomeIntent {
    new_connection: bool,
    new_query: bool,
    open_palette: bool,
    open_draft_query: bool,
    connect_id: Option<String>,
}

impl DbProApp {
    pub(super) fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        let mut intent = WelcomeIntent::default();
        let available = ui.available_size();
        let column_w = available.x.min(CONTENT_MAX_WIDTH);
        let inset = ((available.x - column_w) * 0.5).max(SPACE_XL);

        ui.horizontal(|ui| {
            ui.add_space(inset);
            ui.allocate_ui_with_layout(Vec2::new(column_w, available.y), Layout::top_down(Align::Min), |ui| {
                ui.add_space(SPACE_3XL);
                self.draw_welcome_identity(ui);
                ui.add_space(SPACE_2XL);
                self.draw_welcome_hairline(ui);
                ui.add_space(SPACE_XL);

                if column_w >= NARROW_BREAKPOINT {
                    self.draw_welcome_two_column(ui, &mut intent);
                } else {
                    self.draw_welcome_start_actions(ui, &mut intent);
                    ui.add_space(SPACE_2XL);
                    self.draw_welcome_connections(ui, &mut intent);
                }

                if !self.welcome.prompt.trim().is_empty() {
                    ui.add_space(SPACE_XL);
                    self.draw_welcome_draft(ui, &mut intent);
                }
            });
        });

        self.apply_welcome_intent(intent);
    }

    fn apply_welcome_intent(&mut self, intent: WelcomeIntent) {
        if intent.new_connection {
            self.open_new_connection();
        }
        if intent.new_query {
            self.new_query_document();
            self.workspace.active_tab = WorkspaceTab::Query;
        }
        if intent.open_draft_query {
            let draft = self.welcome.prompt.trim().to_owned();
            self.set_active_query_text(draft);
            self.workspace.active_tab = WorkspaceTab::Query;
            self.feedback.runtime_message = "Opened draft in Query".to_owned();
        }
        if intent.open_palette {
            self.open_palette(PaletteMode::Commands);
        }
        if let Some(id) = intent.connect_id {
            if let Some(connection) = self.connection_catalog.connections.iter().find(|c| c.id == id).cloned() {
                self.connect_to_connection(&connection);
            }
        }
    }

    fn draw_welcome_identity(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("DB Pro")
                        .font(font_page_title())
                        .strong()
                        .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_XS);
                let subtitle = if let Some(active) = self.active_connection() {
                    format!("Connected · {}", active.name)
                } else if self.connection_catalog.connections.is_empty() {
                    "A focused database workspace. Connect to begin.".to_owned()
                } else {
                    "Resume a connection, or start something new.".to_owned()
                };
                ui.label(
                    RichText::new(subtitle)
                        .font(font_body())
                        .color(self.theme.text_secondary),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(SPACE_XS, 0.0);
                    ui.label(
                        RichText::new("Palette")
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                    let parts = Self::shortcut_parts(&["Shift", "P"]);
                    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
                    kbd_combo(ui, &refs, self.theme);
                });
            });
        });
    }

    fn draw_welcome_hairline(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
        ui.painter().hline(
            rect.x_range(),
            rect.center().y,
            egui::Stroke::new(1.0, self.theme.border_subtle),
        );
    }

    fn draw_welcome_two_column(&self, ui: &mut egui::Ui, intent: &mut WelcomeIntent) {
        let gap = SPACE_2XL;
        let total = ui.available_width();
        let left_w = ((total - gap) * 0.42).clamp(220.0, 320.0);
        let right_w = (total - gap - left_w).max(280.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| self.draw_welcome_start_actions(ui, intent),
            );
            ui.allocate_ui_with_layout(
                Vec2::new(right_w, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| self.draw_welcome_connections(ui, intent),
            );
        });
    }

    fn draw_welcome_start_actions(&self, ui: &mut egui::Ui, intent: &mut WelcomeIntent) {
        self.welcome_section_label(ui, "Start");
        ui.add_space(SPACE_SM);

        if self.welcome_action_row(ui, Icon::PlugZap, "New connection", &Self::shortcut_parts(&["N"]), true) {
            intent.new_connection = true;
        }
        ui.add_space(SPACE_XXS);
        if self.welcome_action_row(ui, Icon::SquarePen, "New query", &Self::shortcut_parts(&["T"]), false) {
            intent.new_query = true;
        }
        ui.add_space(SPACE_XXS);
        if self.welcome_action_row(
            ui,
            Icon::Search,
            "Command palette",
            &Self::shortcut_parts(&["Shift", "P"]),
            false,
        ) {
            intent.open_palette = true;
        }
    }

    fn welcome_section_label(&self, ui: &mut egui::Ui, label: &str) {
        ui.label(
            RichText::new(label)
                .font(font_caption())
                .strong()
                .color(self.theme.text_muted),
        );
    }

    fn welcome_action_row(
        &self,
        ui: &mut egui::Ui,
        icon: Icon,
        label: &str,
        shortcut: &[String],
        primary: bool,
    ) -> bool {
        let width = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, ACTION_ROW_HEIGHT), Sense::click());
        let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
        let hovered = resp.hovered();
        let focused = resp.has_focus();

        if hovered || focused {
            ui.painter()
                .rect_filled(rect, egui::Rounding::same(RADIUS_SM), self.theme.surface_hover);
        }
        if focused {
            paint_focus_ring(ui, rect, RADIUS_SM, self.theme);
        }

        let icon_color = if primary {
            self.theme.accent
        } else if hovered || focused {
            self.theme.text_primary
        } else {
            self.theme.text_secondary
        };
        let label_color = if hovered || focused || primary {
            self.theme.text_primary
        } else {
            self.theme.text_secondary
        };

        ui.painter().text(
            egui::pos2(rect.left() + SPACE_SM, rect.center().y),
            egui::Align2::LEFT_CENTER,
            char::from(icon).to_string(),
            font_icon(ICON_DEFAULT),
            icon_color,
        );
        ui.painter().text(
            egui::pos2(rect.left() + SPACE_SM + 24.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            if primary { font_ui_label() } else { font_body() },
            label_color,
        );

        // Trailing shortcut chips (discoverability without competing with the label).
        let mut x = rect.right() - SPACE_SM;
        for (i, part) in shortcut.iter().rev().enumerate() {
            let galley =
                ui.painter()
                    .layout_no_wrap(part.clone(), egui::FontId::monospace(10.0), self.theme.text_muted);
            let chip_w = galley.size().x + 10.0;
            x -= chip_w;
            let chip = egui::Rect::from_min_size(
                egui::pos2(x, rect.center().y - (galley.size().y + 4.0) * 0.5),
                Vec2::new(chip_w, galley.size().y + 4.0),
            );
            ui.painter()
                .rect_filled(chip, egui::Rounding::same(RADIUS_XS), self.theme.surface_elevated);
            ui.painter().rect_stroke(
                chip,
                egui::Rounding::same(RADIUS_XS),
                egui::Stroke::new(1.0, self.theme.border_subtle),
            );
            ui.painter().galley(
                egui::pos2(chip.left() + 5.0, chip.center().y - galley.size().y * 0.5),
                galley,
                self.theme.text_muted,
            );
            if i + 1 < shortcut.len() {
                x -= 12.0;
                ui.painter().text(
                    egui::pos2(x + 6.0, rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(10.0),
                    self.theme.text_muted,
                );
                x -= 2.0;
            }
        }

        resp.clicked()
    }

    fn draw_welcome_connections(&self, ui: &mut egui::Ui, intent: &mut WelcomeIntent) {
        self.welcome_section_label(ui, "Connections");
        ui.add_space(SPACE_SM);

        egui::Frame {
            fill: self.theme.surface_elevated,
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            inner_margin: egui::Margin::same(SPACE_XS),
            rounding: egui::Rounding::same(RADIUS_MD),
            ..Default::default()
        }
        .show(ui, |ui| {
            if self.connection_catalog.connections.is_empty() {
                ui.add_space(SPACE_MD);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Database).to_string())
                            .font(font_icon(ICON_LG))
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(SPACE_SM);
                    ui.label(
                        RichText::new("No connections yet")
                            .font(font_ui_label())
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    ui.add_space(SPACE_XXS);
                    ui.label(
                        RichText::new("Add one to load schemas and run queries.")
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(SPACE_MD);
                    if Button::new(self.theme)
                        .icon(Icon::Plus)
                        .text("New connection")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        intent.new_connection = true;
                    }
                });
                ui.add_space(SPACE_MD);
                return;
            }

            let active_id = self.connection_lifecycle.active_connection_id.clone();
            let rows: Vec<_> = self
                .connection_catalog
                .connections
                .iter()
                .take(CONNECTION_ROW_LIMIT)
                .cloned()
                .collect();

            for connection in &rows {
                let is_active = active_id.as_deref() == Some(connection.id.as_str());
                let meta = format!(
                    "{} · {}:{}/{}",
                    connection.driver, connection.host, connection.port, connection.database
                );
                if self.welcome_connection_row(ui, &connection.name, &meta, is_active) && !is_active {
                    intent.connect_id = Some(connection.id.clone());
                }
            }
        });
    }

    fn welcome_connection_row(&self, ui: &mut egui::Ui, name: &str, meta: &str, is_active: bool) -> bool {
        let width = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, CONNECTION_ROW_HEIGHT), Sense::click());
        let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
        let hovered = resp.hovered();
        let focused = resp.has_focus();

        if is_active || hovered || focused {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(RADIUS_SM),
                if is_active {
                    self.theme.accent_soft
                } else {
                    self.theme.surface_hover
                },
            );
        }
        if focused {
            paint_focus_ring(ui, rect, RADIUS_SM, self.theme);
        }

        let icon_color = if is_active {
            self.theme.accent
        } else {
            self.theme.text_secondary
        };
        ui.painter().text(
            egui::pos2(rect.left() + SPACE_SM, rect.center().y),
            egui::Align2::LEFT_CENTER,
            char::from(Icon::Database).to_string(),
            font_icon(ICON_DEFAULT),
            icon_color,
        );

        let text_x = rect.left() + SPACE_SM + 24.0;
        ui.painter().text(
            egui::pos2(text_x, rect.center().y - 8.0),
            egui::Align2::LEFT_CENTER,
            name,
            font_ui_label(),
            self.theme.text_primary,
        );
        ui.painter().text(
            egui::pos2(text_x, rect.center().y + 9.0),
            egui::Align2::LEFT_CENTER,
            meta,
            font_caption(),
            self.theme.text_muted,
        );

        if is_active {
            ui.painter().text(
                egui::pos2(rect.right() - SPACE_SM, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "Active",
                font_caption(),
                self.theme.accent,
            );
        }

        resp.clicked()
    }

    fn draw_welcome_draft(&self, ui: &mut egui::Ui, intent: &mut WelcomeIntent) {
        let preview = self.welcome.prompt.trim();
        let preview = if preview.chars().count() > 72 {
            format!("{}…", preview.chars().take(72).collect::<String>())
        } else {
            preview.to_owned()
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Draft query")
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
            ui.label(
                RichText::new(preview)
                    .font(egui::FontId::monospace(11.0))
                    .color(self.theme.text_secondary),
            );
            if Button::new(self.theme)
                .text("Open")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                intent.open_draft_query = true;
            }
        });
    }
}
