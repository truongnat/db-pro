//! Database connection and schema explorer components.
//!
//! Implements ConnectionCard, ConnectionBadge, DatabaseDriverIcon,
//! DatabaseTypeBadge, and SchemaTree nodes per `open-ai-refer.md`.

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, Color32, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

// ── DatabaseDriverIcon Component ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseDriver {
    PostgreSql,
    Sqlite,
    MySql,
    SqlServer,
}

impl DatabaseDriver {
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseDriver::PostgreSql => "PostgreSQL",
            DatabaseDriver::Sqlite => "SQLite",
            DatabaseDriver::MySql => "MySQL",
            DatabaseDriver::SqlServer => "SQL Server",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            DatabaseDriver::PostgreSql => Icon::Database,
            DatabaseDriver::Sqlite => Icon::FileCode,
            DatabaseDriver::MySql => Icon::Server,
            DatabaseDriver::SqlServer => Icon::Layers,
        }
    }
}

// ── ConnectionCard Component ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

pub struct ConnectionCard<'a> {
    name: &'a str,
    driver: DatabaseDriver,
    host: &'a str,
    database: &'a str,
    status: ConnectionStatus,
    ssl: bool,
    theme: DbProTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionCardAction {
    Connect,
    Disconnect,
    Edit,
    Delete,
}

impl<'a> ConnectionCard<'a> {
    pub fn new(
        name: &'a str,
        driver: DatabaseDriver,
        host: &'a str,
        database: &'a str,
        status: ConnectionStatus,
        theme: DbProTheme,
    ) -> Self {
        Self {
            name,
            driver,
            host,
            database,
            status,
            ssl: false,
            theme,
        }
    }

    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = ssl;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<ConnectionCardAction> {
        let mut triggered = None;
        let is_connected = self.status == ConnectionStatus::Connected;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(
                STROKE_THIN,
                if is_connected {
                    self.theme.border_strong
                } else {
                    self.theme.border_default
                },
            ))
            .rounding(Rounding::same(RADIUS_CARD))
            .inner_margin(egui::Margin::same(CARD_INNER_PAD));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header: Driver icon, connection name, status badge
            ui.horizontal(|ui| {
                // Driver icon
                ui.label(
                    RichText::new(char::from(self.driver.icon()).to_string())
                        .font(font_icon(ICON_TOOLBAR))
                        .color(self.theme.text_primary),
                );

                ui.add_space(SPACE_XS);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(self.name)
                            .size(FONT_SIZE_UI_LABEL)
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    ui.add_space(SPACE_XXS);
                    ui.label(
                        RichText::new(format!("{} · {}", self.driver.name(), self.database))
                            .size(FONT_SIZE_CAPTION)
                            .color(self.theme.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (dot_color, label) = match self.status {
                        ConnectionStatus::Connected => (self.theme.success, "Active"),
                        ConnectionStatus::Connecting => (self.theme.info, "Connecting..."),
                        ConnectionStatus::Disconnected => (self.theme.text_tertiary, "Offline"),
                        ConnectionStatus::Error => (self.theme.danger, "Error"),
                    };

                    let (rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(rect.center(), 3.5, dot_color);
                    ui.add_space(SPACE_XXS);
                    ui.label(RichText::new(label).size(FONT_SIZE_CAPTION).color(dot_color));
                });
            });

            ui.add_space(SPACE_SM);

            // Host info & badges
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.host)
                        .size(FONT_SIZE_CAPTION)
                        .monospace()
                        .color(self.theme.text_tertiary),
                );

                if self.ssl {
                    ui.add_space(SPACE_XS);
                    let ssl_galley =
                        ui.painter()
                            .layout_no_wrap("SSL".to_owned(), font_caption(), self.theme.text_secondary);
                    let ssl_rect = Rect::from_min_size(
                        Pos2::new(ui.cursor().min.x, ui.cursor().min.y),
                        Vec2::new(ssl_galley.size().x + 8.0, 16.0),
                    );
                    ui.painter()
                        .rect_filled(ssl_rect, Rounding::same(RADIUS_XS), self.theme.surface_hover);
                    ui.painter().galley(
                        Pos2::new(ssl_rect.left() + 4.0, ssl_rect.top() + 1.0),
                        ssl_galley,
                        Color32::PLACEHOLDER,
                    );
                    ui.add_space(ssl_rect.width());
                }
            });

            ui.add_space(SPACE_MD);

            // Action buttons
            ui.horizontal(|ui| {
                if is_connected {
                    if Button::new(self.theme)
                        .text("Disconnect")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(ConnectionCardAction::Disconnect);
                    }
                } else if Button::new(self.theme)
                    .text("Connect")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    triggered = Some(ConnectionCardAction::Connect);
                }

                ui.add_space(SPACE_XS);

                if Button::new(self.theme)
                    .text("Edit")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    triggered = Some(ConnectionCardAction::Edit);
                }
            });
        });

        triggered
    }
}

// ── DatabaseTypeBadge Component ─────────────────────────────────────────────

pub struct DatabaseTypeBadge<'a> {
    driver: DatabaseDriver,
    theme: DbProTheme,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> DatabaseTypeBadge<'a> {
    pub fn new(driver: DatabaseDriver, theme: DbProTheme) -> Self {
        Self {
            driver,
            theme,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(90.0, 22.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, Rounding::same(RADIUS_SM), self.theme.surface_hover);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(RADIUS_SM),
            Stroke::new(STROKE_THIN, self.theme.border_subtle),
        );

        ui.painter().text(
            Pos2::new(rect.left() + 6.0, rect.center().y),
            Align2::LEFT_CENTER,
            char::from(self.driver.icon()).to_string(),
            font_icon(ICON_XS),
            self.theme.text_secondary,
        );

        ui.painter().text(
            Pos2::new(rect.left() + 24.0, rect.center().y),
            Align2::LEFT_CENTER,
            self.driver.name(),
            font_caption(),
            self.theme.text_primary,
        );

        resp
    }
}
