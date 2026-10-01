use egui::{Align, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};

use crate::components::input::config::{INPUT_AUX_FONT_SIZE, INPUT_LABEL_FONT_SIZE, TEXTAREA_INNER_MARGIN_Y};
use crate::components::input::layout::{paint_field_chrome, FieldChromeState};
use crate::components::interact::text_input_info;
use crate::tokens::{LABEL_HELPER_GAP, RADIUS_XS, SPACE_SM};
use crate::DbProTheme;

pub struct Textarea<'a> {
    label: Option<&'a str>,
    value: &'a mut String,
    placeholder: &'a str,
    min_rows: usize,
    max_chars: Option<usize>,
    theme: DbProTheme,
}

impl<'a> Textarea<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            label: None,
            value,
            placeholder,
            min_rows: 3,
            max_chars: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows;
        self
    }

    pub fn max_chars(mut self, max: usize) -> Self {
        self.max_chars = Some(max);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if let Some(label) = self.label {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(label)
                            .size(INPUT_LABEL_FONT_SIZE)
                            .strong()
                            .color(self.theme.text_secondary),
                    );
                    if let Some(max) = self.max_chars {
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(super::super::handler::character_count(self.value, max))
                                    .size(INPUT_AUX_FONT_SIZE)
                                    .color(self.theme.text_muted),
                            );
                        });
                    }
                });
                ui.add_space(LABEL_HELPER_GAP);
            }

            let frame_output = Frame {
                fill: self.theme.surface_editor,
                stroke: Stroke::NONE,
                inner_margin: Margin::symmetric(SPACE_SM, TEXTAREA_INNER_MARGIN_Y),
                rounding: Rounding::same(RADIUS_XS),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(self.value)
                        .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                        .desired_rows(self.min_rows)
                        .desired_width(ui.available_width())
                        .frame(false)
                        .margin(Margin::ZERO)
                        .text_color(self.theme.text_primary),
                )
            });

            let edit_response = frame_output.inner;
            let frame_rect = frame_output.response.rect;
            let info_label = self.label.unwrap_or(self.placeholder);
            edit_response.widget_info(|| text_input_info(true, info_label));

            paint_field_chrome(
                ui,
                edit_response.id,
                frame_rect,
                FieldChromeState {
                    focused: edit_response.has_focus(),
                    hovered: frame_output.response.hovered() || edit_response.hovered(),
                    enabled: true,
                    has_error: false,
                },
                self.theme,
            );

            edit_response
        })
        .inner
    }
}
