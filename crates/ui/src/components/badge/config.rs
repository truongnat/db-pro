use crate::tokens::RADIUS_BADGE;

/// Badge corner radius, in egui points, shared with the canonical badge token.
pub const BADGE_RADIUS: f32 = RADIUS_BADGE;
/// Standard badge label font size, in egui points, for dense workstation metadata.
pub const BADGE_FONT_SIZE: f32 = 12.0;
/// Standard Lucide icon size, in egui points, used when a badge has a leading icon.
pub const BADGE_ICON_SIZE: f32 = 12.0;
/// Standard horizontal content padding, in egui points.
pub const BADGE_PAD_X: f32 = 8.0;
/// Standard vertical content padding, in egui points.
pub const BADGE_PAD_Y: f32 = 2.0;
/// Standard gap, in egui points, between the leading dot/icon and the label.
pub const BADGE_GAP: f32 = 6.0;
/// Standard status-dot diameter, in egui points.
pub const BADGE_DOT: f32 = 6.0;
/// Standard minimum badge height, in egui points, preserving a compact 20px target.
pub const BADGE_MIN_HEIGHT: f32 = 20.0;
/// Compact badge label font size, in egui points, for especially tight metadata rows.
pub const COMPACT_BADGE_FONT_SIZE: f32 = 10.5;
/// Compact horizontal content padding, in egui points.
pub const COMPACT_BADGE_PAD_X: f32 = 6.0;
/// Compact vertical content padding, in egui points.
pub const COMPACT_BADGE_PAD_Y: f32 = 1.5;
/// Compact minimum badge height, in egui points.
pub const COMPACT_BADGE_MIN_HEIGHT: f32 = 18.0;
/// Compact status-dot diameter, in egui points.
pub const COMPACT_BADGE_DOT: f32 = 4.0;
/// Compact Lucide icon size, in egui points.
pub const COMPACT_BADGE_ICON_SIZE: f32 = 9.5;
/// Compact gap, in egui points, between the leading dot/icon and the label.
pub const COMPACT_BADGE_GAP: f32 = 4.0;
