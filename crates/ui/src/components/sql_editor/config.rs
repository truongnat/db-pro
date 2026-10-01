//! Sizing and layout configuration for SqlEditorToolbar.

use crate::tokens::{RADIUS_MD, SPACE_SM, SPACE_XS};
use egui::{Margin, Rounding};

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
