use crate::components::animation::hover_t;
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleVariant {
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleSize {
    Sm,
    Default,
    Lg,
}

impl ToggleSize {
    pub fn height(self) -> f32 {
        match self {
            Self::Sm => 28.0,
            Self::Default => 32.0,
            Self::Lg => 38.0,
        }
    }

    pub fn font_size(self) -> f32 {
        match self {
            Self::Sm => 11.5,
            Self::Default => 12.5,
            Self::Lg => 13.5,
        }
    }

    pub fn icon_size(self) -> f32 {
        match self {
            Self::Sm => 13.0,
            Self::Default => 14.5,
            Self::Lg => 16.0,
        }
    }

    pub fn padding_x(self) -> f32 {
        match self {
            Self::Sm => 8.0,
            Self::Default => 11.0,
            Self::Lg => 14.0,
        }
    }
}

pub struct Toggle<'a> {
    pressed: &'a mut bool,
    label: Option<&'a str>,
    icon: Option<Icon>,
    variant: ToggleVariant,
    size: ToggleSize,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Toggle<'a> {
    pub fn new(pressed: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            pressed,
            label: None,
            icon: None,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            enabled: true,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let height = self.size.height();
        let pad_x = self.size.padding_x();
        let font_size = self.size.font_size();
        let icon_size = self.size.icon_size();

        let mut content_width = 0.0;
        if self.icon.is_some() {
            content_width += icon_size;
            if self.label.is_some() {
                content_width += 6.0;
            }
        }
        if let Some(label) = self.label {
            let galley =
                ui.painter()
                    .layout_no_wrap(label.to_string(), FontId::proportional(font_size), Color32::WHITE);
            content_width += galley.size().x;
        }

        let total_width = (content_width + pad_x * 2.0).max(height);
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_width, height), sense);

        if self.enabled && response.clicked() {
            *self.pressed = !*self.pressed;
            response.mark_changed();
        }

        let is_pressed = *self.pressed;
        let hover = hover_t(
            ui.ctx(),
            response.id.with("toggle_hover"),
            self.enabled && response.hovered(),
        );

        let rounding = Rounding::same(6.0);

        // Background & border
        let (fill, stroke, text_color) = match self.variant {
            ToggleVariant::Default => {
                if is_pressed {
                    (self.theme.accent, Stroke::NONE, self.theme.accent_foreground)
                } else {
                    let bg = if hover > 0.001 {
                        self.theme.surface_hover.linear_multiply(hover)
                    } else {
                        Color32::TRANSPARENT
                    };
                    (
                        bg,
                        Stroke::NONE,
                        if self.enabled {
                            self.theme.text_secondary
                        } else {
                            self.theme.text_disabled
                        },
                    )
                }
            }
            ToggleVariant::Outline => {
                if is_pressed {
                    (
                        self.theme.surface_hover,
                        Stroke::new(1.0, self.theme.border_strong),
                        self.theme.text_primary,
                    )
                } else {
                    let stroke = Stroke::new(
                        1.0,
                        if hover > 0.001 {
                            self.theme.border_strong
                        } else {
                            self.theme.border_default
                        },
                    );
                    let bg = if hover > 0.001 {
                        self.theme.surface_hover.linear_multiply(hover * 0.5)
                    } else {
                        Color32::TRANSPARENT
                    };
                    (
                        bg,
                        stroke,
                        if self.enabled {
                            self.theme.text_secondary
                        } else {
                            self.theme.text_disabled
                        },
                    )
                }
            }
        };

        ui.painter().rect_filled(rect, rounding, fill);
        if stroke != Stroke::NONE {
            ui.painter().rect_stroke(rect, rounding, stroke);
        }

        // Draw icon & text centered
        let start_x = rect.center().x - (content_width * 0.5);
        let mut cur_x = start_x;
        let center_y = rect.center().y;

        if let Some(icon) = self.icon {
            let icon_pos = Pos2::new(cur_x, center_y);
            ui.painter().text(
                icon_pos,
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(icon_size, FontFamily::Name("lucide".into())),
                text_color,
            );
            cur_x += icon_size;
            if self.label.is_some() {
                cur_x += 6.0;
            }
        }

        if let Some(label) = self.label {
            let text_pos = Pos2::new(cur_x, center_y);
            ui.painter().text(
                text_pos,
                Align2::LEFT_CENTER,
                label,
                DbProTheme::ui_medium_font(font_size),
                text_color,
            );
        }

        if self.enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        }
    }
}

