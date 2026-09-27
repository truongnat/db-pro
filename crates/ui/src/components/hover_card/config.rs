//! HoverCard-owned sizing and timing defaults.

/// Default delay, in seconds, before a hovered/focused trigger opens the card.
pub const DEFAULT_OPEN_DELAY: f64 = 0.20;
/// Default delay, in seconds, before an open card closes after pointer/focus leaves.
pub const DEFAULT_CLOSE_DELAY: f64 = 0.15;
/// Default card width in egui points.
pub const DEFAULT_WIDTH: f32 = 300.0;
/// Minimum card width in egui points when the viewport is narrower than the requested width.
pub const MIN_WIDTH: f32 = 160.0;
/// Gap between trigger and card in egui points.
pub const TRIGGER_GAP: f32 = 6.0;
/// Minimum edge inset that keeps the floating card off the screen boundary.
pub const SCREEN_EDGE_INSET: f32 = 10.0;
/// Inner margin for the floating card frame in egui points.
pub const FRAME_INNER_MARGIN: f32 = 14.0;
/// Floating card corner radius in egui points.
pub const FRAME_ROUNDING: f32 = 8.0;
/// Smallest useful visible height before the card should prefer opening upward.
pub const MIN_VISIBLE_HEIGHT: f32 = 96.0;
/// Shadow offset Y in egui points.
pub const SHADOW_OFFSET_Y: f32 = 4.0;
/// Shadow blur in egui points.
pub const SHADOW_BLUR: f32 = 16.0;
/// Shadow alpha channel value (0..255).
pub const SHADOW_ALPHA: u8 = 40;
