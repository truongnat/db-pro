//! Tests for `../handler.rs` (moved out of the source file).
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
fn reduced_motion_beam_uses_a_static_center_segment() {
    assert_eq!(
        calculate_beam_geometry(10.0, 100.0, PROGRESS_STATIC_BEAM_TAIL, PROGRESS_STATIC_BEAM_HEAD),
        (35.0, 50.0)
    );
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
