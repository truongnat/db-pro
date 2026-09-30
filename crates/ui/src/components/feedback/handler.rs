use super::config::*;
use egui::{WidgetInfo, WidgetType};

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
mod tests {
    use super::*;

    #[test]
    fn test_normalize_progress_inputs() {
        assert_eq!(normalize_progress_fraction(f32::NAN), 0.0);
        assert_eq!(normalize_progress_fraction(f32::INFINITY), 0.0);
        assert_eq!(normalize_progress_fraction(-0.2), 0.0);
        assert_eq!(normalize_progress_fraction(1.2), 1.0);
        assert_eq!(normalize_progress_fraction(0.4), 0.4);

        assert_eq!(normalize_progress_height(f32::NAN), PROGRESS_DEFAULT_HEIGHT);
        assert_eq!(normalize_progress_height(f32::NEG_INFINITY), PROGRESS_DEFAULT_HEIGHT);
        assert_eq!(normalize_progress_height(-1.0), PROGRESS_DEFAULT_HEIGHT);
        assert_eq!(normalize_progress_height(0.0), 0.0);
        assert_eq!(normalize_progress_height(12.0), 12.0);
    }

    #[test]
    fn progress_indicator_info_exposes_label_and_percentage() {
        let info = progress_indicator_info(0.42, false, "Import progress", true);

        assert_eq!(info.typ, WidgetType::ProgressIndicator);
        assert_eq!(info.label.as_deref(), Some("Import progress"));
        assert!(matches!(info.value, Some(value) if (value - 42.0).abs() < 0.001));
        assert!(info.enabled);
    }

    #[test]
    fn progress_indicator_info_sanitizes_non_finite_and_out_of_range_values() {
        for (fraction, expected) in [
            (f32::NAN, 0.0),
            (f32::INFINITY, 0.0),
            (f32::NEG_INFINITY, 0.0),
            (-0.5, 0.0),
            (1.5, 100.0),
        ] {
            let info = progress_indicator_info(fraction, false, "Progress", true);
            assert_eq!(info.value, Some(expected));
        }
    }

    #[test]
    fn indeterminate_progress_and_spinner_omit_accessible_value() {
        let progress = progress_indicator_info(0.0, true, "Loading schema", true);
        let spinner = progress_indicator_info(0.0, true, "Loading", true);

        assert_eq!(progress.label.as_deref(), Some("Loading schema"));
        assert_eq!(progress.value, None);
        assert_eq!(spinner.label.as_deref(), Some("Loading"));
        assert_eq!(spinner.value, None);
    }

    #[test]
    fn test_calculate_separator_line_width() {
        let width = calculate_separator_line_width(100.0, 40.0);
        // (100 - 40 - 16) * 0.5 = 44 * 0.5 = 22.0
        assert_eq!(width, 22.0);

        let min_width = calculate_separator_line_width(60.0, 40.0);
        assert_eq!(min_width, SEPARATOR_MIN_LINE_WIDTH);
        assert_eq!(calculate_separator_line_width(40.0, 40.0), 0.0);
        assert_eq!(calculate_separator_line_width(f32::NAN, 40.0), 0.0);
    }

    #[test]
    fn test_calculate_spinner_radius() {
        assert_eq!(calculate_spinner_radius(18.0), 7.5);
        assert_eq!(calculate_spinner_radius(-1.0), SPINNER_MIN_RADIUS);
        assert_eq!(calculate_spinner_radius(f32::INFINITY), SPINNER_MIN_RADIUS);
    }

    #[test]
    fn test_calculate_beam_geometry() {
        let (start, width) = calculate_beam_geometry(10.0, 100.0, 0.2, 0.5);
        assert_eq!(start, 30.0);
        assert_eq!(width, 30.0);

        let (start_small, width_min) = calculate_beam_geometry(10.0, 100.0, 0.2, 0.21);
        assert_eq!(start_small, 30.0);
        assert_eq!(width_min, PROGRESS_MIN_BEAM_WIDTH);

        let (narrow_start, narrow_width) = calculate_beam_geometry(10.0, 5.0, -1.0, 2.0);
        assert_eq!((narrow_start, narrow_width), (10.0, 5.0));
        let (invalid_start, invalid_width) = calculate_beam_geometry(10.0, 100.0, f32::NAN, f32::INFINITY);
        assert_eq!((invalid_start, invalid_width), (10.0, 100.0));

        let (negative_width_start, negative_width) = calculate_beam_geometry(10.0, -1.0, 0.0, 1.0);
        assert_eq!((negative_width_start, negative_width), (10.0, 0.0));

        let (overflow_start, overflow_width) = calculate_beam_geometry(f32::MAX / 2.0, f32::MAX, 0.0, 1.0);
        assert_eq!((overflow_start, overflow_width), (f32::MAX / 2.0, 0.0));

        let (extreme_start, extreme_width) = calculate_beam_geometry(10.0, 100.0, f32::MAX, -f32::MAX);
        assert_eq!((extreme_start, extreme_width), (98.0, PROGRESS_MIN_BEAM_WIDTH));
    }
}
