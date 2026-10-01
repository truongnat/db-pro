use super::config::INPUT_MIN_WIDTH;
use crate::components::animation::{hover_t, lerp_color};
use crate::tokens::component::input::{
    resolve_chrome, FieldChrome, BORDER_STROKE_WIDTH, ERROR_STROKE_WIDTH, FOCUS_STROKE_WIDTH,
};
use crate::tokens::RADIUS_XS;
use crate::DbProTheme;
use egui::{Id, Rect, Rounding, Stroke, Ui};

// The state struct is owned by the input contract; re-exported here so the
// input module keeps constructing field state through this module.
pub(crate) use crate::tokens::component::input::FieldChromeState;

/// Fields fill their container by default. Requested widths are capped to the available
/// space; the preferred minimum only applies when the parent can accommodate it.
pub fn resolve_field_width(requested: Option<f32>, available: f32) -> f32 {
    let available = available.max(0.0);
    requested
        .unwrap_or(available)
        .clamp(INPUT_MIN_WIDTH.min(available), available)
}

pub(crate) fn paint_field_chrome(ui: &Ui, id: Id, rect: Rect, state: FieldChromeState, theme: DbProTheme) {
    // Keep the border inside the field's own rect (not on or around it) so no container
    // can clip it, and so we do not stack a gray Frame border under a blue outline.
    //
    // Precedence comes from the input contract: disabled → error → focus → hover →
    // default, so a disabled field paints no border even while focused or errored.
    let stroke = match resolve_chrome(state) {
        FieldChrome::Disabled => return,
        FieldChrome::Error => Stroke::new(ERROR_STROKE_WIDTH, theme.danger),
        FieldChrome::Focus => Stroke::new(FOCUS_STROKE_WIDTH, theme.border_focus),
        chrome => {
            let hover = hover_t(ui.ctx(), id.with("input_hover"), matches!(chrome, FieldChrome::Hover));
            Stroke::new(
                BORDER_STROKE_WIDTH,
                lerp_color(theme.border_default, theme.border_strong, hover),
            )
        }
    };

    // egui paints a rectangle stroke outside its path, so stroking `rect` directly spills
    // beyond the field allocation. A parent clipped at the field edge can discard that
    // overflow, making the vertical border edges disappear while the horizontal edges remain.
    //
    // Stroking an inset path keeps every painted pixel within `rect` while preserving the
    // outer silhouette: `rect.shrink(w)` grown by an outside stroke of width `w` is
    // exactly `rect`, and `rounding - w` grown by the same stroke is `rounding`.
    ui.painter().rect_stroke(
        rect.shrink(stroke.width),
        Rounding::same((RADIUS_XS - stroke.width).max(0.0)),
        stroke,
    );
}
