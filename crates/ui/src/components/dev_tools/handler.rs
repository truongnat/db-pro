use egui::{Color32, Pos2, Rect};

use super::config::{
    PROGRESS_RING_ARC_POINTS, PROGRESS_RING_DEFAULT_RADIUS, PROGRESS_RING_INSET, PROGRESS_RING_MIN_RADIUS,
    TERMINAL_DOT_SPACING, TERMINAL_DOT_START_X,
};
use crate::DbProTheme;

/// Fallback title used when no explicit title is supplied to TerminalBlock.
pub const DEFAULT_TERMINAL_TITLE: &str = "Terminal Output";

/// Returns the effective title for a terminal block.
pub fn terminal_title(custom_title: Option<&str>) -> &str {
    custom_title.unwrap_or(DEFAULT_TERMINAL_TITLE)
}

/// Computes the center positions for the 3 macOS-style control dots.
pub fn mac_dot_positions(header_rect: Rect) -> [Pos2; 3] {
    let y = header_rect.center().y;
    let base_x = header_rect.left() + TERMINAL_DOT_START_X;
    [
        Pos2::new(base_x, y),
        Pos2::new(base_x + TERMINAL_DOT_SPACING, y),
        Pos2::new(base_x + (TERMINAL_DOT_SPACING * 2.0), y),
    ]
}

/// Returns the themed colors for the 3 window control dots (danger, warning, success).
pub fn mac_dot_colors(theme: &DbProTheme) -> [Color32; 3] {
    [theme.danger, theme.warning, theme.success]
}

/// Clamps progress to the inclusive interval [0.0, 1.0].
pub fn clamp_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Normalizes arbitrary caller input to a finite, renderable ring radius.
pub fn normalize_ring_radius(radius: f32) -> f32 {
    if radius.is_finite() {
        radius.max(PROGRESS_RING_MIN_RADIUS)
    } else {
        PROGRESS_RING_DEFAULT_RADIUS
    }
}

/// Computes the sequence of vertices approximating the circular progress arc.
///
/// Starts at 12 o'clock (-π/2) and sweeps clockwise according to `progress` (0.0 to 1.0).
pub fn progress_ring_arc_points(center: Pos2, radius: f32, progress: f32) -> Vec<Pos2> {
    let effective_radius = (radius - PROGRESS_RING_INSET).max(0.0);
    let clamped = clamp_progress(progress);
    if clamped <= 0.0 {
        return Vec::new();
    }
    let sweep_angle = clamped * std::f32::consts::TAU;
    (0..=PROGRESS_RING_ARC_POINTS)
        .map(|i| {
            let frac = i as f32 / PROGRESS_RING_ARC_POINTS as f32;
            let angle = -std::f32::consts::FRAC_PI_2 + frac * sweep_angle;
            Pos2::new(
                center.x + effective_radius * angle.cos(),
                center.y + effective_radius * angle.sin(),
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
