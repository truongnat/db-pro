//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn metric_trend_visual_maps_each_direction_and_tone_independently() {
    let theme = DbProTheme::light();
    let directions = [
        (MetricTrendDirection::Up, Some(Icon::TrendingUp)),
        (MetricTrendDirection::Down, Some(Icon::TrendingDown)),
        (MetricTrendDirection::Unspecified, None),
    ];
    let tones = [
        (MetricTrendTone::Positive, theme.success),
        (MetricTrendTone::Negative, theme.danger),
        (MetricTrendTone::Neutral, theme.text_secondary),
        (MetricTrendTone::Warning, theme.warning),
    ];

    for (direction, expected_icon) in directions {
        for (tone, expected_color) in tones {
            let visual = metric_trend_visual(direction, tone, &theme);
            assert_eq!(visual.icon.map(char::from), expected_icon.map(char::from));
            assert_eq!(visual.color, expected_color);
        }
    }
}

#[test]
fn footer_separator_segment_uses_rect_edges_and_vertical_center() {
    let rect = Rect::from_min_max(Pos2::new(8.0, 4.0), Pos2::new(108.0, 6.0));

    assert_eq!(
        footer_separator_segment(rect),
        [Pos2::new(8.0, 5.0), Pos2::new(108.0, 5.0)]
    );
}

#[test]
fn trend_label_text_keeps_icon_gap_prefix() {
    assert_eq!(trend_label_text("+12%"), " +12%");
}
