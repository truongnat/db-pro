use crate::components::animation::hover_t;
use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

use super::{config, handler};

/// A two-state control whose pressed value is owned by the caller.
pub struct Toggle<'a> {
    pressed: &'a mut bool,
    label: Option<&'a str>,
    icon: Option<Icon>,
    variant: handler::ToggleVariant,
    size: handler::ToggleSize,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Toggle<'a> {
    pub fn new(pressed: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            pressed,
            label: None,
            icon: None,
            variant: handler::ToggleVariant::Default,
            size: handler::ToggleSize::Default,
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

    pub fn variant(mut self, variant: handler::ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: handler::ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn show(self, ui: &mut Ui) -> Response {
        let size = self.size.tokens();
        let label_width = measure_label(ui, self.label, size.font_size);
        let content_width = handler::content_width(self.icon.is_some(), label_width, size.icon_size);
        let total_width = handler::control_width(content_width, size.padding_x, size.height);
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_width, size.height), sense);

        // The input event changes caller-owned state first. The resulting state is then
        // passed to the handler for color resolution, so painting never owns interaction state.
        if handler::apply_toggle_click(self.pressed, response.clicked(), self.enabled) {
            response.mark_changed();
        }

        let appearance = handler::standalone_appearance(
            self.variant,
            *self.pressed,
            self.enabled,
            hover_t(
                ui.ctx(),
                response.id.with("toggle_hover"),
                self.enabled && response.hovered(),
            ),
            self.theme,
        );

        ui.painter().rect_filled(rect, config::TOGGLE_ROUNDING, appearance.fill);
        if appearance.stroke != Stroke::NONE {
            ui.painter()
                .rect_stroke(rect, config::TOGGLE_ROUNDING, appearance.stroke);
        }

        // egui supplies the measured label width; the handler has already calculated the
        // content origin. This keeps text/icon painting here while centralizing layout rules.
        let start_x = rect.center().x - content_width * config::CONTENT_CENTER_FACTOR;
        paint_contents(
            ui,
            start_x,
            rect.center().y,
            ContentPaint {
                icon: self.icon,
                label: self.label,
                icon_size: size.icon_size,
                font_size: size.font_size,
                text_color: appearance.text_color,
            },
        );

        if self.enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        }
    }
}

/// One caller-owned option in a [`ToggleGroup`].
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

/// A contiguous single-select group of toggle items.
pub struct ToggleGroup<'a, T: Clone + PartialEq> {
    items: Vec<ToggleGroupItem<'a, T>>,
    size: handler::ToggleSize,
    theme: DbProTheme,
}

impl<'a, T: Clone + PartialEq> ToggleGroup<'a, T> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            items: Vec::new(),
            size: handler::ToggleSize::Default,
            theme,
        }
    }

    pub fn size(mut self, size: handler::ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn item(mut self, item: ToggleGroupItem<'a, T>) -> Self {
        self.items.push(item);
        self
    }

    /// Renders single-select toggle group.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn show_single(self, ui: &mut Ui, selected: &mut T) -> Option<T> {
        let count = self.items.len();
        if count == 0 {
            return None;
        }

        let size = self.size.tokens();
        let mut changed_to = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            for (index, item) in self.items.into_iter().enumerate() {
                let is_selected = item.value == *selected;
                let label_width = measure_label(ui, item.label, size.font_size);
                let content_width = handler::content_width(item.icon.is_some(), label_width, size.icon_size);
                let width = handler::control_width(content_width, size.padding_x, size.height);
                let (rect, response) = ui.allocate_exact_size(Vec2::new(width, size.height), Sense::click());
                let mut response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if let Some(tooltip) = item.tooltip {
                    response = response.on_hover_text(tooltip);
                }

                // Selection is a typed handler decision: clicking the active item is inert,
                // while a different value is cloned back through the caller's state reference.
                if let Some(next) = handler::apply_selection_click(selected, &item.value, response.clicked()) {
                    changed_to = Some(next);
                }

                let hover = hover_t(ui.ctx(), response.id.with("tg_hover"), response.hovered());
                let fill = handler::group_fill(is_selected, hover, self.theme);
                let text_color = handler::group_text_color(is_selected, response.hovered(), self.theme);
                let rounding = handler::group_rounding(index, count);

                ui.painter().rect_filled(rect, rounding, fill);
                ui.painter().rect_stroke(
                    rect,
                    rounding,
                    Stroke::new(config::OUTLINE_STROKE_WIDTH, self.theme.border_default),
                );
                paint_contents(
                    ui,
                    rect.left() + size.padding_x,
                    rect.center().y,
                    ContentPaint {
                        icon: item.icon,
                        label: item.label,
                        icon_size: size.icon_size,
                        font_size: size.font_size,
                        text_color,
                    },
                );
            }
        });

        changed_to
    }
}

fn measure_label(ui: &Ui, label: Option<&str>, font_size: f32) -> Option<f32> {
    label.map(|text| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), FontId::proportional(font_size), Color32::WHITE)
            .size()
            .x
    })
}

struct ContentPaint<'a> {
    icon: Option<Icon>,
    label: Option<&'a str>,
    icon_size: f32,
    font_size: f32,
    text_color: Color32,
}

fn paint_contents(ui: &Ui, start_x: f32, center_y: f32, content: ContentPaint<'_>) {
    let mut current_x = start_x;
    if let Some(icon) = content.icon {
        ui.painter().text(
            Pos2::new(current_x, center_y),
            Align2::LEFT_CENTER,
            char::from(icon).to_string(),
            FontId::new(content.icon_size, FontFamily::Name("lucide".into())),
            content.text_color,
        );
        current_x += content.icon_size;
        if content.label.is_some() {
            current_x += config::ICON_TEXT_GAP;
        }
    }

    if let Some(label) = content.label {
        ui.painter().text(
            Pos2::new(current_x, center_y),
            Align2::LEFT_CENTER,
            label,
            DbProTheme::ui_medium_font(content.font_size),
            content.text_color,
        );
    }
}
