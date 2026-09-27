use crate::tokens::STROKE_THIN;
use crate::DbProTheme;
use egui::{Color32, Pos2, Stroke, Vec2};

use super::config::{
    BADGE_DOT, BADGE_FONT_SIZE, BADGE_GAP, BADGE_ICON_SIZE, BADGE_MIN_HEIGHT, BADGE_PAD_X, BADGE_PAD_Y,
    COMPACT_BADGE_DOT, COMPACT_BADGE_FONT_SIZE, COMPACT_BADGE_GAP, COMPACT_BADGE_ICON_SIZE, COMPACT_BADGE_MIN_HEIGHT,
    COMPACT_BADGE_PAD_X, COMPACT_BADGE_PAD_Y,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
    Destructive,
    Success,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgePalette {
    pub fill: Color32,
    pub text_color: Color32,
    pub border_stroke: Stroke,
    pub dot_color: Color32,
}

impl BadgePalette {
    pub fn from_variant(variant: BadgeVariant, theme: &DbProTheme) -> Self {
        match variant {
            BadgeVariant::Default => Self {
                fill: theme.accent_soft,
                text_color: theme.accent,
                border_stroke: Stroke::new(STROKE_THIN, theme.accent),
                dot_color: theme.accent,
            },
            BadgeVariant::Secondary => Self {
                fill: theme.surface_elevated,
                text_color: theme.text_secondary,
                border_stroke: Stroke::new(STROKE_THIN, theme.border_default),
                dot_color: theme.text_tertiary,
            },
            BadgeVariant::Outline => Self {
                fill: Color32::TRANSPARENT,
                text_color: theme.text_primary,
                border_stroke: Stroke::new(STROKE_THIN, theme.border_default),
                dot_color: theme.text_tertiary,
            },
            BadgeVariant::Destructive => Self {
                fill: theme.danger_soft(),
                text_color: theme.danger,
                border_stroke: Stroke::new(STROKE_THIN, theme.danger.linear_multiply(0.5)),
                dot_color: theme.danger,
            },
            BadgeVariant::Success => Self {
                fill: theme.success_soft(),
                text_color: theme.success,
                border_stroke: Stroke::new(STROKE_THIN, theme.success.linear_multiply(0.4)),
                dot_color: theme.success,
            },
            BadgeVariant::Warning => Self {
                fill: theme.warning_soft(),
                text_color: theme.warning,
                border_stroke: Stroke::new(STROKE_THIN, theme.warning.linear_multiply(0.5)),
                dot_color: theme.warning,
            },
            BadgeVariant::Info => Self {
                fill: theme.info_soft(),
                text_color: theme.info,
                border_stroke: Stroke::new(STROKE_THIN, theme.info.linear_multiply(0.5)),
                dot_color: theme.info,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeMetrics {
    pub font_size: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub min_height: f32,
    pub dot_size: f32,
    pub icon_size: f32,
    pub gap: f32,
}

impl BadgeMetrics {
    pub fn from_compact(compact: bool) -> Self {
        if compact {
            Self {
                font_size: COMPACT_BADGE_FONT_SIZE,
                pad_x: COMPACT_BADGE_PAD_X,
                pad_y: COMPACT_BADGE_PAD_Y,
                min_height: COMPACT_BADGE_MIN_HEIGHT,
                dot_size: COMPACT_BADGE_DOT,
                icon_size: COMPACT_BADGE_ICON_SIZE,
                gap: COMPACT_BADGE_GAP,
            }
        } else {
            Self {
                font_size: BADGE_FONT_SIZE,
                pad_x: BADGE_PAD_X,
                pad_y: BADGE_PAD_Y,
                min_height: BADGE_MIN_HEIGHT,
                dot_size: BADGE_DOT,
                icon_size: BADGE_ICON_SIZE,
                gap: BADGE_GAP,
            }
        }
    }

    pub fn leading_size(&self, has_dot: bool, has_icon: bool) -> f32 {
        if has_dot {
            self.dot_size
        } else if has_icon {
            self.icon_size
        } else {
            0.0
        }
    }

    pub fn calculate_size(&self, text_size: Vec2, has_dot: bool, has_icon: bool) -> Vec2 {
        let leading = self.leading_size(has_dot, has_icon);
        let gap = if leading > 0.0 { self.gap } else { 0.0 };
        let width = self.pad_x * 2.0 + leading + gap + text_size.x;
        let height = (text_size.y + self.pad_y * 2.0).max(self.min_height);
        Vec2::new(width, height)
    }
}

pub fn leading_gap(has_dot: bool, has_icon: bool, gap: f32) -> f32 {
    if has_dot || has_icon {
        gap
    } else {
        0.0
    }
}

pub fn text_position(cursor_x: f32, vertical_center: f32, text_height: f32) -> Pos2 {
    Pos2::new(cursor_x, vertical_center - text_height * 0.5)
}

#[cfg(test)]
mod tests {
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
}
