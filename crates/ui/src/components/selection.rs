use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: &'a str,
    description: Option<&'a str>,
    enabled: bool,
    theme: DbProTheme,
}

pub type ShadcnCheckbox<'a> = Checkbox<'a>;

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
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(size, size), Sense::click());

        if self.enabled && response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }

        let rounding = Rounding::same(4.0);

        if *self.checked {
            ui.painter().rect_filled(rect, rounding, self.theme.accent);
            let check_icon = char::from(Icon::Check).to_string();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                check_icon,
                FontId::new(11.0, FontFamily::Name("lucide".into())),
                self.theme.accent_foreground,
            );
        } else {
            let fill = if response.hovered() && self.enabled {
                self.theme.surface_hover
            } else {
                self.theme.surface_editor
            };
            let stroke = if response.hovered() && self.enabled {
                Stroke::new(1.0, self.theme.border_strong)
            } else {
                Stroke::new(1.0, self.theme.border_default)
            };
            ui.painter().rect_filled(rect, rounding, fill);
            ui.painter().rect_stroke(rect, rounding, stroke);
        }

        ui.add_space(8.0);
        ui.vertical(|ui| {
            let text_color = if self.enabled {
                self.theme.text_primary
            } else {
                self.theme.text_muted
            };
            let label_resp = ui.label(RichText::new(self.label).size(13.0).color(text_color));
            if self.enabled && label_resp.clicked() {
                *self.checked = !*self.checked;
                response.mark_changed();
            }

            if let Some(desc) = self.description {
                ui.label(RichText::new(desc).size(11.5).color(self.theme.text_muted));
            }
        });

        response
    }
}

pub struct Switch<'a> {
    on: &'a mut bool,
    label: Option<&'a str>,
    description: Option<&'a str>,
    enabled: bool,
    theme: DbProTheme,
}

pub type ShadcnSwitch<'a> = Switch<'a>;

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
        let width = 36.0;
        let height = 20.0;
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        if self.enabled && response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        let rounding = Rounding::same(height * 0.5);
        let bg_color = if *self.on {
            self.theme.accent
        } else {
            self.theme.border_default
        };

        ui.painter().rect_filled(rect, rounding, bg_color);

        // Knob
        let knob_radius = (height - 4.0) * 0.5;
        let knob_x = if *self.on {
            rect.right() - 2.0 - knob_radius
        } else {
            rect.left() + 2.0 + knob_radius
        };
        let knob_center = Pos2::new(knob_x, rect.center().y);
        ui.painter().circle_filled(knob_center, knob_radius, Color32::WHITE);

        if self.label.is_some() || self.description.is_some() {
            ui.add_space(8.0);
            ui.vertical(|ui| {
                if let Some(label) = self.label {
                    let text_color = if self.enabled {
                        self.theme.text_primary
                    } else {
                        self.theme.text_muted
                    };
                    let label_resp = ui.label(RichText::new(label).size(13.0).strong().color(text_color));
                    if self.enabled && label_resp.clicked() {
                        *self.on = !*self.on;
                        response.mark_changed();
                    }
                }
                if let Some(desc) = self.description {
                    ui.label(RichText::new(desc).size(11.5).color(self.theme.text_muted));
                }
            });
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

pub type ShadcnRadio<'a> = Radio<'a>;

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
        let (rect, response) = ui.allocate_exact_size(Vec2::new(size, size), Sense::click());

        let center = rect.center();
        let radius = size * 0.5;

        if self.selected {
            ui.painter()
                .circle_stroke(center, radius, Stroke::new(1.5, self.theme.accent));
            ui.painter().circle_filled(center, radius - 4.0, self.theme.accent);
        } else {
            let stroke_color = if response.hovered() && self.enabled {
                self.theme.border_strong
            } else {
                self.theme.border_default
            };
            ui.painter()
                .circle_stroke(center, radius, Stroke::new(1.0, stroke_color));
            ui.painter()
                .circle_filled(center, radius - 1.0, self.theme.surface_editor);
        }

        ui.add_space(8.0);
        ui.vertical(|ui| {
            let text_color = if self.enabled {
                self.theme.text_primary
            } else {
                self.theme.text_muted
            };
            ui.label(RichText::new(self.label).size(13.0).color(text_color));
            if let Some(desc) = self.description {
                ui.label(RichText::new(desc).size(11.5).color(self.theme.text_muted));
            }
        });

        response
    }
}

pub struct Slider<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: Option<&'a str>,
    show_value: bool,
    width: Option<f32>,
    theme: DbProTheme,
}

pub type ShadcnSlider<'a> = Slider<'a>;

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
            let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());

            let min = *self.range.start();
            let max = *self.range.end();
            let range_span = (max - min).max(0.001);

            if response.dragged() || response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let normalized = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    *self.value = min + normalized * range_span;
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

            response
        })
        .inner
    }
}
