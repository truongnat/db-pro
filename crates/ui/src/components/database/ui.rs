// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::config::{
    DATABASE_BADGE_HEIGHT, DATABASE_BADGE_ICON_INSET_X, DATABASE_BADGE_TEXT_INSET_X, DATABASE_BADGE_WIDTH,
    SSL_BADGE_HEIGHT, SSL_BADGE_PADDING_X, SSL_BADGE_TEXT_INSET_Y, STATUS_DOT_BOX_SIZE, STATUS_DOT_RADIUS,
};
use super::handler::{
    connection_action_visibility, connection_status_presentation, remaining_content_width, ConnectionStatus,
    DatabaseDriver,
};
use super::ConnectionCardAction;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::*;
use crate::DbProTheme;
use egui::{Align2, CornerRadius, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

/// A stateless summary of a saved connection that returns the action chosen by the user.
pub struct ConnectionCard<'a> {
    name: &'a str,
    driver: DatabaseDriver,
    host: &'a str,
    database: &'a str,
    status: ConnectionStatus,
    ssl: bool,
    theme: DbProTheme,
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

    /// Paints the card and returns at most one action; no connection state is mutated here.
    pub fn show(self, ui: &mut Ui) -> Option<ConnectionCardAction> {
        let mut triggered = None;
        let action_visibility = connection_action_visibility(self.status);
        let border_color = if action_visibility.show_disconnect {
            self.theme.border_strong
        } else {
            self.theme.border_default
        };

        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(STROKE_THIN, border_color))
            .corner_radius(CornerRadius::same(RADIUS_CARD as u8))
            .inner_margin(egui::Margin::same(CARD_INNER_PAD as i8));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Reserve the status label before allocating the identity column so long names truncate instead of
            // pushing status and actions outside narrow connection cards.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(self.driver.icon()).to_string())
                        .font(font_icon(ICON_TOOLBAR))
                        .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_XS);

                let status = connection_status_presentation(self.status, &self.theme);
                let status_text = ui
                    .painter()
                    .layout_no_wrap(status.label.to_owned(), font_caption(), status.color);
                let status_width = status_text.size().x + STATUS_DOT_BOX_SIZE + SPACE_XXS;
                let identity_width =
                    remaining_content_width(ui.available_width(), status_width, ui.spacing().item_spacing.x);
                ui.allocate_ui_with_layout(
                    Vec2::new(identity_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(identity_width);
                        ui.add(
                            egui::Label::new(
                                RichText::new(self.name)
                                    .size(FONT_SIZE_UI_LABEL)
                                    .strong()
                                    .color(self.theme.text_primary),
                            )
                            .truncate(),
                        )
                        .on_hover_text(self.name);
                        ui.add_space(SPACE_XXS);
                        let database_label = format!("{} · {}", self.driver.name(), self.database);
                        ui.add(
                            egui::Label::new(
                                RichText::new(database_label.clone())
                                    .size(FONT_SIZE_CAPTION)
                                    .color(self.theme.text_secondary),
                            )
                            .truncate(),
                        )
                        .on_hover_text(database_label);
                    },
                );

                ui.allocate_ui_with_layout(
                    Vec2::new(status_width, ui.available_height()),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(STATUS_DOT_BOX_SIZE), Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), STATUS_DOT_RADIUS, status.color);
                        ui.add_space(SPACE_XXS);
                        ui.label(RichText::new(status.label).size(FONT_SIZE_CAPTION).color(status.color));
                    },
                );
            });

            ui.add_space(SPACE_SM);

            // The host keeps its full value in a hover tooltip while the visible label shares width with SSL.
            ui.horizontal(|ui| {
                let ssl_galley = self.ssl.then(|| {
                    ui.painter()
                        .layout_no_wrap("SSL".to_owned(), font_caption(), self.theme.text_secondary)
                });
                let ssl_width = ssl_galley
                    .as_ref()
                    .map_or(0.0, |galley| galley.size().x + SSL_BADGE_PADDING_X);
                // egui inserts item_spacing between the host allocation and the SSL badge on top of
                // the manual SPACE_XS gap, so the reservation must cover both or the card overflows.
                let ssl_gap = if self.ssl {
                    ui.spacing().item_spacing.x + SPACE_XS
                } else {
                    0.0
                };
                let host_width = remaining_content_width(ui.available_width(), ssl_width, ssl_gap);
                let host_response = ui.add_sized(
                    [host_width, SSL_BADGE_HEIGHT],
                    egui::Label::new(
                        RichText::new(self.host)
                            .size(FONT_SIZE_CAPTION)
                            .monospace()
                            .color(self.theme.text_tertiary),
                    )
                    .truncate(),
                );
                host_response.on_hover_text(self.host);

                if let Some(ssl_galley) = ssl_galley {
                    ui.add_space(SPACE_XS);
                    let (ssl_rect, ssl_response) =
                        ui.allocate_exact_size(Vec2::new(ssl_width, SSL_BADGE_HEIGHT), Sense::hover());
                    ui.painter()
                        .rect_filled(ssl_rect, CornerRadius::same(RADIUS_XS as u8), self.theme.surface_hover);
                    ui.painter().galley(
                        Pos2::new(
                            ssl_rect.left() + SSL_BADGE_PADDING_X * 0.5,
                            ssl_rect.top() + SSL_BADGE_TEXT_INSET_Y,
                        ),
                        ssl_galley,
                        self.theme.text_secondary,
                    );
                    ssl_response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, "SSL enabled"));
                    ssl_response.on_hover_text("SSL enabled");
                }
            });

            ui.add_space(SPACE_MD);

            ui.horizontal(|ui| {
                if action_visibility.show_disconnect {
                    if Button::new(self.theme)
                        .text("Disconnect")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(ConnectionCardAction::Disconnect);
                    }
                } else if action_visibility.show_connect {
                    if Button::new(self.theme)
                        .text("Connect")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(ConnectionCardAction::Connect);
                    }
                } else if action_visibility.show_retry {
                    if Button::new(self.theme)
                        .text("Retry")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        triggered = Some(ConnectionCardAction::Connect);
                    }
                } else if action_visibility.show_connecting {
                    Button::new(self.theme)
                        .text("Connecting...")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .enabled(false)
                        .show(ui);
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

/// Compact non-interactive badge identifying a database provider.
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
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(DATABASE_BADGE_WIDTH, DATABASE_BADGE_HEIGHT), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, self.driver.name()));
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS_SM as u8), self.theme.surface_hover);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(RADIUS_SM as u8),
            Stroke::new(STROKE_THIN, self.theme.border_subtle),
            egui::StrokeKind::Inside,
        );

        ui.painter().text(
            Pos2::new(rect.left() + DATABASE_BADGE_ICON_INSET_X, rect.center().y),
            Align2::LEFT_CENTER,
            char::from(self.driver.icon()).to_string(),
            font_icon(ICON_XS),
            self.theme.text_secondary,
        );

        ui.painter().text(
            Pos2::new(rect.left() + DATABASE_BADGE_TEXT_INSET_X, rect.center().y),
            Align2::LEFT_CENTER,
            self.driver.name(),
            font_caption(),
            self.theme.text_primary,
        );

        response
    }
}