pub struct ToggleGroupItem<'a, T: Clone + PartialEq> {
    pub value: T,
    pub label: Option<&'a str>,
    pub icon: Option<Icon>,
    pub tooltip: Option<&'a str>,
}

impl<'a, T: Clone + PartialEq> ToggleGroupItem<'a, T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            label: None,
            icon: None,
            tooltip: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn tooltip(mut self, tip: &'a str) -> Self {
        self.tooltip = Some(tip);
        self
    }
}

pub struct ToggleGroup<'a, T: Clone + PartialEq> {
    items: Vec<ToggleGroupItem<'a, T>>,
    size: ToggleSize,
    theme: DbProTheme,
}

impl<'a, T: Clone + PartialEq> ToggleGroup<'a, T> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            items: Vec::new(),
            size: ToggleSize::Default,
            theme,
        }
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn item(mut self, item: ToggleGroupItem<'a, T>) -> Self {
        self.items.push(item);
        self
    }

    /// Renders single-select toggle group.
    pub fn show_single(self, ui: &mut Ui, selected: &mut T) -> Option<T> {
        let mut changed_to = None;
        let count = self.items.len();
        if count == 0 {
            return None;
        }

        let height = self.size.height();
        let pad_x = self.size.padding_x();
        let font_size = self.size.font_size();
        let icon_size = self.size.icon_size();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            for (index, it) in self.items.into_iter().enumerate() {
                let is_selected = it.value == *selected;
                let is_first = index == 0;
                let is_last = index + 1 == count;

                let rounding = if count == 1 {
                    Rounding::same(6.0)
                } else if is_first {
                    Rounding {
                        nw: 6.0,
                        sw: 6.0,
                        ne: 0.0,
                        se: 0.0,
                    }
                } else if is_last {
                    Rounding {
                        nw: 0.0,
                        sw: 0.0,
                        ne: 6.0,
                        se: 6.0,
                    }
                } else {
                    Rounding::ZERO
                };

                let mut w = pad_x * 2.0;
                if it.icon.is_some() {
                    w += icon_size;
                    if it.label.is_some() {
                        w += 6.0;
                    }
                }
                if let Some(label) = it.label {
                    let galley =
                        ui.painter()
                            .layout_no_wrap(label.to_string(), FontId::proportional(font_size), Color32::WHITE);
                    w += galley.size().x;
                }
                w = w.max(height);

                let (rect, response) = ui.allocate_exact_size(Vec2::new(w, height), Sense::click());
                let mut resp = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if let Some(tip) = it.tooltip {
                    resp = resp.on_hover_text(tip);
                }

                if resp.clicked() && !is_selected {
                    *selected = it.value.clone();
                    changed_to = Some(it.value.clone());
                }

                let hover = hover_t(ui.ctx(), resp.id.with("tg_hover"), resp.hovered());

                let fill = if is_selected {
                    self.theme.accent
                } else if hover > 0.001 {
                    self.theme.surface_hover.linear_multiply(hover)
                } else {
                    self.theme.surface_panel
                };

                ui.painter().rect_filled(rect, rounding, fill);
                ui.painter()
                    .rect_stroke(rect, rounding, Stroke::new(1.0, self.theme.border_default));

                let text_color = if is_selected {
                    self.theme.accent_foreground
                } else if resp.hovered() {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };

                // Draw contents
                let mut cur_x = rect.left() + pad_x;
                let center_y = rect.center().y;

                if let Some(icon) = it.icon {
                    ui.painter().text(
                        Pos2::new(cur_x, center_y),
                        Align2::LEFT_CENTER,
                        char::from(icon).to_string(),
                        FontId::new(icon_size, FontFamily::Name("lucide".into())),
                        text_color,
                    );
                    cur_x += icon_size;
                    if it.label.is_some() {
                        cur_x += 6.0;
                    }
                }

                if let Some(label) = it.label {
                    ui.painter().text(
                        Pos2::new(cur_x, center_y),
                        Align2::LEFT_CENTER,
                        label,
                        DbProTheme::ui_medium_font(font_size),
                        text_color,
                    );
                }
            }
        });

        changed_to
    }
}
