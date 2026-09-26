use super::config::{INPUT_MIN_WIDTH, INPUT_ROUNDING};
use crate::components::animation::{hover_t, lerp_color};
use crate::DbProTheme;
use egui::{Id, Rect, Rounding, Stroke, Ui};

/// Fields fill their container by default. Requested widths are capped to the available
/// space; the preferred minimum only applies when the parent can accommodate it.
pub fn resolve_field_width(requested: Option<f32>, available: f32) -> f32 {
    let available = available.max(0.0);
    requested
        .unwrap_or(available)
        .clamp(INPUT_MIN_WIDTH.min(available), available)
}

pub fn paint_field_chrome(
    ui: &Ui,
    id: Id,
    rect: Rect,
    focused: bool,
    hovered: bool,
    enabled: bool,
    theme: DbProTheme,
    has_error: bool,
) {
    // Keep the border inside the field's own rect (not on or around it) so no container
    // can clip it, and so we do not stack a gray Frame border under a blue outline.
    let stroke = if has_error {
        Stroke::new(1.5, theme.danger)
    } else if focused {
        Stroke::new(1.5, theme.accent)
    } else if !enabled {
        return;
    } else {
        let hover = hover_t(ui.ctx(), id.with("input_hover"), hovered);
        Stroke::new(1.0, lerp_color(theme.border_default, theme.border_strong, hover))
    };

    // `Shape::rect_stroke` paints *entirely outside* the path (`StrokeKind::Outside`,
    // epaint `tessellator.rs`), so stroking `rect` directly spills half the border past
    // the field. A container whose clip ends exactly at the field edge — the sidebar
    // column, a flush toolbar — then discards that overflow, and the two vertical edges
    // disappear while the horizontal ones survive (they sit well inside the clip).
    //
    // Stroking an inset path keeps every painted pixel within `rect` while preserving the
    // outer silhouette: `rect.shrink(w)` grown by an outside stroke of width `w` is
    // exactly `rect`, and `rounding - w` grown by the same stroke is `rounding`.
    ui.painter().rect_stroke(
        rect.shrink(stroke.width),
        Rounding::same((INPUT_ROUNDING - stroke.width).max(0.0)),
        stroke,
    );
}
