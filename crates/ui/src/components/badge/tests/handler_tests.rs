//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn metrics_prioritize_dot_over_icon_for_leading_space() {
    let metrics = BadgeMetrics::from_compact(false);
    assert_eq!(metrics.leading_size(true, true), metrics.dot_size);
    assert_eq!(metrics.leading_size(false, true), metrics.icon_size);
    assert_eq!(metrics.leading_size(false, false), 0.0);
}

#[test]
fn calculate_size_adds_gap_only_when_leading_visual_exists() {
    let metrics = BadgeMetrics::from_compact(false);
    let text = Vec2::new(40.0, 10.0);
    assert_eq!(metrics.calculate_size(text, false, false).x, 56.0);
    assert_eq!(metrics.calculate_size(text, true, false).x, 68.0);
    assert_eq!(metrics.calculate_size(text, false, true).x, 74.0);
    assert_eq!(metrics.calculate_size(text, false, false).y, BADGE_MIN_HEIGHT);
}

#[test]
fn compact_metrics_use_smaller_density() {
    let regular = BadgeMetrics::from_compact(false);
    let compact = BadgeMetrics::from_compact(true);
    assert!(compact.font_size < regular.font_size);
    assert!(compact.min_height < regular.min_height);
    assert!(compact.gap < regular.gap);
}

#[test]
fn text_position_centers_measured_galley_on_vertical_center() {
    assert_eq!(text_position(12.0, 20.0, 8.0), Pos2::new(12.0, 16.0));
    assert_eq!(leading_gap(false, false, 6.0), 0.0);
    assert_eq!(leading_gap(true, false, 6.0), 6.0);
}
