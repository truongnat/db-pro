//! Sizing and layout configuration for SqlEditorToolbar.

use crate::components::button::ButtonSize;
use crate::tokens::{RADIUS_MD, SPACE_SM, SPACE_XS, SPACE_XXS, STROKE_THIN};
use egui::{Margin, Rounding};

/// Standard button size used across the SQL editor toolbar actions.
pub const TOOLBAR_BUTTON_SIZE: ButtonSize = ButtonSize::Sm;

/// Space between toolbar action buttons in egui points.
pub const BUTTON_SPACING: f32 = SPACE_XXS;

/// Inner margin for the toolbar panel frame.
pub const TOOLBAR_MARGIN: Margin = Margin {
    left: SPACE_SM,
    right: SPACE_SM,
    top: SPACE_XS,
    bottom: SPACE_XS,
};

/// Rounding applied to the toolbar panel top corners.
pub const TOOLBAR_ROUNDING: Rounding = Rounding {
    nw: RADIUS_MD,
    ne: RADIUS_MD,
    sw: 0.0,
    se: 0.0,
};

/// Border stroke thickness for the toolbar frame.
pub const TOOLBAR_STROKE_WIDTH: f32 = STROKE_THIN;
