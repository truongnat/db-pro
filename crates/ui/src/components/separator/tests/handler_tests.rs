//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn test_horizontal_size_respects_minimum_height() {
    let size = horizontal_size(200.0, 1.0, 2.0);
    // 1.0 + 2.0 * 2.0 = 5.0, clamped to MIN_HORIZONTAL_HEIGHT (14.0)
    assert_eq!(size, Vec2::new(200.0, 14.0));
}

#[test]
fn test_horizontal_size_scales_with_margins_and_thickness() {
    let size = horizontal_size(300.0, 2.0, 10.0);
    // 2.0 + 10.0 * 2.0 = 22.0 > 14.0
    assert_eq!(size, Vec2::new(300.0, 22.0));
}

#[test]
fn test_vertical_size_clamps_to_max_height() {
    let size = vertical_size(100.0, 1.0, 6.0);
    // height clamped to MAX_VERTICAL_HEIGHT (24.0), width = 1.0 + 6.0 * 2.0 = 13.0
    assert_eq!(size, Vec2::new(13.0, 24.0));

    let small_size = vertical_size(18.0, 1.0, 4.0);
    assert_eq!(small_size, Vec2::new(9.0, 18.0));
}

#[test]
fn test_horizontal_line_segment_spans_rect_width() {
    let rect = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(110.0, 40.0));
    let segment = horizontal_line_segment(rect, 30.0);
    assert_eq!(segment[0], Pos2::new(10.0, 30.0));
    assert_eq!(segment[1], Pos2::new(110.0, 30.0));
}

#[test]
fn test_vertical_line_segment_applies_insets() {
    let segment = vertical_line_segment(50.0, 10.0, 34.0);
    assert_eq!(segment[0], Pos2::new(50.0, 10.0 + VERTICAL_LINE_INSET));
    assert_eq!(segment[1], Pos2::new(50.0, 34.0 - VERTICAL_LINE_INSET));
}

#[test]
fn test_vertical_line_segment_never_reverses_for_short_or_reversed_bounds() {
    for height in [0.0, 1.0, 3.0, -1.0] {
        let segment = vertical_line_segment(50.0, 10.0, 10.0 + height);
        assert!(segment[0].y <= segment[1].y);
    }

    let reversed = vertical_line_segment(50.0, 34.0, 10.0);
    assert_eq!(reversed[0].y, 12.0);
    assert_eq!(reversed[1].y, 32.0);
}

#[test]
fn test_labeled_separator_geometry_centers_text_and_splits_lines() {
    let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 20.0));
    let center_y = 10.0;
    let text_size = Vec2::new(20.0, 12.0);
    let text_pad = 8.0;

    // available width = 100, text_w = 20, text_pad * 2 = 16 => remaining line width = (100 - 20 - 16) / 2 = 32
    let geom = calculate_labeled_separator_geometry(rect, center_y, text_size, text_pad);

    assert_eq!(geom.left_line[0], Pos2::new(0.0, 10.0));
    assert_eq!(geom.left_line[1], Pos2::new(32.0, 10.0));

    assert_eq!(geom.text_pos, Pos2::new(32.0 + 8.0, 10.0 - 6.0));

    assert_eq!(geom.right_line[0], Pos2::new(100.0 - 32.0, 10.0));
    assert_eq!(geom.right_line[1], Pos2::new(100.0, 10.0));
}

#[test]
fn test_labeled_separator_geometry_clips_lines_when_label_is_wide() {
    let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(30.0, 20.0));
    let geom = calculate_labeled_separator_geometry(rect, 10.0, Vec2::new(25.0, 10.0), 8.0);

    assert_eq!(geom.left_line[1].x - geom.left_line[0].x, 0.0);
    assert_eq!(geom.right_line[1].x - geom.right_line[0].x, 0.0);
    assert_eq!(geom.text_pos.x, 2.5);
}

#[test]
fn test_sizes_sanitize_negative_and_non_finite_dimensions() {
    assert_eq!(horizontal_size(100.0, -1.0, f32::NAN).y, MIN_HORIZONTAL_HEIGHT);
    assert_eq!(vertical_size(100.0, f32::INFINITY, -2.0).x, 0.0);
}
