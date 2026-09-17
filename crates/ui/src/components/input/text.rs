use egui::{Button, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;

use super::config::INPUT_ROUNDING;
use super::layout::{paint_field_chrome, resolve_field_width};
use crate::components::interact::text_input_info;
use crate::DbProTheme;

pub struct Input<'a> {
    label: Option<Cow<'a, str>>,
    value: &'a mut String,
    placeholder: Cow<'a, str>,
    helper_text: Option<Cow<'a, str>>,
    error_text: Option<Cow<'a, str>>,
    leading_icon: Option<Icon>,
    clearable: bool,
    width: Option<f32>,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Input<'a> {
    pub fn new(value: &'a mut String, placeholder: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            label: None,
            value,
            placeholder: placeholder.into(),
            helper_text: None,
            error_text: None,
            leading_icon: None,
            clearable: false,
            width: None,
            enabled: true,
            theme,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn helper_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    pub fn error_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.error_text = Some(text.into());
        self
    }

    pub fn leading_icon(mut self, icon: Icon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = resolve_field_width(self.width, ui.available_width());

        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            if let Some(label) = &self.label {
                ui.label(
                    RichText::new(label.as_ref())
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.add_space(3.0);
            }

            let has_error = self.error_text.is_some();
            let fill = if self.enabled {
                self.theme.surface_editor
            } else {
                self.theme.surface_panel
            };

            let frame = Frame {
                fill,
                stroke: if has_error {
                    Stroke::new(1.5, self.theme.danger)
                } else {
                    Stroke::NONE
                },
                inner_margin: Margin::symmetric(8.0, 4.0),
                rounding: Rounding::same(INPUT_ROUNDING),
                ..Default::default()
            };

            let frame_w = (width - 16.0).max(60.0);
            let frame_output = frame.show(ui, |ui| {
                ui.set_width(frame_w);
                ui.set_max_width(frame_w);
                ui.horizontal(|ui| {
                    if let Some(icon) = self.leading_icon {
                        ui.label(
                            RichText::new(char::from(icon).to_string())
                                .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                                .color(if self.enabled {
                                    self.theme.text_muted
                                } else {
                                    self.theme.border_subtle
                                }),
                        );
                        ui.add_space(4.0);
                    }

                    let has_text = !self.value.is_empty();
                    let extra_width = if self.clearable && has_text { 24.0 } else { 0.0 };
                    let edit_w = (ui.available_width() - extra_width).max(40.0);
                    let edit_response = ui.add_enabled(
                        self.enabled,
                        TextEdit::singleline(self.value)
                            .hint_text(RichText::new(self.placeholder.as_ref()).color(self.theme.text_muted))
                            .desired_width(edit_w)
                            .margin(Margin::ZERO)
                            .frame(false)
                            .text_color(if self.enabled {
                                self.theme.text_primary
                            } else {
                                self.theme.text_muted
                            }),
                    );

                    if self.clearable
                        && has_text
                        && self.enabled
                        && ui
                            .add(
                                Button::new(
                                    RichText::new(char::from(Icon::X).to_string())
                                        .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                                        .color(self.theme.text_muted),
                                )
                                .frame(false),
                            )
                            .on_hover_text("Clear")
                            .clicked()
                    {
                        self.value.clear();
                    }

                    edit_response
                })
                .inner
            });

            let edit_response = frame_output.inner;
            let frame_rect = frame_output.response.rect;
            let info_label = self.label.as_deref().unwrap_or(self.placeholder.as_ref());
            edit_response.widget_info(|| text_input_info(self.enabled, info_label));

            if frame_output.response.interact(egui::Sense::click()).clicked() && self.enabled {
                edit_response.request_focus();
            }

            paint_field_chrome(
                ui,
                edit_response.id,
                frame_rect,
                edit_response.has_focus(),
                frame_output.response.hovered() || edit_response.hovered(),
                self.enabled && !has_error,
                self.theme,
            );

            if let Some(err) = &self.error_text {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::AlertCircle).to_string())
                            .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.danger),
                    );
                    ui.add_space(2.0);
                    ui.label(RichText::new(err.as_ref()).size(11.0).color(self.theme.danger));
                });
            } else if let Some(helper) = &self.helper_text {
                ui.add_space(2.0);
                ui.label(RichText::new(helper.as_ref()).size(11.0).color(self.theme.text_muted));
            }

            edit_response
        })
        .inner
    }
}
