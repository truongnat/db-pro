use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, Color32, Pos2, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};

use super::config::{
    activity_accent_rect, activity_item_rect, activity_item_start_y, is_latency_warning, next_activity_item_y,
    next_status_item_cursor, right_status_item_start, status_item_width, status_text_top, ACTIVITY_ACCENT_RADIUS,
    ACTIVITY_BAR_ITEMS, CONNECTION_DOT_BOX_SIZE, CONNECTION_DOT_RADIUS, STATUS_BAR_ICON_SLOT_WIDTH,
};
use super::handler::{connection_health_label, ActivityBarItemKind, ConnectionHealth, StatusBarItem};

// ── StatusBar Component ──────────────────────────────────────────────────────

pub struct StatusBar<'a> {
    left_items: &'a [StatusBarItem],
    right_items: &'a [StatusBarItem],
    theme: DbProTheme,
}

impl<'a> StatusBar<'a> {
    pub fn new(left_items: &'a [StatusBarItem], right_items: &'a [StatusBarItem], theme: DbProTheme) -> Self {
        Self {
            left_items,
            right_items,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), STATUS_BAR_HEIGHT), Sense::hover());

        // Background & border top
        ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_panel);
        ui.painter().hline(
            rect.x_range(),
            rect.top(),
            Stroke::new(STROKE_THIN, self.theme.border_subtle),
        );

        // Left items
        let mut x_cursor = rect.left() + SPACE_MD;
        let center_y = rect.center().y;

        for item in self.left_items {
            let item_w = self.paint_status_item(ui, item, x_cursor, center_y);
            x_cursor = next_status_item_cursor(x_cursor, item_w);
        }

        // Right items
        let mut r_cursor = rect.right() - SPACE_MD;
        for item in self.right_items.iter().rev() {
            let item_w = self.measure_status_item(ui, item);
            r_cursor = right_status_item_start(r_cursor, item_w);
            self.paint_status_item(ui, item, r_cursor, center_y);
            r_cursor -= SPACE_MD;
        }

        resp
    }

    fn measure_status_item(&self, ui: &Ui, item: &StatusBarItem) -> f32 {
        let text_galley = ui
            .painter()
            .layout_no_wrap(item.text.clone(), font_caption(), self.theme.text_secondary);
        status_item_width(text_galley.size().x, item.icon.is_some())
    }

    fn paint_status_item(&self, ui: &mut Ui, item: &StatusBarItem, x: f32, center_y: f32) -> f32 {
        let mut cur_x = x;
        let color = if item.is_accent {
            self.theme.accent
        } else {
            self.theme.text_secondary
        };

        if let Some(icon) = item.icon {
            ui.painter().text(
                Pos2::new(cur_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                font_icon(ICON_XS),
                color,
            );
            cur_x += STATUS_BAR_ICON_SLOT_WIDTH;
        }

        let galley = ui.painter().layout_no_wrap(item.text.clone(), font_caption(), color);
        let w = galley.size().x;
        ui.painter().galley(
            Pos2::new(cur_x, status_text_top(center_y, galley.size().y)),
            galley,
            Color32::PLACEHOLDER,
        );
        cur_x += w;

        cur_x - x
    }
}

// ── ActivityBar Component ───────────────────────────────────────────────────

pub struct ActivityBar {
    selected: ActivityBarItemKind,
    theme: DbProTheme,
}

impl ActivityBar {
    pub fn new(selected: ActivityBarItemKind, theme: DbProTheme) -> Self {
        Self { selected, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ActivityBarItemKind> {
        let mut clicked = None;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ACTIVITY_BAR_WIDTH, ui.available_height()), Sense::hover());

        // Sidebar background
        ui.painter()
            .rect_filled(rect, Rounding::ZERO, self.theme.surface_editor);
        ui.painter().vline(
            rect.right(),
            rect.y_range(),
            Stroke::new(STROKE_THIN, self.theme.border_subtle),
        );

        let mut y = activity_item_start_y(rect.top());
        for item in ACTIVITY_BAR_ITEMS {
            let is_selected = self.selected == item.kind;
            let item_rect = activity_item_rect(rect.left(), y);
            let resp = ui.interact(item_rect, ui.id().with(item.tooltip), Sense::click());

            if resp.hovered() && !is_selected {
                ui.painter()
                    .rect_filled(item_rect, Rounding::same(RADIUS_SM), self.theme.surface_hover);
            }

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, Rounding::same(RADIUS_SM), self.theme.surface_active);
                // Left accent bar
                ui.painter().rect_filled(
                    activity_accent_rect(rect.left(), y),
                    Rounding::same(ACTIVITY_ACCENT_RADIUS),
                    self.theme.accent,
                );
            }

            let icon_color = if is_selected {
                self.theme.accent
            } else if resp.hovered() {
                self.theme.text_primary
            } else {
                self.theme.text_secondary
            };

            ui.painter().text(
                item_rect.center(),
                Align2::CENTER_CENTER,
                char::from(item.icon).to_string(),
                font_icon(ICON_TOOLBAR),
                icon_color,
            );

            if resp.clicked() {
                clicked = Some(item.kind);
            }

            y = next_activity_item_y(y);
        }

        clicked
    }
}

// ── ConnectionIndicator Component ───────────────────────────────────────────

pub struct ConnectionIndicator<'a> {
    name: &'a str,
    driver: &'a str,
    health: ConnectionHealth,
    latency_ms: Option<u32>,
    theme: DbProTheme,
}

impl<'a> ConnectionIndicator<'a> {
    pub fn new(name: &'a str, driver: &'a str, health: ConnectionHealth, theme: DbProTheme) -> Self {
        Self {
            name,
            driver,
            health,
            latency_ms: None,
            theme,
        }
    }

    pub fn latency(mut self, latency_ms: u32) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (dot_color, _health_label) = match self.health {
            ConnectionHealth::Healthy => (self.theme.success, connection_health_label(self.health)),
            ConnectionHealth::Degraded => (self.theme.warning, connection_health_label(self.health)),
            ConnectionHealth::Disconnected => (self.theme.danger, connection_health_label(self.health)),
        };

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(RADIUS_MD))
            .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM));

        frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Dot
                    let (dot_rect, _) = ui.allocate_exact_size(
                        Vec2::new(CONNECTION_DOT_BOX_SIZE, CONNECTION_DOT_BOX_SIZE),
                        Sense::hover(),
                    );
                    ui.painter()
                        .circle_filled(dot_rect.center(), CONNECTION_DOT_RADIUS, dot_color);
                    ui.add_space(SPACE_XS);

                    // Name
                    ui.label(
                        RichText::new(self.name)
                            .size(FONT_SIZE_UI_LABEL)
                            .strong()
                            .color(self.theme.text_primary),
                    );

                    // Driver tag
                    ui.add_space(SPACE_XS);
                    ui.label(
                        RichText::new(format!("[{}]", self.driver))
                            .size(FONT_SIZE_CAPTION)
                            .monospace()
                            .color(self.theme.text_tertiary),
                    );

                    // Latency
                    if let Some(ms) = self.latency_ms {
                        ui.add_space(SPACE_SM);
                        ui.label(RichText::new(format!("{}ms", ms)).size(FONT_SIZE_CAPTION).color(
                            if is_latency_warning(ms) {
                                self.theme.warning
                            } else {
                                self.theme.text_secondary
                            },
                        ));
                    }
                });
            })
            .response
    }
}
