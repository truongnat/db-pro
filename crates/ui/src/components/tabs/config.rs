//! Shared sizing and motion constants for tab tracks.

use crate::tokens::{RADIUS_MD, RADIUS_SM, SPACE_XS};

pub(super) const TAB_TRANSITION_SECS: f32 = 0.200;

pub(super) const SEGMENTED_ITEM_HEIGHT: f32 = 28.0;
pub(super) const SEGMENTED_ITEM_MIN_WIDTH: f32 = 48.0;
pub(super) const SEGMENTED_LABEL_PAD_X: f32 = 24.0;
pub(super) const SEGMENTED_ITEM_GAP: f32 = 2.0;
pub(super) const SEGMENTED_TRACK_PAD: f32 = 3.0;
pub(super) const SEGMENTED_TRACK_RADIUS: f32 = RADIUS_MD;
pub(super) const SEGMENTED_PILL_RADIUS: f32 = RADIUS_SM;

pub(super) const UNDERLINE_ITEM_HEIGHT: f32 = 32.0;
pub(super) const UNDERLINE_LABEL_PAD_X: f32 = 24.0;
pub(super) const UNDERLINE_ITEM_GAP: f32 = SPACE_XS;
pub(super) const UNDERLINE_HOVER_INSET_Y: f32 = 4.0;
pub(super) const UNDERLINE_HOVER_RADIUS: f32 = 5.0;
pub(super) const UNDERLINE_BASELINE_HEIGHT: f32 = 1.0;
pub(super) const UNDERLINE_INDICATOR_HEIGHT: f32 = 2.0;
pub(super) const UNDERLINE_INDICATOR_INSET_X: f32 = 4.0;
pub(super) const UNDERLINE_INDICATOR_MIN_WIDTH: f32 = 12.0;

pub(super) const SEGMENTED_FONT_SIZE: f32 = 12.5;
pub(super) const UNDERLINE_FONT_SIZE: f32 = 13.0;
