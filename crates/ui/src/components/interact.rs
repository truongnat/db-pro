use crate::tokens::STROKE_THICK;
use crate::DbProTheme;
use egui::{CornerRadius, Rect, Stroke, Ui, WidgetInfo, WidgetType};

/// Shared focus ring: `border.focus` drawn at `STROKE_THICK`, expanded by the
/// same amount so ring and rounding stay concentric.
pub fn paint_focus_ring(ui: &Ui, rect: Rect, corner_radius: f32, theme: DbProTheme) {
    ui.painter().rect_stroke(
        rect.expand(STROKE_THICK),
        CornerRadius::same((corner_radius + STROKE_THICK) as u8),
        Stroke::new(STROKE_THICK, theme.border_focus),
        egui::StrokeKind::Inside,
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

pub fn combo_box_info(enabled: bool, label: &str) -> WidgetInfo {
    WidgetInfo::labeled(WidgetType::ComboBox, enabled, label)
}
