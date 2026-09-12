use crate::DbProTheme;
use egui::{Rect, Rounding, Stroke, Ui, WidgetInfo, WidgetType};

pub fn paint_focus_ring(ui: &Ui, rect: Rect, rounding: f32, theme: DbProTheme) {
    ui.painter().rect_stroke(
        rect.expand(2.0),
        Rounding::same(rounding + 2.0),
        Stroke::new(2.0, theme.accent),
    );
}

pub fn button_info(enabled: bool, label: &str) -> WidgetInfo {
    WidgetInfo::labeled(WidgetType::Button, enabled, label)
}

pub fn checkbox_info(enabled: bool, checked: bool, label: &str) -> WidgetInfo {
    WidgetInfo::selected(WidgetType::Checkbox, enabled, checked, label)
}

pub fn radio_info(enabled: bool, selected: bool, label: &str) -> WidgetInfo {
    WidgetInfo::selected(WidgetType::RadioButton, enabled, selected, label)
}

pub fn text_input_info(enabled: bool, label: &str) -> WidgetInfo {
    WidgetInfo::labeled(WidgetType::TextEdit, enabled, label)
}
