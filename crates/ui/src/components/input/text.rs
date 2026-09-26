use egui::{FontFamily, FontId, Frame, Id, Margin, Response, RichText, Rounding, Sense, Stroke, TextEdit, Ui, Vec2};
use lucide_icons::Icon;
use std::borrow::Cow;

use super::config::{FIELD_INNER_MARGIN_X, FIELD_INNER_MARGIN_Y, INPUT_ROUNDING};
use super::layout::{paint_field_chrome, resolve_field_width};
use crate::components::interact::{button_info, text_input_info};
use crate::tokens::{LABEL_HELPER_GAP, SPACE_XS};
use crate::DbProTheme;

pub struct Input<'a> {
    label: Option<Cow<'a, str>>,
    access_label: Option<Cow<'a, str>>,
    value: &'a mut String,
    placeholder: Cow<'a, str>,
    helper_text: Option<Cow<'a, str>>,
    error_text: Option<Cow<'a, str>>,
    leading_icon: Option<Icon>,
    clearable: bool,
    width: Option<f32>,
    id_salt: Option<Id>,
    auto_focus: bool,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Input<'a> {
    pub fn new(value: &'a mut String, placeholder: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            label: None,
            access_label: None,
            value,
            placeholder: placeholder.into(),
            helper_text: None,
            error_text: None,
            leading_icon: None,
            clearable: false,
            width: None,
            id_salt: None,
            auto_focus: false,
            enabled: true,
            theme,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn access_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.access_label = Some(label.into());
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

    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(id_salt));
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
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
                ui.add_space(LABEL_HELPER_GAP);
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
                inner_margin: Margin::symmetric(FIELD_INNER_MARGIN_X, FIELD_INNER_MARGIN_Y),
                rounding: Rounding::same(INPUT_ROUNDING),
                ..Default::default()
            };

            let frame_w = (width - FIELD_INNER_MARGIN_X * 2.0).max(60.0);
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
                        ui.add_space(SPACE_XS);
                    }

                    let has_text = !self.value.is_empty();
                    let extra_width = if self.clearable && has_text { 24.0 } else { 0.0 };
                    let edit_w = (ui.available_width() - extra_width).max(40.0);
                    let mut text_edit = TextEdit::singleline(self.value);
                    if let Some(id_salt) = self.id_salt {
                        text_edit = text_edit.id_salt(id_salt);
                    }
                    let edit_response = ui.add_enabled(
                        self.enabled,
                        text_edit
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

                    if self.clearable && has_text && self.enabled {
                        let (clear_rect, clear_response) = ui.allocate_at_least(
                            Vec2::new(24.0, ui.spacing().interact_size.y),
                            Sense {
                                click: true,
                                drag: false,
                                focusable: false,
                            },
                        );
                        let clear_response = clear_response.on_hover_text("Clear");
                        clear_response.widget_info(|| button_info(true, "Clear"));
                        ui.painter().text(
                            clear_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            char::from(Icon::X).to_string(),
                            FontId::new(12.0, FontFamily::Name("lucide".into())),
                            self.theme.text_muted,
                        );
                        if clear_response.clicked() {
                            self.value.clear();
                        }
                    }

                    edit_response
                })
                .inner
            });

            let edit_response = frame_output.inner;
            if self.auto_focus {
                edit_response.request_focus();
            }
            let frame_rect = frame_output.response.rect;
            let info_label = self
                .access_label
                .as_deref()
                .or(self.label.as_deref())
                .unwrap_or(self.placeholder.as_ref());
            edit_response.widget_info(|| text_input_info(self.enabled, info_label));

            // NOTE: do NOT register `frame.interact(Sense::click())` here. That call lands on
            // top of the `TextEdit` added inside the frame, so egui reports the click as
            // consumed by the frame and the text field never receives it — double-click never
            // selects text and the caret never lands where you click. The `TextEdit` already
            // focuses itself on its own click, so removing this block restores that.

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
