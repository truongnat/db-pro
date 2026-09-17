//! Active-indicator motion for tab tracks.
//!
//! Animates in **track-local** coordinates so a parent panel shift (sidebar
//! resize) does not look like a selection change.

use super::config::{
    TAB_TRANSITION_SECS, UNDERLINE_INDICATOR_HEIGHT, UNDERLINE_INDICATOR_INSET_X, UNDERLINE_INDICATOR_MIN_WIDTH,
};
use egui::{Rect, Vec2};

pub(super) struct TabTrackerAnimation;

impl TabTrackerAnimation {
    pub fn animate_indicator(ctx: &egui::Context, track_id: egui::Id, track_origin_x: f32, target: Rect) -> (f32, f32) {
        let target_rel_x = target.left() - track_origin_x;
        // Distinct from the old absolute `x` key so upgrades do not lerp from a
        // stale screen coordinate into a relative offset.
        let rel_x = ctx.animate_value_with_time(track_id.with("rel_x"), target_rel_x, TAB_TRANSITION_SECS);
        let w = ctx.animate_value_with_time(track_id.with("w"), target.width(), TAB_TRANSITION_SECS);
        let x = track_origin_x + rel_x;

        if (rel_x - target_rel_x).abs() > 0.5 || (w - target.width()).abs() > 0.5 {
            ctx.request_repaint();
        }
        (x, w)
    }

    pub fn animate_pill(ctx: &egui::Context, track_id: egui::Id, track_origin_x: f32, target: Rect) -> Rect {
        let (x, w) = Self::animate_indicator(ctx, track_id, track_origin_x, target);
        Rect::from_min_size(egui::pos2(x, target.top()), Vec2::new(w, target.height()))
    }

    pub fn animate_underline(ctx: &egui::Context, track_id: egui::Id, track_origin_x: f32, target: Rect) -> Rect {
        let (x, w) = Self::animate_indicator(ctx, track_id, track_origin_x, target);
        Rect::from_min_size(
            egui::pos2(
                x + UNDERLINE_INDICATOR_INSET_X,
                target.bottom() - UNDERLINE_INDICATOR_HEIGHT,
            ),
            Vec2::new(
                (w - UNDERLINE_INDICATOR_INSET_X * 2.0).max(UNDERLINE_INDICATOR_MIN_WIDTH),
                UNDERLINE_INDICATOR_HEIGHT,
            ),
        )
    }
}
