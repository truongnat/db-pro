use crate::tokens::{SPACE_SM, SPACE_XXS};

/// Default line thickness in points/pixels for both horizontal and vertical separators.
pub const DEFAULT_THICKNESS: f32 = 1.0;

/// Default outer margin around a horizontal separator (top and bottom).
pub const DEFAULT_HORIZONTAL_MARGIN: f32 = SPACE_SM; // 8.0

/// Default outer margin around a vertical separator (left and right).
pub const DEFAULT_VERTICAL_MARGIN: f32 = 6.0;

/// Minimum allocated height in points for a horizontal separator bounding box.
/// Ensures adequate touch and visual separation even with minimal margins.
pub const MIN_HORIZONTAL_HEIGHT: f32 = 14.0;

/// Maximum height in points for a vertical separator within its parent row or toolbar.
pub const MAX_VERTICAL_HEIGHT: f32 = 24.0;

/// Font size in points used for horizontal separator labels.
pub const LABEL_FONT_SIZE: f32 = 11.0;

/// Horizontal padding in points between divider line endpoints and label text.
pub const LABEL_PADDING: f32 = SPACE_SM; // 8.0

/// Vertical inset in points applied from top and bottom boundaries for vertical separator lines.
pub const VERTICAL_LINE_INSET: f32 = SPACE_XXS; // 2.0
