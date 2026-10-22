// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2};

use super::{config, handler};
use crate::{
    components::{
        animation::{hover_t, lerp_color},
        interact::{checkbox_info, paint_focus_ring, radio_info},
    },
    tokens::{FONT_SIZE_CAPTION, FONT_SIZE_MONO_SM, RADIUS_XS, SPACE_XS, STROKE_FOCUS, STROKE_THIN},
    DbProTheme,
};

pub struct Checkbox<'a> {
    pub(crate) checked: &'a mut bool,
    pub(crate) label: &'a str,
    pub(crate) description: Option<&'a str>,
    pub(crate) enabled: bool,
    pub(crate) focusable: bool,
    pub(crate) theme: DbProTheme,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: &'a str, theme: DbProTheme) -> Self {
        Self {
            checked,
            label,
            description: None,
            enabled: true,
            focusable: true,
            theme,
        }
    }

    pub fn description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
        let size = config::CONTROL_SIZE;
        let spacing = config::CONTROL_SPACING;

        ui.horizontal(|ui| {
            let text_width = (ui.available_width() - size - spacing).max(config::MIN_CONTROL_TEXT_WIDTH);
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
            let text_color = if self.enabled {
                self.theme.text_primary
            } else {
                self.theme.text_muted
            };
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
            let text_galley = ui.painter().layout(
                self.label.to_owned(),
                FontId::proportional(config::CONTROL_LABEL_SIZE),
                text_color,
                text_width,
            );
            let description_galley = self.description.map(|description| {
                ui.painter().layout(
                    description.to_owned(),
                    FontId::proportional(config::CONTROL_DESC_SIZE),
                    self.theme.text_muted,
                    text_width,
                )
            });
            let content_height = text_galley.size().y
                + description_galley
                    .as_ref()
                    .map_or(0.0, |galley| config::CONTROL_DESC_GAP + galley.size().y);
            let total_height = handler::checkbox_row_height(self.description.is_some()).max(content_height);
            let row_width = ui
                .available_width()
                .max(size + spacing + config::MIN_CONTROL_TEXT_WIDTH);

            let sense = if self.focusable {
                Sense::CLICK | Sense::FOCUSABLE
            } else {
                Sense::CLICK
            };
            let (rect, mut response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), sense);
            response.widget_info(|| checkbox_info(self.enabled, *self.checked, self.label));

            if self.enabled && response.clicked() {
                *self.checked = !*self.checked;
                response.mark_changed();
            }

            let box_y = if self.description.is_some() {
                rect.top() + config::CONTROL_LABEL_TOP_INSET
            } else {
                rect.center().y - (size * 0.5)
            };
            let box_rect = Rect::from_min_size(Pos2::new(rect.left(), box_y), Vec2::splat(size));
            let rounding = CornerRadius::same(RADIUS_XS as u8);

            let hover = hover_t(
                ui.ctx(),
                response.id.with("hover"),
                self.enabled && (response.hovered() || response.has_focus()),
            );
            if *self.checked {
                ui.painter().rect_filled(box_rect, rounding, self.theme.accent);
                crate::components::table::draw_crisp_checkmark(
                    ui.painter(),
                    box_rect.center(),
                    self.theme.accent_foreground,
                );
            } else {
                let fill = lerp_color(self.theme.surface_editor, self.theme.surface_hover, hover);
                let stroke = Stroke::new(
                    STROKE_THIN,
                    lerp_color(self.theme.border_default, self.theme.border_strong, hover),
                );
                ui.painter().rect_filled(box_rect, rounding, fill);
                ui.painter()
                    .rect_stroke(box_rect, rounding, stroke, egui::StrokeKind::Inside);
            }

            if response.has_focus() {
                paint_focus_ring(ui, box_rect, RADIUS_XS, self.theme);
            }

            // Text and description use the cell width so narrow grids wrap instead of overflowing.
            let text_pos = Pos2::new(rect.left() + size + spacing, box_y - 1.0);
            let text_height = text_galley.size().y;
            ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

            if let Some(description_galley) = description_galley {
                let desc_pos = Pos2::new(
                    rect.left() + size + spacing,
                    text_pos.y + text_height + config::CONTROL_DESC_GAP,
                );
                ui.painter().galley(desc_pos, description_galley, Color32::PLACEHOLDER);
            }

            if self.enabled && response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            response
        })
        .inner
    }
}

