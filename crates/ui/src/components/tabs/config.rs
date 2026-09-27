//! Shared sizing and motion constants for tab tracks.

use crate::tokens::{RADIUS_MD, RADIUS_SM, SPACE_XS};

// Duration of the active tab transition, in seconds.
pub(super) const TAB_TRANSITION_SECS: f32 = 0.200;

// Segmented control item height, in egui points.
pub(super) const SEGMENTED_ITEM_HEIGHT: f32 = 28.0;
// Minimum segmented item width so short labels remain easy to target, in egui points.
pub(super) const SEGMENTED_ITEM_MIN_WIDTH: f32 = 48.0;
// Horizontal padding around segmented labels, in egui points.
pub(super) const SEGMENTED_LABEL_PAD_X: f32 = 24.0;
// Gap between adjacent segmented items, in egui points.
pub(super) const SEGMENTED_ITEM_GAP: f32 = 2.0;
// Inset between the segmented track and its selected pill, in egui points.
pub(super) const SEGMENTED_TRACK_PAD: f32 = 3.0;
// Shared theme radius for the segmented track.
pub(super) const SEGMENTED_TRACK_RADIUS: f32 = RADIUS_MD;
// Shared theme radius for the selected segmented pill.
pub(super) const SEGMENTED_PILL_RADIUS: f32 = RADIUS_SM;

// Underline tab hit-target height, in egui points.
pub(super) const UNDERLINE_ITEM_HEIGHT: f32 = 32.0;
// Horizontal padding around underline-tab labels, in egui points.
pub(super) const UNDERLINE_LABEL_PAD_X: f32 = 24.0;
// Shared theme spacing between underline tabs.
pub(super) const UNDERLINE_ITEM_GAP: f32 = SPACE_XS;
// Vertical inset of a hovered underline-tab background, in egui points.
pub(super) const UNDERLINE_HOVER_INSET_Y: f32 = 4.0;
// Corner radius of the hovered underline-tab background, in egui points.
pub(super) const UNDERLINE_HOVER_RADIUS: f32 = 5.0;
// Base divider thickness beneath the tab row, in egui points.
pub(super) const UNDERLINE_BASELINE_HEIGHT: f32 = 1.0;
// Selected-tab indicator thickness, in egui points.
pub(super) const UNDERLINE_INDICATOR_HEIGHT: f32 = 2.0;
// Horizontal inset that aligns the indicator with its label, in egui points.
pub(super) const UNDERLINE_INDICATOR_INSET_X: f32 = 4.0;
// Minimum selected-indicator width for very short labels, in egui points.
pub(super) const UNDERLINE_INDICATOR_MIN_WIDTH: f32 = 12.0;

// Font size shared by labels in the segmented control, in egui points.
pub(super) const SEGMENTED_FONT_SIZE: f32 = 12.5;
// Font size shared by labels in the underline tab row, in egui points.
pub(super) const UNDERLINE_FONT_SIZE: f32 = 13.0;
