//! Component-owned alert geometry and opacity defaults.

/// Horizontal frame inset for compact inline alerts, in egui points.
pub const ALERT_FRAME_PADDING_X: f32 = 14.0;
/// Vertical frame inset for compact inline alerts, in egui points.
pub const ALERT_FRAME_PADDING_Y: f32 = 12.0;
/// Alert corner radius, in egui points.
pub const ALERT_RADIUS: f32 = crate::tokens::RADIUS_CARD;
/// Alert icon/text gap, in egui points.
pub const ALERT_ICON_GAP: f32 = 8.0;
/// Dismiss button hit target, in egui points.
pub const DISMISS_TARGET_SIZE: f32 = 32.0;
/// Space reserved between alert copy and the dismiss target, in egui points.
pub const DISMISS_GAP: f32 = 8.0;
/// Compact alert border width, in egui points.
pub const ALERT_BORDER_WIDTH: f32 = crate::tokens::STROKE_THIN;
/// Compact alert title font size, in egui points.
pub const ALERT_TITLE_SIZE: f32 = crate::tokens::FONT_SIZE_UI_LABEL;
/// Compact alert description font size, in egui points.
pub const ALERT_DESCRIPTION_SIZE: f32 = 12.5;
/// Gap between compact alert title and description, in egui points.
pub const ALERT_DESCRIPTION_GAP: f32 = 3.0;
/// Compact alert icon size, in egui points.
pub const ALERT_ICON_SIZE: f32 = 15.0;

// Opacity multipliers for semantic variant fills and borders moved to
// `crate::tokens::semantic` (`STATUS_FILL_OPACITY`, `STATUS_BORDER_OPACITY`):
// they describe the status roles, not this component.

/// Maximum dialog outer width, in egui points.
pub const DIALOG_MAX_WIDTH: f32 = 440.0;
/// Safe horizontal viewport inset for a centered dialog, in egui points.
pub const DIALOG_VIEWPORT_INSET: f32 = 16.0;
/// Dialog frame padding, in egui points.
pub const DIALOG_PADDING: f32 = 20.0;
/// Gap between dialog actions, in egui points.
pub const DIALOG_ACTION_GAP: f32 = 8.0;
/// Dialog frame corner radius, in egui points.
pub const DIALOG_RADIUS: f32 = crate::tokens::RADIUS_DIALOG;
/// Dialog border width, in egui points.
pub const DIALOG_BORDER_WIDTH: f32 = crate::tokens::STROKE_THIN;
/// Dialog title font size, in egui points.
pub const DIALOG_TITLE_SIZE: f32 = crate::tokens::FONT_SIZE_BODY;
/// Dialog description font size, in egui points.
pub const DIALOG_DESCRIPTION_SIZE: f32 = 13.0;
/// Gap after the dialog title, in egui points.
pub const DIALOG_TITLE_GAP: f32 = 6.0;
/// Gap before dialog actions, in egui points.
pub const DIALOG_ACTIONS_GAP: f32 = 20.0;
// Dialog shadow uses `DbProTheme::floating_shadow`; the backdrop uses the
// `background.overlay` semantic role, so neither is component-owned.

#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
