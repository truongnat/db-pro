use egui::Vec2;

use super::config::{FALLBACK_RATIO, MIN_RATIO};

/// Returns the caller-provided ratio when it is usable, otherwise falls back to a square.
pub fn sanitize_ratio(ratio: f32) -> f32 {
    if !ratio.is_finite() || ratio <= MIN_RATIO {
        FALLBACK_RATIO
    } else {
        ratio
    }
}

/// Calculates the rectangle size that preserves the configured aspect ratio.
pub fn aspect_size(available_width: f32, ratio: f32) -> Vec2 {
    Vec2::new(available_width, available_width / sanitize_ratio(ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_ratio_falls_back_for_tiny_or_negative_values() {
        assert_eq!(sanitize_ratio(0.0), FALLBACK_RATIO);
        assert_eq!(sanitize_ratio(-2.0), FALLBACK_RATIO);
        assert_eq!(sanitize_ratio(MIN_RATIO), FALLBACK_RATIO);
        assert_eq!(sanitize_ratio(f32::NAN), FALLBACK_RATIO);
        assert_eq!(sanitize_ratio(f32::INFINITY), FALLBACK_RATIO);
        assert_eq!(sanitize_ratio(MIN_RATIO + f32::EPSILON), MIN_RATIO + f32::EPSILON);
    }

    #[test]
    fn aspect_size_preserves_width_and_ratio() {
        let size = aspect_size(160.0, 16.0 / 9.0);
        assert_eq!(size.x, 160.0);
        assert!((size.y - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn aspect_size_uses_square_fallback_for_invalid_ratio() {
        let size = aspect_size(42.0, 0.0);
        assert_eq!(size, Vec2::new(42.0, 42.0));
    }
}
