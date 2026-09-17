use super::config::{INPUT_MIN_WIDTH, INPUT_ROUNDING};
use crate::components::animation::{hover_t, lerp_color};
use crate::DbProTheme;
use egui::{Id, Rect, Rounding, Stroke, Ui};

/// Explicit min/max width contract for form fields: a field fills its container by
/// default, a requested width never exceeds the container (long labels or values cannot
/// spill out of dialogs, sheets or split panes), and the result never drops below
/// [`INPUT_MIN_WIDTH`] so a field stays editable even in a very narrow parent.
pub fn resolve_field_width(requested: Option<f32>, available: f32) -> f32 {
    requested
        .unwrap_or(available)
        .clamp(INPUT_MIN_WIDTH, available.max(INPUT_MIN_WIDTH))
}

pub fn paint_field_chrome(ui: &Ui, id: Id, rect: Rect, focused: bool, hovered: bool, enabled: bool, theme: DbProTheme) {
    // Stroke on the field rect (not expand) so sidebar/toolbars cannot clip a
    // halo, and so we do not stack a gray Frame border under a blue outline.
    if focused {
        ui.painter().rect_stroke(
            rect,
            Rounding::same(INPUT_ROUNDING),
            Stroke::new(1.5, theme.accent),
        );
        return;
    }
    if !enabled {
        return;
    }
    let hover = hover_t(ui.ctx(), id.with("input_hover"), hovered);
    let border = lerp_color(theme.border_default, theme.border_strong, hover);
    ui.painter()
        .rect_stroke(rect, Rounding::same(INPUT_ROUNDING), Stroke::new(1.0, border));
}
