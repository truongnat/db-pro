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
#[path = "tests/handler_tests.rs"]
mod tests;
