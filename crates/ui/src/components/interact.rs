use egui::{CornerRadius, Id, Rect, Stroke, Ui, WidgetInfo, WidgetType};

use crate::{tokens::STROKE_THICK, DbProTheme};

/// `ctx.data` key recording whether the last interaction came from the
/// keyboard. Updated once per frame by [`update_focus_visible`]; default
/// `false` so a freshly-opened app paints no rings until the user navigates
/// by keyboard.
const FOCUS_VISIBLE_ID: &str = "db_pro.focus_visible";

/// Focus-visible tracking, called once per frame from `App::logic`.
///
/// egui grants `response.has_focus()` to widgets that request focus on
/// pointer click (tabs, calendar cells, radio, …), so `has_focus` alone
/// cannot drive the ring — it would paint on every mouse click. We mirror
/// the browser's `:focus-visible`: keyboard focus keys (Tab, arrows, Enter,
/// Space) mark the session keyboard-driven; a pointer press switches it
/// back off. The flag lives in `ctx.data` so every paint site reads the
/// same value within a frame.
pub fn update_focus_visible(ctx: &egui::Context) {
    let (pointer_used, focus_key) = ctx.input(|input| {
        (
            input.pointer.any_pressed() || input.pointer.any_released(),
            input.key_pressed(egui::Key::Tab)
                || input.key_pressed(egui::Key::ArrowUp)
                || input.key_pressed(egui::Key::ArrowDown)
                || input.key_pressed(egui::Key::ArrowLeft)
                || input.key_pressed(egui::Key::ArrowRight)
                || input.key_pressed(egui::Key::Enter)
                || input.key_pressed(egui::Key::Space),
        )
    });
    let visible = if pointer_used {
        false
    } else if focus_key {
        true
    } else {
        focus_visible(ctx)
    };
    ctx.data_mut(|d| d.insert_temp(Id::new(FOCUS_VISIBLE_ID), visible));
}

/// Whether the focus ring should render this frame (keyboard-driven focus).
/// False by default and after pointer interaction.
pub fn focus_visible(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp(Id::new(FOCUS_VISIBLE_ID)).unwrap_or(false))
}

/// Shared focus ring: `border.focus` drawn at `STROKE_THICK`, expanded by the
/// same amount so ring and rounding stay concentric. No-op unless the last
/// interaction was keyboard navigation — see [`update_focus_visible`].
pub fn paint_focus_ring(ui: &Ui, rect: Rect, corner_radius: f32, theme: DbProTheme) {
    if !focus_visible(ui.ctx()) {
        return;
    }
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
