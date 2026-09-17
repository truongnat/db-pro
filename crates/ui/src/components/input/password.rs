use egui::{Button, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};
use lucide_icons::Icon;
use std::borrow::Cow;

use super::config::INPUT_ROUNDING;
use super::layout::{paint_field_chrome, resolve_field_width};
use crate::components::interact::text_input_info;
use crate::DbProTheme;

pub struct PasswordInput<'a> {
    label: Option<Cow<'a, str>>,
    value: &'a mut String,
    placeholder: Cow<'a, str>,
    show_password: &'a mut bool,
    helper_text: Option<Cow<'a, str>>,
    error_text: Option<Cow<'a, str>>,
    required: bool,
    width: Option<f32>,
    theme: DbProTheme,
}

impl<'a> PasswordInput<'a> {
    pub fn new(
        value: &'a mut String,
        placeholder: impl Into<Cow<'a, str>>,
        show_password: &'a mut bool,
        theme: DbProTheme,
    ) -> Self {
        Self {
            label: None,
            value,
            placeholder: placeholder.into(),
            show_password,
            helper_text: None,
            error_text: None,
            required: false,
            width: None,
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

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = resolve_field_width(self.width, ui.available_width());

        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            if let Some(label) = &self.label {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(label.as_ref())
                            .size(12.0)
                            .strong()
                            .color(self.theme.text_secondary),
                    );
                    if self.required {
                        ui.label(RichText::new("*").size(12.0).strong().color(self.theme.danger));
                    }
                });
                ui.add_space(3.0);
            }

            let has_error = self.error_text.is_some();
            let border_stroke = if has_error {
                Stroke::new(1.5, self.theme.danger)
            } else {
                Stroke::new(1.0, self.theme.border_default)
            };

            let frame_w = (width - 16.0).max(60.0);
            let frame_output = Frame {
                fill: self.theme.surface_editor,
                stroke: border_stroke,
                inner_margin: Margin::symmetric(8.0, 4.0),
                rounding: Rounding::same(INPUT_ROUNDING),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_width(frame_w);
                ui.set_max_width(frame_w);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Lock).to_string())
                            .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);

                    let edit_w = (ui.available_width() - 26.0).max(40.0);
                    let edit_response = ui.add(
                        TextEdit::singleline(self.value)
                            .password(!*self.show_password)
                            .hint_text(RichText::new(self.placeholder.as_ref()).color(self.theme.text_muted))
                            .desired_width(edit_w)
                            .margin(Margin::ZERO)
                            .frame(false)
                            .text_color(self.theme.text_primary),
                    );

                    let eye_icon = if *self.show_password { Icon::EyeOff } else { Icon::Eye };
                    if ui
                        .add(
                            Button::new(
                                RichText::new(char::from(eye_icon).to_string())
                                    .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            )
                            .frame(false),
                        )
                        .on_hover_text(if *self.show_password { "Hide" } else { "Show" })
                        .clicked()
                    {
                        *self.show_password = !*self.show_password;
                    }

                    edit_response
                })
                .inner
            });

            let edit_response = frame_output.inner;
            let frame_rect = frame_output.response.rect;
            let info_label = self.label.as_deref().unwrap_or("Password");
            edit_response.widget_info(|| text_input_info(true, info_label));

            if frame_output.response.interact(egui::Sense::click()).clicked() {
                edit_response.request_focus();
            }

            paint_field_chrome(
                ui,
                edit_response.id,
                frame_rect,
                edit_response.has_focus(),
                frame_output.response.hovered() || edit_response.hovered(),
                !has_error,
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
