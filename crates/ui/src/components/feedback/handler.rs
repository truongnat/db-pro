use egui::{WidgetInfo, WidgetType};

use super::config::*;

pub(super) fn normalize_progress_fraction(fraction: f32) -> f32 {
    // `f32::clamp` preserves NaN, which would otherwise propagate into animation and paint geometry.
    if !fraction.is_finite() {
        return 0.0;
    }
    fraction.clamp(0.0, 1.0)
}

pub(super) fn normalize_progress_height(height: f32) -> f32 {
    // Keep invalid dimensions out of egui allocation while preserving explicit zero-height bars.
    if !height.is_finite() || height < 0.0 {
        return PROGRESS_DEFAULT_HEIGHT;
    }
    height
}

/// Builds the semantic payload shared by determinate, indeterminate, and spinner indicators.
pub(super) fn progress_indicator_info(fraction: f32, indeterminate: bool, label: &str, enabled: bool) -> WidgetInfo {
    let mut info = WidgetInfo::labeled(WidgetType::ProgressIndicator, enabled, label);
    if !indeterminate {
        // egui exposes progress indicator values on a 0–100 scale; sanitize even if called outside the UI path.
        info.value = Some(normalize_progress_fraction(fraction) as f64 * 100.0);
    }
    info
}

pub fn calculate_separator_line_width(total_width: f32, text_width: f32) -> f32 {
    let available = total_width - text_width - SEPARATOR_PADDING_EXTRA;
    if !available.is_finite() || available <= 0.0 {
        return 0.0;
    }
    (available * 0.5).max(SEPARATOR_MIN_LINE_WIDTH)
}

pub fn calculate_spinner_radius(size: f32) -> f32 {
    if !size.is_finite() {
        return SPINNER_MIN_RADIUS;
    }
    ((size - SPINNER_RADIUS_OFFSET) * 0.5).max(SPINNER_MIN_RADIUS)
}

pub fn calculate_beam_geometry(rect_left: f32, rect_width: f32, tail: f32, head: f32) -> (f32, f32) {
    if !rect_left.is_finite() || !rect_width.is_finite() || rect_width <= 0.0 {
        return (rect_left, 0.0);
    }

    let right = rect_left + rect_width;
    if !right.is_finite() {
        // Finite inputs can still overflow when combined; return an empty finite beam for egui painting.
        return (rect_left, 0.0);
    }
    let tail = if tail.is_finite() { tail } else { 0.0 };
    let head = if head.is_finite() { head } else { 1.0 };
    let start = (rect_left + rect_width * tail).clamp(rect_left, right);
    let end = (rect_left + rect_width * head).clamp(rect_left, right);
    let desired_width = (end - start).max(PROGRESS_MIN_BEAM_WIDTH);
    let beam_width = desired_width.min(rect_width);
    let beam_start = start.min(right - beam_width).max(rect_left);
    (beam_start, beam_width)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
