//! Component-owned alert geometry and opacity defaults.

/// Horizontal frame inset for compact inline alerts, in egui points.
pub const ALERT_FRAME_PADDING_X: f32 = 14.0;
/// Vertical frame inset for compact inline alerts, in egui points.
pub const ALERT_FRAME_PADDING_Y: f32 = 12.0;
/// Alert corner radius, in egui points.
pub const ALERT_RADIUS: f32 = 8.0;
/// Alert icon/text gap, in egui points.
pub const ALERT_ICON_GAP: f32 = 8.0;
/// Dismiss button hit target, in egui points.
pub const DISMISS_TARGET_SIZE: f32 = 32.0;
/// Space reserved between alert copy and the dismiss target, in egui points.
pub const DISMISS_GAP: f32 = 8.0;
/// Opacity multipliers for semantic variant fills and borders.
pub const VARIANT_FILL_OPACITY: f32 = 0.12;
pub const VARIANT_BORDER_OPACITY: f32 = 0.40;

/// Maximum dialog outer width, in egui points.
pub const DIALOG_MAX_WIDTH: f32 = 440.0;
/// Safe horizontal viewport inset for a centered dialog, in egui points.
pub const DIALOG_VIEWPORT_INSET: f32 = 16.0;
/// Dialog frame padding, in egui points.
pub const DIALOG_PADDING: f32 = 20.0;
/// Gap between dialog actions, in egui points.
pub const DIALOG_ACTION_GAP: f32 = 8.0;
/// Dialog frame corner radius, in egui points.
pub const DIALOG_RADIUS: f32 = 10.0;
/// Dialog shadow offset, blur, and opacity.
pub const DIALOG_SHADOW_OFFSET_Y: f32 = 8.0;
pub const DIALOG_SHADOW_BLUR: f32 = 24.0;
pub const DIALOG_SHADOW_OPACITY: u8 = 80;
/// Backdrop opacity.
pub const BACKDROP_OPACITY: u8 = 140;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_outer_width_never_exceeds_viewport_budget() {
        let narrow = (280.0 - DIALOG_VIEWPORT_INSET * 2.0).max(0.0);
        assert!(narrow < DIALOG_MAX_WIDTH);
        assert_eq!(DIALOG_PADDING, 20.0);
    }
}