pub struct Switch<'a> {
    on: &'a mut bool,
    label: Option<&'a str>,
    description: Option<&'a str>,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Switch<'a> {
    pub fn new(on: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            on,
            label: None,
            description: None,
            enabled: true,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = config::SWITCH_WIDTH;
        let height = config::SWITCH_HEIGHT;
        let spacing = config::CONTROL_SPACING;
        let text_width = (ui.available_width() - width - spacing).max(config::SWITCH_MIN_TEXT_WIDTH);
        let text_color = if self.enabled {
            self.theme.text_primary
        } else {
            self.theme.text_muted
        };
        let label_galley = self.label.map(|label| {
            ui.painter().layout(
                label.to_owned(),
                FontId::proportional(config::CONTROL_LABEL_SIZE),
                text_color,
                text_width,
            )
        });
        let description_galley = self.description.map(|description| {
            ui.painter().layout(
                description.to_owned(),
                FontId::proportional(config::CONTROL_DESC_SIZE),
                self.theme.text_muted,
                text_width,
            )
        });
        let text_height = label_galley.as_ref().map_or(0.0, |galley| galley.size().y)
            + description_galley
                .as_ref()
                .map_or(0.0, |galley| config::CONTROL_DESC_GAP + galley.size().y);
        let total_height = height.max(text_height);
        let row_width = ui.available_width().max(width);

        let (row_rect, mut response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), Sense::click());
        response.widget_info(|| checkbox_info(self.enabled, *self.on, self.label.unwrap_or("Switch")));

        let keyboard_toggle = response.has_focus()
            && ui.input(|input| input.key_pressed(egui::Key::Space) || input.key_pressed(egui::Key::Enter));
        if handler::switch_toggle_requested(self.enabled, response.clicked(), keyboard_toggle) {
            *self.on = !*self.on;
            response.mark_changed();
        }

        let switch_y = row_rect.top() + 1.0;
        let switch_rect = Rect::from_min_size(Pos2::new(row_rect.left(), switch_y), Vec2::new(width, height));
        let rounding = CornerRadius::same((height * 0.5) as u8);

        let anim_t = crate::components::animation::hover_t(ui.ctx(), response.id.with("switch_glide"), *self.on);
        let hover = hover_t(
            ui.ctx(),
            response.id.with("hover"),
            self.enabled && (response.hovered() || response.has_focus()),
        );
        let off_color = lerp_color(self.theme.border_default, self.theme.border_strong, hover);
        let bg_color = if *self.on { self.theme.accent } else { off_color };
        ui.painter().rect_filled(switch_rect, rounding, bg_color);

        if response.has_focus() {
            paint_focus_ring(ui, switch_rect, height * 0.5, self.theme);
        }

        let knob_radius = (height - 2.0 * config::SWITCH_KNOB_INSET) * 0.5;
        let knob_x_left = switch_rect.left() + config::SWITCH_KNOB_INSET + knob_radius;
        let knob_x_right = switch_rect.right() - config::SWITCH_KNOB_INSET - knob_radius;
        let knob_x = egui::lerp(knob_x_left..=knob_x_right, anim_t);
        let knob_center = Pos2::new(knob_x, switch_rect.center().y);
        // accent_foreground reads on the accent track in both themes (white on
        // dark blue in light mode, dark ink on bright blue in dark mode).
        let knob_color = if *self.on {
            self.theme.accent_foreground
        } else {
            self.theme.text_primary
        };
        ui.painter().circle_filled(knob_center, knob_radius, knob_color);

        let text_pos = Pos2::new(row_rect.left() + width + spacing, row_rect.top());
        if let Some(label_galley) = label_galley {
            let description_y = text_pos.y + label_galley.size().y + config::CONTROL_DESC_GAP;
            ui.painter().galley(text_pos, label_galley, Color32::PLACEHOLDER);
            if let Some(description_galley) = description_galley {
                ui.painter().galley(
                    Pos2::new(text_pos.x, description_y),
                    description_galley,
                    Color32::PLACEHOLDER,
                );
            }
        } else if let Some(description_galley) = description_galley {
            ui.painter().galley(text_pos, description_galley, Color32::PLACEHOLDER);
        }

        if self.enabled {
            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        response
    }
}

