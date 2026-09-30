// Horizontal offset between a plan node and its child row, in egui points.
pub const EXPLAIN_ROW_INDENT: f32 = 18.0;
// Fixed track width for each node's relative cost/time indicator, in egui points.
pub const EXPLAIN_BAR_WIDTH: f32 = 44.0;
// Height of the cost/time indicator track, in egui points.
pub const EXPLAIN_BAR_HEIGHT: f32 = 5.0;
// Corner radius for the narrow cost/time indicator, in egui points.
pub const EXPLAIN_BAR_ROUNDING: f32 = 2.0;
// Height shared by hotspot and row-skew badges, in egui points.
pub const EXPLAIN_BADGE_HEIGHT: f32 = 16.0;
// Horizontal padding inside the hotspot badge, in egui points.
pub const EXPLAIN_BADGE_PAD_X: f32 = 8.0;
// Vertical placement inset for badge backgrounds, in egui points.
pub const EXPLAIN_BADGE_PAD_Y: f32 = 2.0;
// Left inset between hotspot badge edge and its text, in egui points.
pub const EXPLAIN_BADGE_TEXT_PAD_X: f32 = 4.0;
// Left inset between row-skew badge edge and its text, in egui points.
pub const EXPLAIN_SKEW_BADGE_TEXT_PAD_X: f32 = 3.0;
// Extra nesting offset for node findings beneath their owning plan row, in egui points.
pub const EXPLAIN_FINDING_INDENT_OFFSET: f32 = 24.0;
// Additional top inset for text painted into badges, in egui points.
pub const EXPLAIN_BADGE_TEXT_TOP_OFFSET: f32 = 1.0;
// Horizontal padding around the row-skew badge text, in egui points.
pub const EXPLAIN_SKEW_BADGE_PAD_X: f32 = 6.0;
// Do not paint flame-bar fill for node shares too small to be visually legible.
pub const EXPLAIN_MIN_BAR_RATIO: f32 = 0.01;
// Cost/time share above which a plan node is highlighted as a hotspot.
pub const EXPLAIN_HOTSPOT_RATIO: f32 = 0.45;
// Cost/time share above which a plan node receives a warning color.
pub const EXPLAIN_WARNING_RATIO: f32 = 0.20;
// Planner estimate ratios below this boundary receive a row-skew warning.
pub const EXPLAIN_ROW_SKEW_MIN_RATIO: f64 = 0.1;
// Planner estimate ratios above this boundary receive a row-skew warning.
pub const EXPLAIN_ROW_SKEW_MAX_RATIO: f64 = 10.0;
