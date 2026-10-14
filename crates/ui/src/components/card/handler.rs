use egui::{Pos2, Rect};
use lucide_icons::Icon;

use crate::DbProTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricTrendDirection {
    Up,
    Down,
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricTrendTone {
    Positive,
    Negative,
    Neutral,
    Warning,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricTrendVisual {
    pub color: egui::Color32,
    pub icon: Option<Icon>,
}

pub fn metric_trend_visual(
    direction: MetricTrendDirection,
    tone: MetricTrendTone,
    theme: &DbProTheme,
) -> MetricTrendVisual {
    let icon = match direction {
        MetricTrendDirection::Up => Some(Icon::TrendingUp),
        MetricTrendDirection::Down => Some(Icon::TrendingDown),
        MetricTrendDirection::Unspecified => None,
    };
    let color = match tone {
        MetricTrendTone::Positive => theme.success,
        MetricTrendTone::Negative => theme.danger,
        MetricTrendTone::Neutral => theme.text_secondary,
        MetricTrendTone::Warning => theme.warning,
    };

    MetricTrendVisual { color, icon }
}

pub fn footer_separator_segment(separator_rect: Rect) -> [Pos2; 2] {
    [
        Pos2::new(separator_rect.left(), separator_rect.center().y),
        Pos2::new(separator_rect.right(), separator_rect.center().y),
    ]
}

pub fn trend_label_text(text: &str) -> String {
    format!(" {text}")
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