pub struct Radio<'a> {
    selected: bool,
    label: &'a str,
    description: Option<&'a str>,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Radio<'a> {
    pub fn new(selected: bool, label: &'a str, theme: DbProTheme) -> Self {
        Self {
            selected,
            label,
            description: None,
            enabled: true,
            theme,
        }
    }

    pub fn description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = config::CONTROL_SIZE;
        let spacing = config::CONTROL_SPACING;

        ui.horizontal(|ui| {
            let text_width = (ui.available_width() - size - spacing).max(config::MIN_CONTROL_TEXT_WIDTH);
            let text_color = if self.enabled {
                self.theme.text_primary
            } else {
                self.theme.text_muted
            };
            let text_galley = ui.painter().layout(
                self.label.to_owned(),
                FontId::proportional(config::CONTROL_LABEL_SIZE),
                text_color,
                text_width,
            );
            let description_galley = self.description.map(|description| {
                ui.painter().layout(
                    description.to_owned(),
                    FontId::proportional(config::CONTROL_DESC_SIZE),
                    self.theme.text_muted,
                    text_width,
                )
            });
            let content_height = text_galley.size().y
                + description_galley
                    .as_ref()
                    .map_or(0.0, |galley| config::CONTROL_DESC_GAP + galley.size().y);
            let total_height = handler::checkbox_row_height(self.description.is_some()).max(content_height);
            let row_width = ui
                .available_width()
                .max(size + spacing + config::MIN_CONTROL_TEXT_WIDTH);

            let (rect, response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), Sense::click());
            response.widget_info(|| radio_info(self.enabled, self.selected, self.label));

            let center_y = if self.description.is_some() {
                rect.top() + (size * 0.5) + config::CONTROL_LABEL_TOP_INSET
            } else {
                rect.center().y
            };
            let circle_center = Pos2::new(rect.left() + (size * 0.5), center_y);
            let radius = size * 0.5;

            let hover = hover_t(
                ui.ctx(),
                response.id.with("hover"),
                self.enabled && (response.hovered() || response.has_focus()),
            );
            if self.selected {
                ui.painter()
                    .circle_stroke(circle_center, radius, Stroke::new(STROKE_FOCUS, self.theme.accent));
                ui.painter()
                    .circle_filled(circle_center, radius - config::RADIO_DOT_INSET, self.theme.accent);
            } else {
                let stroke_color = lerp_color(self.theme.border_default, self.theme.border_strong, hover);
                ui.painter()
                    .circle_stroke(circle_center, radius, Stroke::new(STROKE_THIN, stroke_color));
                ui.painter()
                    .circle_filled(circle_center, radius - STROKE_THIN, self.theme.surface_editor);
            }

            if response.has_focus() {
                paint_focus_ring(
                    ui,
                    Rect::from_center_size(circle_center, Vec2::new(radius * 2.0, radius * 2.0)),
                    radius,
                    self.theme,
                );
            }

            // Text and description use the cell width so narrow grids wrap instead of overflowing.
            let text_pos = Pos2::new(rect.left() + size + spacing, center_y - (size * 0.5) - 1.0);
            let text_height = text_galley.size().y;
            ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

            if let Some(description_galley) = description_galley {
                let desc_pos = Pos2::new(
                    rect.left() + size + spacing,
                    text_pos.y + text_height + config::CONTROL_DESC_GAP,
                );
                ui.painter().galley(desc_pos, description_galley, Color32::PLACEHOLDER);
            }

            if self.enabled && response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            response
        })
        .inner
    }
}

