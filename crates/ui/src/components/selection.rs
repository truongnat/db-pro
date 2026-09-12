use crate::components::animation::{hover_t, lerp_color};
use crate::components::interact::{checkbox_info, paint_focus_ring, radio_info};
use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};

pub struct Checkbox<'a> {
    pub(crate) checked: &'a mut bool,
    pub(crate) label: &'a str,
    pub(crate) description: Option<&'a str>,
    pub(crate) enabled: bool,
    pub(crate) theme: DbProTheme,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: &'a str, theme: DbProTheme) -> Self {
        Self {
            checked,
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
        let size = 16.0;
        let spacing = 8.0;

        ui.horizontal(|ui| {
            let desc_extra = if self.description.is_some() { 16.0 } else { 0.0 };
            let total_height = 20.0 + desc_extra;

            let text_font = FontId::proportional(13.0);
            let text_galley = ui.painter().layout_no_wrap(
                self.label.to_owned(),
                text_font,
                if self.enabled {
                    self.theme.text_primary
                } else {
                    self.theme.text_muted
                },
            );
            let row_width = (size + spacing + text_galley.size().x).max(size + spacing + 60.0);

            let (rect, mut response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), Sense::click());
            response.widget_info(|| checkbox_info(self.enabled, *self.checked, self.label));

            if self.enabled && response.clicked() {
                *self.checked = !*self.checked;
                response.mark_changed();
            }

            let box_y = if self.description.is_some() {
                rect.top() + 2.0
            } else {
                rect.center().y - (size * 0.5)
            };
            let box_rect = Rect::from_min_size(Pos2::new(rect.left(), box_y), Vec2::splat(size));
            let rounding = Rounding::same(4.0);

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
                    1.0,
                    lerp_color(self.theme.border_default, self.theme.border_strong, hover),
                );
                ui.painter().rect_filled(box_rect, rounding, fill);
                ui.painter().rect_stroke(box_rect, rounding, stroke);
            }

            if response.has_focus() {
                paint_focus_ring(ui, box_rect, 4.0, self.theme);
            }

            // Text and description
            let text_pos = Pos2::new(rect.left() + size + spacing, box_y - 1.0);
            ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

            if let Some(desc) = self.description {
                let desc_galley =
                    ui.painter()
                        .layout_no_wrap(desc.to_owned(), FontId::proportional(11.5), self.theme.text_muted);
                let desc_pos = Pos2::new(rect.left() + size + spacing, text_pos.y + 16.0);
                ui.painter().galley(desc_pos, desc_galley, Color32::PLACEHOLDER);
            }

            if self.enabled {
                response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
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
        let width: f32 = 36.0;
        let height: f32 = 20.0;
        let spacing: f32 = 8.0;

        ui.horizontal(|ui| {
            let desc_extra = if self.description.is_some() { 16.0 } else { 0.0 };
            let total_height = height.max(20.0 + desc_extra);

            let label_width = if let Some(lbl) = self.label {
                let text_font = FontId::proportional(13.0);
                ui.painter()
                    .layout_no_wrap(lbl.to_owned(), text_font, self.theme.text_primary)
                    .size()
                    .x
            } else {
                0.0
            };
            let row_width = (width + spacing + label_width).max(width);

            let (row_rect, mut response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), Sense::click());
            response.widget_info(|| checkbox_info(self.enabled, *self.on, self.label.unwrap_or("Switch")));

            if self.enabled && response.clicked() {
                *self.on = !*self.on;
                response.mark_changed();
            }

            let switch_y = if self.description.is_some() {
                row_rect.top() + 1.0
            } else {
                row_rect.center().y - (height * 0.5)
            };
            let switch_rect = Rect::from_min_size(Pos2::new(row_rect.left(), switch_y), Vec2::new(width, height));
            let rounding = Rounding::same(height * 0.5);

            // Animated smooth transition for knob position
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

            // Smooth animated knob
            let knob_radius = (height - 4.0) * 0.5;
            let knob_x_left = switch_rect.left() + 2.0 + knob_radius;
            let knob_x_right = switch_rect.right() - 2.0 - knob_radius;
            let knob_x = egui::lerp(knob_x_left..=knob_x_right, anim_t);
            let knob_center = Pos2::new(knob_x, switch_rect.center().y);
            ui.painter().circle_filled(knob_center, knob_radius, Color32::WHITE);

            // Text and description
            if let Some(lbl) = self.label {
                let text_color = if self.enabled {
                    self.theme.text_primary
                } else {
                    self.theme.text_muted
                };
                let text_pos = Pos2::new(row_rect.left() + width + spacing, switch_y);
                let galley = ui
                    .painter()
                    .layout_no_wrap(lbl.to_owned(), FontId::proportional(13.0), text_color);
                ui.painter().galley(text_pos, galley, Color32::PLACEHOLDER);

                if let Some(desc) = self.description {
                    let desc_galley =
                        ui.painter()
                            .layout_no_wrap(desc.to_owned(), FontId::proportional(11.5), self.theme.text_muted);
                    let desc_pos = Pos2::new(row_rect.left() + width + spacing, text_pos.y + 16.0);
                    ui.painter().galley(desc_pos, desc_galley, Color32::PLACEHOLDER);
                }
            }

            if self.enabled {
                response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
            }

            response
        })
        .inner
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
        let size = 16.0;
        let spacing = 8.0;

        ui.horizontal(|ui| {
            let desc_extra = if self.description.is_some() { 16.0 } else { 0.0 };
            let total_height = 20.0 + desc_extra;

            let text_font = FontId::proportional(13.0);
            let text_galley = ui.painter().layout_no_wrap(
                self.label.to_owned(),
                text_font,
                if self.enabled {
                    self.theme.text_primary
                } else {
                    self.theme.text_muted
                },
            );
            let row_width = (size + spacing + text_galley.size().x).max(size + spacing + 60.0);

            let (rect, mut response) = ui.allocate_exact_size(Vec2::new(row_width, total_height), Sense::click());
            response.widget_info(|| radio_info(self.enabled, self.selected, self.label));

            let center_y = if self.description.is_some() {
                rect.top() + (size * 0.5) + 2.0
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
                    .circle_stroke(circle_center, radius, Stroke::new(1.5, self.theme.accent));
                ui.painter()
                    .circle_filled(circle_center, radius - 4.0, self.theme.accent);
            } else {
                let stroke_color = lerp_color(self.theme.border_default, self.theme.border_strong, hover);
                ui.painter()
                    .circle_stroke(circle_center, radius, Stroke::new(1.0, stroke_color));
                ui.painter()
                    .circle_filled(circle_center, radius - 1.0, self.theme.surface_editor);
            }

            if response.has_focus() {
                ui.painter()
                    .circle_stroke(circle_center, radius + 2.0, Stroke::new(2.0, self.theme.accent));
            }

            // Text and description
            let text_pos = Pos2::new(rect.left() + size + spacing, center_y - (size * 0.5) - 1.0);
            ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

            if let Some(desc) = self.description {
                let desc_galley =
                    ui.painter()
                        .layout_no_wrap(desc.to_owned(), FontId::proportional(11.5), self.theme.text_muted);
                let desc_pos = Pos2::new(rect.left() + size + spacing, text_pos.y + 16.0);
                ui.painter().galley(desc_pos, desc_galley, Color32::PLACEHOLDER);
            }

            if self.enabled {
                response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
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
                    if let Some(lbl) = self.label {
                        ui.label(RichText::new(lbl).size(12.0).strong().color(self.theme.text_secondary));
                    }
                    if self.show_value {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{:.0}", *self.value))
                                    .size(11.5)
                                    .strong()
                                    .color(self.theme.accent),
                            );
                        });
                    }
                });
                ui.add_space(4.0);
            }

            let height = 20.0;
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
                Pos2::new(rect.left(), rect.center().y - 2.5),
                Pos2::new(rect.right(), rect.center().y + 2.5),
            );
            ui.painter()
                .rect_filled(track_rect, Rounding::same(2.5), self.theme.border_default);

            // Active track
            let active_rect = Rect::from_min_max(
                Pos2::new(rect.left(), rect.center().y - 2.5),
                Pos2::new(rect.left() + normalized * rect.width(), rect.center().y + 2.5),
            );
            ui.painter()
                .rect_filled(active_rect, Rounding::same(2.5), self.theme.accent);

            // Thumb
            let thumb_x = rect.left() + normalized * rect.width();
            let thumb_center = Pos2::new(thumb_x, rect.center().y);
            let thumb_color = if response.hovered() || response.dragged() {
                self.theme.accent
            } else {
                Color32::WHITE
            };
            ui.painter().circle_filled(thumb_center, 7.0, thumb_color);
            ui.painter()
                .circle_stroke(thumb_center, 7.0, Stroke::new(1.5, self.theme.accent));

            // Focus ring around thumb
            if response.has_focus() {
                ui.painter()
                    .circle_stroke(thumb_center, 9.5, Stroke::new(2.0, self.theme.accent));
            }

            response
        })
        .inner
    }
}
