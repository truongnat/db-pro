use super::{config, handler};
use crate::components::button::{Button, ButtonVariant};
use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Default,
    Info,
    Success,
    Warning,
    Destructive,
}

pub struct Alert<'a> {
    pub(crate) title: Cow<'a, str>,
    pub(crate) description: Option<Cow<'a, str>>,
    pub(crate) variant: AlertVariant,
    pub(crate) icon: Option<Icon>,
    pub(crate) dismissable: bool,
    pub(crate) theme: DbProTheme,
}

impl<'a> Alert<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, description: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            title: title.into(),
            description: Some(description.into()),
            variant: AlertVariant::Default,
            icon: None,
            dismissable: false,
            theme,
        }
    }

    pub fn title_only(title: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            title: title.into(),
            description: None,
            variant: AlertVariant::Default,
            icon: None,
            dismissable: false,
            theme,
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn dismissable(mut self, dismissable: bool) -> Self {
        self.dismissable = dismissable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let style = handler::style(self.theme, self.variant);
        let icon = self.icon.unwrap_or(style.default_icon);
        let mut dismiss_response = None;
        Frame {
            fill: style.fill,
            stroke: Stroke::new(1.0, style.border),
            inner_margin: Margin::symmetric(config::ALERT_FRAME_PADDING_X, config::ALERT_FRAME_PADDING_Y),
            rounding: Rounding::same(config::ALERT_RADIUS),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(FontId::new(15.0, FontFamily::Name("lucide".into())))
                        .color(style.icon),
                );
                ui.add_space(config::ALERT_ICON_GAP);
                let dismiss_width = if self.dismissable {
                    config::DISMISS_TARGET_SIZE + config::DISMISS_GAP
                } else {
                    0.0
                };
                let text_avail = (ui.available_width() - dismiss_width).max(0.0);

                ui.allocate_ui_with_layout(
                    Vec2::new(text_avail, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(self.title.as_ref())
                                    .font(DbProTheme::ui_medium_font(13.0))
                                    .color(self.theme.text_primary),
                            )
                            .wrap(),
                        );
                        if let Some(desc) = &self.description {
                            ui.add_space(3.0);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(desc.as_ref()).size(12.5).color(self.theme.text_secondary),
                                )
                                .wrap(),
                            );
                        }
                    },
                );

                if self.dismissable {
                    let response = ui
                        .add_sized(
                            Vec2::splat(config::DISMISS_TARGET_SIZE),
                            egui::Button::new(
                                RichText::new(char::from(Icon::X).to_string())
                                    .font(FontId::new(15.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_secondary),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Dismiss alert")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    response.widget_info(|| crate::components::interact::button_info(true, "Dismiss alert"));
                    dismiss_response = Some(response);
                }
            });
        });

        dismiss_response
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertDialogAction {
    Confirm,
    Cancel,
}

pub struct AlertDialog<'a> {
    pub title: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub confirm_label: Cow<'a, str>,
    pub cancel_label: Cow<'a, str>,
    pub destructive: bool,
    pub theme: DbProTheme,
    id_salt: egui::Id,
}

impl<'a> AlertDialog<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, description: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        let title = title.into();
        let id_salt = egui::Id::new(("alert_dialog", title.as_ref()));
        Self {
            title,
            description: description.into(),
            confirm_label: Cow::Borrowed("Continue"),
            cancel_label: Cow::Borrowed("Cancel"),
            destructive: false,
            theme,
            id_salt,
        }
    }

    pub fn confirm_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    /// Supplies a stable identity when more than one alert dialog can be rendered.
    pub fn id_salt(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_salt = egui::Id::new(("alert_dialog", salt));
        self
    }

    pub fn show(self, ctx: &egui::Context, open: &mut bool) -> Option<AlertDialogAction> {
        if !*open {
            return None;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            *open = false;
            return Some(handler::action_for_close(false));
        }

        let mut action = None;
        let screen_rect = ctx.screen_rect();
        let modal_width = handler::dialog_outer_width(screen_rect.width());
        let content_width = handler::dialog_content_width(screen_rect.width());
        let focus_cancel = ctx.memory(|memory| memory.focused().is_none());

        // Backdrop
        let backdrop_id = self.id_salt.with("backdrop");
        egui::Area::new(backdrop_id)
            .order(egui::Order::Foreground)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (_, response) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter().rect_filled(
                    screen_rect,
                    Rounding::ZERO,
                    Color32::from_black_alpha(config::BACKDROP_OPACITY),
                );
                if handler::should_close_from_backdrop(self.destructive, response.clicked()) {
                    *open = false;
                    action = Some(handler::action_for_close(false));
                }
            });

        // Dialog box centered
        let dialog_id = self.id_salt.with("modal");

        egui::Area::new(dialog_id)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                Frame {
                    fill: self.theme.surface_elevated,
                    stroke: Stroke::new(1.0, self.theme.border_default),
                    inner_margin: Margin::same(config::DIALOG_PADDING),
                    rounding: Rounding::same(config::DIALOG_RADIUS),
                    shadow: egui::epaint::Shadow {
                        offset: egui::vec2(0.0, config::DIALOG_SHADOW_OFFSET_Y),
                        blur: config::DIALOG_SHADOW_BLUR,
                        spread: 0.0,
                        color: Color32::from_black_alpha(config::DIALOG_SHADOW_OPACITY),
                    },
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(content_width.max(0.0).min(modal_width));
                    ui.vertical(|ui| {
                        // Title
                        ui.label(
                            RichText::new(self.title.as_ref())
                                .font(DbProTheme::ui_medium_font(15.0))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(6.0);

                        // Description
                        ui.add(
                            egui::Label::new(
                                RichText::new(self.description.as_ref())
                                    .size(13.0)
                                    .color(self.theme.text_secondary),
                            )
                            .wrap(),
                        );
                        ui.add_space(20.0);

                        // Buttons
                        ui.horizontal_wrapped(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let confirm_variant = if self.destructive {
                                    ButtonVariant::Destructive
                                } else {
                                    ButtonVariant::Default
                                };

                                let confirm_resp = Button::new(self.theme)
                                    .text(self.confirm_label.as_ref())
                                    .variant(confirm_variant)
                                    .show(ui);

                                if confirm_resp.clicked() {
                                    *open = false;
                                    action = Some(handler::action_for_close(true));
                                }

                                ui.add_space(config::DIALOG_ACTION_GAP);

                                let cancel_resp = Button::new(self.theme)
                                    .text(self.cancel_label.as_ref())
                                    .variant(ButtonVariant::Secondary)
                                    .show(ui);

                                if focus_cancel {
                                    cancel_resp.request_focus();
                                }
                                if cancel_resp.clicked() {
                                    *open = false;
                                    action = Some(handler::action_for_close(false));
                                }
                            });
                        });
                    });
                });
            });

        action
    }
}