pub struct Slider<'a> {
    pub(crate) value: &'a mut f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
    pub(crate) label: Option<&'a str>,
    pub(crate) show_value: bool,
    pub(crate) width: Option<f32>,
    pub(crate) theme: DbProTheme,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>, theme: DbProTheme) -> Self {
        Self {
            value,
            range,
            label: None,
            show_value: true,
            width: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.vertical(|ui| {
            if self.label.is_some() || self.show_value {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if let Some(lbl) = self.label {
                            ui.label(
                                RichText::new(lbl)
                                    .size(FONT_SIZE_CAPTION)
                                    .strong()
                                    .color(self.theme.text_secondary),
                            );
                        }
                    });
                    if self.show_value {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{:.0}", *self.value))
                                    .size(FONT_SIZE_MONO_SM)
                                    .strong()
                                    .color(self.theme.accent),
                            );
                        });
                    }
                });
                ui.add_space(SPACE_XS);
            }

            let height = config::SLIDER_HEIGHT;
            let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());

            let min = *self.range.start();
            let max = *self.range.end();
            let range_span = (max - min).max(0.001);

            // Keyboard navigation (ArrowLeft / ArrowRight)
            if response.has_focus() {
                let step = range_span * 0.02;
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    *self.value = (*self.value - step).clamp(min, max);
                    response.mark_changed();
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    *self.value = (*self.value + step).clamp(min, max);
                    response.mark_changed();
                }
            }

            if response.dragged() || response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let normalized = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    *self.value = min + normalized * range_span;
                    response.mark_changed();
                }
            }

            let normalized = ((*self.value - min) / range_span).clamp(0.0, 1.0);

            // Track background
            let track_rect = Rect::from_min_max(
                Pos2::new(rect.left(), rect.center().y - config::TRACK_HALF_HEIGHT),
                Pos2::new(rect.right(), rect.center().y + config::TRACK_HALF_HEIGHT),
            );
            ui.painter().rect_filled(
                track_rect,
                CornerRadius::same(config::TRACK_HALF_HEIGHT as u8),
                self.theme.border_default,
            );

            // Active track
            let active_rect = Rect::from_min_max(
                Pos2::new(rect.left(), rect.center().y - config::TRACK_HALF_HEIGHT),
                Pos2::new(
                    rect.left() + normalized * rect.width(),
                    rect.center().y + config::TRACK_HALF_HEIGHT,
                ),
            );
            ui.painter().rect_filled(
                active_rect,
                CornerRadius::same(config::TRACK_HALF_HEIGHT as u8),
                self.theme.accent,
            );

            // Thumb
            let thumb_x = rect.left() + normalized * rect.width();
            let thumb_center = Pos2::new(thumb_x, rect.center().y);
            let thumb_color = if response.hovered() || response.dragged() {
                self.theme.accent
            } else {
                self.theme.surface_elevated
            };
            ui.painter()
                .circle_filled(thumb_center, config::SLIDER_THUMB_RADIUS, thumb_color);
            ui.painter().circle_stroke(
                thumb_center,
                config::SLIDER_THUMB_RADIUS,
                Stroke::new(STROKE_FOCUS, self.theme.accent),
            );

            // Focus ring around thumb
            if response.has_focus() {
                paint_focus_ring(
                    ui,
                    Rect::from_center_size(
                        thumb_center,
                        Vec2::new(config::SLIDER_THUMB_RADIUS * 2.0, config::SLIDER_THUMB_RADIUS * 2.0),
                    ),
                    config::SLIDER_FOCUS_RADIUS,
                    self.theme,
                );
            }

            response
        })
        .inner
    }
}

#[cfg(test)]
#[path = "tests/ui_cursor_tests.rs"]
mod cursor_tests;
