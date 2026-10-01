use egui::{Align, Button, Frame, Layout, Margin, Response, RichText, Rounding, Stroke, TextEdit, Ui};
use lucide_icons::Icon;

use crate::components::input::config::{INPUT_ICON_SIZE, SEARCH_CLEAR_RESERVATION, SEARCH_SHORTCUT_RESERVATION};
use crate::components::input::layout::{paint_field_chrome, resolve_field_width, FieldChromeState};
use crate::tokens::{RADIUS_XS, SPACE_SM, SPACE_XS};
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
        // Cap the frame's content to the field's own width. `set_min_width` alone is
        // only a floor, so the frame grew to whatever the parent offered and the
        // requested `width()` was silently ignored — which is how the sidebar toolbar
        // came to overflow its column and inflate every width measured after it.
        let inner_w = (width - SPACE_SM * 2.0).max(0.0);

        let frame_output = Frame {
            fill: self.theme.surface_editor,
            // Border is owned by `paint_field_chrome` (rest / hover / focus).
            stroke: Stroke::NONE,
            inner_margin: Margin::symmetric(SPACE_SM, SPACE_XS),
            rounding: Rounding::same(RADIUS_XS),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_width(inner_w);
            ui.set_max_width(inner_w);
            // Pin the field's own reading order. `ui.horizontal` *inherits* the parent's
            // layout direction, so inside a right-to-left row (the explorer toolbar) the
            // icon, the text and the clear button would all mirror.
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let icon = ui.label(
                    RichText::new(char::from(Icon::Search).to_string())
                        .font(egui::FontId::new(
                            INPUT_ICON_SIZE,
                            egui::FontFamily::Name("lucide".into()),
                        ))
                        .color(self.theme.text_muted),
                );
                ui.add_space(SPACE_XS);

                let extra_width = if self.shortcut.is_some() {
                    SEARCH_SHORTCUT_RESERVATION
                } else {
                    0.0
                } + if !self.value.is_empty() {
                    SEARCH_CLEAR_RESERVATION
                } else {
                    0.0
                };

                // Reserve the icon, its gap and the inter-item spacing so the text
                // field cannot push the frame past `width`.
                let reserved = icon.rect.width() + SPACE_XS + ui.spacing().item_spacing.x * 2.0 + extra_width;

                let edit = ui.add(
                    TextEdit::singleline(self.value)
                        .hint_text(RichText::new(self.placeholder).color(self.theme.text_muted))
                        .desired_width((inner_w - reserved).max(0.0))
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
            FieldChromeState {
                focused: edit_response.has_focus(),
                hovered: frame_output.response.hovered() || edit_response.hovered(),
                enabled: true,
                has_error: false,
            },
            self.theme,
        );

        edit_response
    }
}
