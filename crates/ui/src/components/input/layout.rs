use egui::{CornerRadius, Id, Rect, Stroke, Ui};

use super::config::INPUT_MIN_WIDTH;
// The state struct is owned by the input contract; re-exported here so the
// input module keeps constructing field state through this module.
pub(crate) use crate::tokens::component::input::FieldChromeState;
use crate::{
    components::animation::{hover_t, lerp_color},
    tokens::{
        component::input::{resolve_chrome, FieldChrome, BORDER_STROKE_WIDTH, ERROR_STROKE_WIDTH, FOCUS_STROKE_WIDTH},
        RADIUS_INPUT,
    },
    DbProTheme,
};

/// Fields fill their container by default. Requested widths are capped to the available
/// space; the preferred minimum only applies when the parent can accommodate it.
pub fn resolve_field_width(requested: Option<f32>, available: f32) -> f32 {
    let available = available.max(0.0);
    requested
        .unwrap_or(available)
        .clamp(INPUT_MIN_WIDTH.min(available), available)
}

pub(crate) fn paint_field_chrome(ui: &Ui, id: Id, rect: Rect, state: FieldChromeState, theme: DbProTheme) {
    // Every enabled field is bounded at rest: `background.field` separates it from the
    // card it sits on and this `border.default` stroke draws the edge. A fill-only rest
    // state left text fields invisible whenever their fill matched the parent surface.
    //
    // Precedence comes from the input contract: disabled → error → focus → hover →
    // rest, so a disabled field paints no border even while focused or errored.
    let stroke = match resolve_chrome(state) {
        FieldChrome::Disabled => return,
        FieldChrome::Error => Stroke::new(ERROR_STROKE_WIDTH, theme.danger),
        FieldChrome::Focus => Stroke::new(FOCUS_STROKE_WIDTH, theme.border_focus),
        FieldChrome::Rest => Stroke::new(BORDER_STROKE_WIDTH, theme.border_default),
        FieldChrome::Hover => {
            let hover = hover_t(ui.ctx(), id.with("input_hover"), true);
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
        CornerRadius::same((RADIUS_INPUT - stroke.width).max(0.0) as u8),
        stroke,
        egui::StrokeKind::Inside,
    );
}
