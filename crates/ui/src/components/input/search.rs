// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use egui::{Align, CornerRadius, Frame, Layout, Margin, Response, RichText, Stroke, TextEdit, Ui};
use lucide_icons::Icon;

use crate::{
    components::{
        button::{Button, ButtonSize, ButtonVariant},
        input::{
            config::{INPUT_ICON_SIZE, SEARCH_CLEAR_RESERVATION, SEARCH_SHORTCUT_RESERVATION},
            layout::{paint_field_chrome, resolve_field_width, FieldChromeState},
        },
    },
    tokens::{FONT_SIZE_KBD, RADIUS_INPUT, SPACE_SM, SPACE_XS},
    DbProTheme,
};

pub struct SearchInput<'a> {
    value: &'a mut String,
    placeholder: &'a str,
    shortcut: Option<&'a str>,
    width: Option<f32>,
    /// Keep a resting outline (`border.default`) instead of fill-only chrome.
    bordered: bool,
    theme: DbProTheme,
}

impl<'a> SearchInput<'a> {
    pub fn new(value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            value,
            placeholder,
            shortcut: None,
            width: None,
            bordered: false,
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

    /// Keep a resting outline (`border.default`) instead of fill-only chrome.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
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
            inner_margin: Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
            corner_radius: CornerRadius::same(RADIUS_INPUT as u8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_width(inner_w);
            ui.set_max_width(inner_w);
            // Bound the row height instead of inheriting the dialog's full remaining
            // height. Keep reading order left-to-right even in a right-aligned toolbar.
            ui.allocate_ui_with_layout(
                egui::vec2(inner_w, ui.spacing().interact_size.y),
                Layout::left_to_right(Align::Center),
                |ui| {
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
                            .frame(egui::Frame::NONE)
                            .text_color(self.theme.text_primary),
                    );
                    edit.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, ui.is_enabled(), self.placeholder)
                    });

                    if edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.value.clear();
                    }

                    if !self.value.is_empty()
                        && Button::new(self.theme)
                            .icon(Icon::X)
                            .access_label("Clear")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .show(ui)
                            .clicked()
                    {
                        self.value.clear();
                    }

                    if let Some(sc) = self.shortcut {
                        Frame {
                            fill: self.theme.surface_panel,
                            stroke: Stroke::new(1.0, self.theme.border_subtle),
                            inner_margin: Margin::symmetric(4.0 as i8, 1.0 as i8),
                            corner_radius: CornerRadius::same(4.0 as u8),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.label(RichText::new(sc).size(FONT_SIZE_KBD).color(self.theme.text_secondary));
                        });
                    }

                    edit
                },
            )
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
                bordered: self.bordered,
            },
            self.theme,
        );

        // Keep the subtle outline on search controls for discoverability. Standard
        // text inputs use their fill at rest and reserve outlines for interaction.
        if !edit_response.has_focus() && !frame_output.response.hovered() && !edit_response.hovered() {
            let width = crate::tokens::component::input::BORDER_STROKE_WIDTH;
            ui.painter().rect_stroke(
                frame_rect.shrink(width),
                CornerRadius::same((RADIUS_INPUT - width).max(0.0) as u8),
                Stroke::new(width, self.theme.border_subtle),
                egui::StrokeKind::Inside,
            );
        }

        edit_response
    }
}

#[cfg(test)]
#[path = "tests/search_tests.rs"]
mod tests;
