use egui::{Button, Frame, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};
use lucide_icons::Icon;

use super::config::INPUT_ROUNDING;
use super::layout::{paint_field_chrome, resolve_field_width};
use crate::DbProTheme;

pub struct SearchInput<'a> {
    value: &'a mut String,
    placeholder: &'a str,
    shortcut: Option<&'a str>,
    width: Option<f32>,
    theme: DbProTheme,
}

impl<'a> SearchInput<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            value,
            placeholder,
            shortcut: None,
            width: None,
            theme,
        }
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = resolve_field_width(self.width, ui.available_width());

        let frame_output = Frame {
            fill: self.theme.surface_editor,
            // Border is owned by `paint_field_chrome` (rest / hover / focus).
            stroke: Stroke::NONE,
            inner_margin: Margin::symmetric(8.0, 5.0),
            rounding: Rounding::same(INPUT_ROUNDING),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_width(width.max(100.0) - 16.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Search).to_string())
                        .font(egui::FontId::new(14.0, egui::FontFamily::Name("lucide".into())))
                        .color(self.theme.text_muted),
                );
                ui.add_space(4.0);

                let extra_width =
                    if self.shortcut.is_some() { 36.0 } else { 0.0 } + if !self.value.is_empty() { 20.0 } else { 0.0 };

                let edit = ui.add(
                    TextEdit::singleline(self.value)
                        .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                        .desired_width(ui.available_width() - extra_width)
                        .margin(Margin::ZERO)
                        .frame(false)
                        .text_color(self.theme.text_primary),
                );

                if edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.value.clear();
                }

                if !self.value.is_empty()
                    && ui
                        .add(
                            Button::new(
                                RichText::new(char::from(Icon::X).to_string())
                                    .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            )
                            .frame(false),
                        )
                        .clicked()
                {
                    self.value.clear();
                }

                if let Some(sc) = self.shortcut {
                    Frame {
                        fill: self.theme.surface_panel,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        inner_margin: Margin::symmetric(4.0, 1.0),
                        rounding: Rounding::same(4.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.label(RichText::new(sc).size(10.0).color(self.theme.text_muted));
                    });
                }

                edit
            })
            .inner
        });

        let edit_response = frame_output.inner;
        let frame_rect = frame_output.response.rect;

        paint_field_chrome(
            ui,
            edit_response.id,
            frame_rect,
            edit_response.has_focus(),
            frame_output.response.hovered() || edit_response.hovered(),
            true,
            self.theme,
        );

        edit_response
    }
}
