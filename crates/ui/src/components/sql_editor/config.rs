//! Sizing and layout configuration for SqlEditorToolbar.

use crate::tokens::{RADIUS_MD, SPACE_SM, SPACE_XS};
use egui::{CornerRadius, Margin};

/// Inner margin for the toolbar panel frame.
pub const TOOLBAR_MARGIN: Margin = Margin {
    left: (SPACE_SM) as i8,
    right: (SPACE_SM) as i8,
    top: (SPACE_XS) as i8,
    bottom: (SPACE_XS) as i8,
};

/// CornerRadius applied to the toolbar panel top corners.
pub const TOOLBAR_ROUNDING: CornerRadius = CornerRadius {
    nw: (RADIUS_MD) as u8,
    ne: (RADIUS_MD) as u8,
    sw: 0.0 as u8,
    se: 0.0 as u8,
};
