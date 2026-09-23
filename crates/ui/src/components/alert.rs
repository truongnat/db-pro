use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::DbProTheme;
use egui::{
    Color32, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2,
};
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
        let (fill, border_color, icon_color, default_icon) = match self.variant {
            AlertVariant::Default => (
                self.theme.surface_panel,
                self.theme.border_default,
                self.theme.text_primary,
                Icon::Terminal,
            ),
            AlertVariant::Info => (
                self.theme.info.linear_multiply(0.12),
                self.theme.info.linear_multiply(0.40),
                self.theme.info,
                Icon::Info,
            ),
            AlertVariant::Success => (
                self.theme.success.linear_multiply(0.12),
                self.theme.success.linear_multiply(0.40),
                self.theme.success,
                Icon::CheckCircle2,
            ),
            AlertVariant::Warning => (
                self.theme.warning.linear_multiply(0.12),
                self.theme.warning.linear_multiply(0.40),
                self.theme.warning,
                Icon::AlertTriangle,
            ),
            AlertVariant::Destructive => (
                self.theme.danger.linear_multiply(0.12),
                self.theme.danger.linear_multiply(0.40),
                self.theme.danger,
                Icon::AlertCircle,
            ),
        };

        let icon = self.icon.unwrap_or(default_icon);
        let mut dismiss_response = None;

        Frame {
            fill,
            stroke: Stroke::new(1.0, border_color),
            inner_margin: Margin::symmetric(14.0, 12.0),
            rounding: Rounding::same(8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(FontId::new(15.0, FontFamily::Name("lucide".into())))
                        .color(icon_color),
                );
                ui.add_space(8.0);
                let text_avail = if self.dismissable {
                    (ui.available_width() - 36.0).max(120.0)
                } else {
                    ui.available_width()
                };

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
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let resp = Button::new(self.theme)
                            .icon(Icon::X)
                            .size(ButtonSize::Icon)
                            .variant(ButtonVariant::Secondary)
                            .access_label("Dismiss")
                            .show(ui);
                        dismiss_response = Some(resp);
                    });
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
}

impl<'a> AlertDialog<'a> {
    pub fn new(
        title: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
        theme: DbProTheme,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            confirm_label: Cow::Borrowed("Continue"),
            cancel_label: Cow::Borrowed("Cancel"),
            destructive: true,
            theme,
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

    pub fn show(self, ctx: &egui::Context, open: &mut bool) -> Option<AlertDialogAction> {
        if !*open {
            return None;
        }

        let mut action = None;
        let screen_rect = ctx.screen_rect();

        // Backdrop
        let backdrop_id = egui::Id::new("alert_dialog_backdrop");
        egui::Area::new(backdrop_id)
            .order(egui::Order::Foreground)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (_, response) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter().rect_filled(
                    screen_rect,
                    Rounding::ZERO,
                    Color32::from_black_alpha(140),
                );
                if response.clicked() {
                    *open = false;
                    action = Some(AlertDialogAction::Cancel);
                }
            });

        // Dialog box centered
        let dialog_id = egui::Id::new("alert_dialog_modal");
        let max_w = (440.0_f32).min(screen_rect.width() - 32.0);

        egui::Area::new(dialog_id)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                Frame {
                    fill: self.theme.surface_elevated,
                    stroke: Stroke::new(1.0, self.theme.border_default),
                    inner_margin: Margin::same(20.0),
                    rounding: Rounding::same(10.0),
                    shadow: egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 8.0),
                        blur: 24.0,
                        spread: 0.0,
                        color: Color32::from_black_alpha(80),
                    },
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(max_w);
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
                        ui.horizontal(|ui| {
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
                                    action = Some(AlertDialogAction::Confirm);
                                }

                                ui.add_space(8.0);

                                let cancel_resp = Button::new(self.theme)
                                    .text(self.cancel_label.as_ref())
                                    .variant(ButtonVariant::Secondary)
                                    .show(ui);

                                if cancel_resp.clicked() {
                                    *open = false;
                                    action = Some(AlertDialogAction::Cancel);
                                }
                            });
                        });
                    });
                });
            });

        action
    }
}
