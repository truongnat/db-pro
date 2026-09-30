use crate::DbProTheme;
use egui::{Color32, Pos2, Rect};

use super::config::{
    PROGRESS_RING_ARC_POINTS, PROGRESS_RING_DEFAULT_RADIUS, PROGRESS_RING_INSET, PROGRESS_RING_MIN_RADIUS,
    TERMINAL_DOT_SPACING, TERMINAL_DOT_START_X,
};

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
mod tests {
    use super::*;

    #[test]
    fn terminal_title_uses_fallback_when_none() {
        assert_eq!(terminal_title(None), "Terminal Output");
        assert_eq!(terminal_title(Some("psql session")), "psql session");
    }

    #[test]
    fn mac_dot_positions_computes_evenly_spaced_dots() {
        let rect = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(200.0, 48.0));
        let dots = mac_dot_positions(rect);
        let center_y = rect.center().y;

        assert_eq!(dots[0], Pos2::new(22.0, center_y));
        assert_eq!(dots[1], Pos2::new(34.0, center_y));
        assert_eq!(dots[2], Pos2::new(46.0, center_y));
    }

    #[test]
    fn mac_dot_colors_matches_semantic_theme_tokens() {
        let theme = DbProTheme::light();
        let colors = mac_dot_colors(&theme);
        assert_eq!(colors[0], theme.danger);
        assert_eq!(colors[1], theme.warning);
        assert_eq!(colors[2], theme.success);
    }

    #[test]
    fn clamp_progress_bounds_values() {
        assert_eq!(clamp_progress(-0.5), 0.0);
        assert_eq!(clamp_progress(0.0), 0.0);
        assert_eq!(clamp_progress(0.75), 0.75);
        assert_eq!(clamp_progress(1.0), 1.0);
        assert_eq!(clamp_progress(1.5), 1.0);
    }

    #[test]
    fn progress_values_and_radii_reject_non_finite_inputs() {
        assert_eq!(clamp_progress(f32::NAN), 0.0);
        assert_eq!(clamp_progress(f32::INFINITY), 0.0);
        assert_eq!(normalize_ring_radius(-5.0), PROGRESS_RING_MIN_RADIUS);
        assert_eq!(normalize_ring_radius(f32::INFINITY), PROGRESS_RING_DEFAULT_RADIUS);
    }

    #[test]
    fn progress_ring_arc_points_produces_smooth_arc() {
        let center = Pos2::new(50.0, 50.0);
        let radius = 20.0;

        let empty = progress_ring_arc_points(center, radius, 0.0);
        assert!(empty.is_empty());

        let half = progress_ring_arc_points(center, radius, 0.5);
        assert_eq!(half.len(), PROGRESS_RING_ARC_POINTS + 1);
        // Start point at 12 o'clock (top): angle = -PI/2 -> cos=0, sin=-1 -> (50.0, 50.0 - 18.0) = (50.0, 32.0)
        assert!((half[0].x - 50.0).abs() < 1e-4);
        assert!((half[0].y - 32.0).abs() < 1e-4);

        // End point at 6 o'clock (bottom): angle = -PI/2 + PI = PI/2 -> cos=0, sin=1 -> (50.0, 68.0)
        let last = half.last().unwrap();
        assert!((last.x - 50.0).abs() < 1e-4);
        assert!((last.y - 68.0).abs() < 1e-4);
    }
}
