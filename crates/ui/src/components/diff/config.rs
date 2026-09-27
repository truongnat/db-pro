/// Header bar height for diff viewer.
pub const DIFF_HEADER_HEIGHT: f32 = 34.0;
/// Outer frame and header top corners rounding radius, in egui points.
pub const DIFF_CORNER_RADIUS: f32 = 8.0;
/// Title font size in diff header.
pub const DIFF_TITLE_FONT_SIZE: f32 = 12.5;
/// Diff summary statistics font size (+X -Y).
pub const DIFF_STATS_FONT_SIZE: f32 = 11.5;

/// Row height for each individual diff line.
pub const DIFF_LINE_HEIGHT: f32 = 20.0;
/// Font size for line numbers (old and new columns).
pub const DIFF_LINE_NUM_FONT_SIZE: f32 = 11.0;
/// Font size for change marker (+ / - / space).
pub const DIFF_MARKER_FONT_SIZE: f32 = 12.0;
/// Font size for diff line text content.
pub const DIFF_CONTENT_FONT_SIZE: f32 = 12.0;

/// Width of one monospace line-number digit in egui points.
pub const DIFF_LINE_NUM_CHAR_WIDTH: f32 = 8.0;
/// Left inset of the old line-number column in egui points.
pub const DIFF_OLD_NUM_INSET: f32 = 8.0;
/// Gap between diff line-number columns in egui points.
pub const DIFF_LINE_NUM_COLUMN_GAP: f32 = 8.0;
/// Width reserved for the diff marker column in egui points.
pub const DIFF_MARKER_COLUMN_WIDTH: f32 = 14.0;
/// Right-side breathing room after the longest diff line, in egui points.
pub const DIFF_CONTENT_RIGHT_PADDING: f32 = 8.0;
/// Bottom padding space in the diff container.
pub const DIFF_BOTTOM_PADDING_Y: f32 = 6.0;
