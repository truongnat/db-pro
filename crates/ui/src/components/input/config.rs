use crate::tokens::{RADIUS_SM, SPACE_SM, SPACE_XS};

/// Smallest usable text field width. Fields inside narrow containers (clamped dialogs,
/// sheets, split panes) may shrink to this but no further.
pub const INPUT_MIN_WIDTH: f32 = 120.0;
pub const INPUT_ROUNDING: f32 = RADIUS_SM; // 6.0
pub const FIELD_INNER_MARGIN_X: f32 = SPACE_SM; // 8.0
pub const FIELD_INNER_MARGIN_Y: f32 = SPACE_XS; // 4.0
