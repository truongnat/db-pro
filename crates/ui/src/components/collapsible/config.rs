/// Height of the clickable collapsible header row in egui points.
pub const HEADER_HEIGHT: f32 = 36.0;

/// Medium-weight title font size for the compact disclosure header, in egui points.
pub const HEADER_TITLE_FONT_SIZE: f32 = 12.5;

/// Duration in seconds for the expand/collapse transition animation.
pub const OPEN_ANIMATION_SECONDS: f32 = 0.18;

/// Minimum animation progress threshold before allocating and rendering collapsible content.
pub const OPEN_CONTENT_THRESHOLD: f32 = 0.01;

/// Animation progress threshold where the chevron flips from right to down.
pub const CHEVRON_OPEN_THRESHOLD: f32 = 0.5;

/// Left padding offset for the leading chevron icon in egui points.
pub const HEADER_PAD_LEFT: f32 = 6.0;

/// Horizontal distance to advance after drawing the chevron in egui points.
pub const CHEVRON_ADVANCE: f32 = 18.0;

/// Horizontal distance to advance after drawing the optional leading icon in egui points.
pub const ICON_ADVANCE: f32 = 20.0;

/// Height of the badge pill background in egui points.
pub const BADGE_HEIGHT: f32 = 16.0;

/// Extra horizontal padding added to badge text galley in egui points.
pub const BADGE_PAD_H: f32 = 10.0;

/// Inset from the right edge of the header row to the badge pill in egui points.
pub const BADGE_RIGHT_MARGIN: f32 = 14.0;

/// Horizontal text offset within the badge pill in egui points.
pub const BADGE_TEXT_OFFSET_X: f32 = 5.0;

/// Vertical text offset within the badge pill in egui points.
pub const BADGE_TEXT_OFFSET_Y: f32 = 2.0;

/// Multiplier applied to hover animation progress for header surface fill alpha.
pub const HOVER_ALPHA_MULTIPLIER: f32 = 0.8;

/// Inner margin around the expanded collapsible body content.
pub const CONTENT_MARGIN: egui::Margin = egui::Margin {
    left: 20.0,
    right: 6.0,
    top: 4.0,
    bottom: 8.0,
};
