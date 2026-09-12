//! Workspace and Application Shell components.
//!
//! Implements AppShell, ActivityBar, PrimarySidebar, WorkspaceTabs, StatusBar,
//! and Breadcrumbs as specified in `open-ai-refer.md` and `open-api-style.md`.

use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, Color32, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

// ── StatusBar Component ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StatusBarItem {
    pub text: String,
    pub icon: Option<Icon>,
    pub tooltip: Option<String>,
    pub is_accent: bool,
}

impl StatusBarItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            tooltip: None,
            is_accent: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn accent(mut self, accent: bool) -> Self {
        self.is_accent = accent;
        self
    }
}

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
            x_cursor += item_w + SPACE_MD;
        }

        // Right items
        let mut r_cursor = rect.right() - SPACE_MD;
        for item in self.right_items.iter().rev() {
            let item_w = self.measure_status_item(ui, item);
            r_cursor -= item_w;
            self.paint_status_item(ui, item, r_cursor, center_y);
            r_cursor -= SPACE_MD;
        }

        resp
    }

    fn measure_status_item(&self, ui: &Ui, item: &StatusBarItem) -> f32 {
        let text_galley = ui
            .painter()
            .layout_no_wrap(item.text.clone(), font_caption(), self.theme.text_secondary);
        let icon_w = if item.icon.is_some() { 16.0 } else { 0.0 };
        icon_w + text_galley.size().x
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
            cur_x += 16.0;
        }

        let galley = ui.painter().layout_no_wrap(item.text.clone(), font_caption(), color);
        let w = galley.size().x;
        ui.painter().galley(
            Pos2::new(cur_x, center_y - galley.size().y * 0.5),
            galley,
            Color32::PLACEHOLDER,
        );
        cur_x += w;

        cur_x - x
    }
}

// ── ActivityBar Component ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityBarItemKind {
    Explorer,
    QueryEditor,
    Agent,
    Diagram,
    Settings,
}

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

        let items = [
            (ActivityBarItemKind::Explorer, Icon::FolderTree, "Database Explorer"),
            (ActivityBarItemKind::QueryEditor, Icon::CodeXml, "SQL Editor"),
            (ActivityBarItemKind::Agent, Icon::Sparkles, "AI Copilot"),
            (ActivityBarItemKind::Diagram, Icon::Network, "Schema ER Diagram"),
        ];

        let mut y = rect.top() + SPACE_MD;
        for (kind, icon, tooltip) in items {
            let is_selected = self.selected == kind;
            let item_rect = Rect::from_min_size(Pos2::new(rect.left() + 6.0, y), Vec2::new(36.0, 36.0));
            let resp = ui.interact(item_rect, ui.id().with(tooltip), Sense::click());

            if resp.hovered() && !is_selected {
                ui.painter()
                    .rect_filled(item_rect, Rounding::same(RADIUS_SM), self.theme.surface_hover);
            }

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, Rounding::same(RADIUS_SM), self.theme.surface_active);
                // Left accent bar
                ui.painter().rect_filled(
                    Rect::from_min_size(Pos2::new(rect.left(), y + 8.0), Vec2::new(2.5, 20.0)),
                    Rounding::same(1.0),
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
                char::from(icon).to_string(),
                font_icon(ICON_TOOLBAR),
                icon_color,
            );

            if resp.clicked() {
                clicked = Some(kind);
            }

            y += 42.0;
        }

        clicked
    }
}

// ── ConnectionIndicator Component ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionHealth {
    Healthy,
    Degraded,
    Disconnected,
}

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
            ConnectionHealth::Healthy => (self.theme.success, "Connected"),
            ConnectionHealth::Degraded => (self.theme.warning, "High Latency"),
            ConnectionHealth::Disconnected => (self.theme.danger, "Disconnected"),
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
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.5, dot_color);
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
                        ui.label(
                            RichText::new(format!("{}ms", ms))
                                .size(FONT_SIZE_CAPTION)
                                .color(if ms > 200 {
                                    self.theme.warning
                                } else {
                                    self.theme.text_secondary
                                }),
                        );
                    }
                });
            })
            .response
    }
}
