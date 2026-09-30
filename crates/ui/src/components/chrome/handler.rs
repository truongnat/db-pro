use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Vec2};

use super::config::{
    AVATAR_ICON_FONT_SCALE, AVATAR_INITIALS_FONT_SCALE, AVATAR_INITIALS_MIN_FONT_SIZE, AVATAR_LG, AVATAR_MD,
    AVATAR_ROUNDED_RADIUS, AVATAR_SM, AVATAR_STATUS_DOT_INSET_FACTOR, AVATAR_STATUS_DOT_LG, AVATAR_STATUS_DOT_MD,
    AVATAR_STATUS_DOT_SM, AVATAR_STATUS_RING_PADDING, SKELETON_MAX_ALPHA, SKELETON_MIN_ALPHA,
    SKELETON_SHIMMER_CYCLES_PER_SECOND, SKELETON_SHIMMER_MIN_WIDTH, SKELETON_SHIMMER_WIDTH_FACTOR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
}

impl AvatarSize {
    pub(super) fn px(self) -> f32 {
        match self {
            Self::Sm => AVATAR_SM,
            Self::Md => AVATAR_MD,
            Self::Lg => AVATAR_LG,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarShape {
    Circle,
    Rounded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AvatarStatusDot {
    pub center: Pos2,
    pub radius: f32,
    pub ring_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SkeletonAlphaRange {
    pub min: f32,
    pub max: f32,
}

pub(super) fn avatar_rounding_radius(size: AvatarSize, shape: AvatarShape) -> f32 {
    match shape {
        AvatarShape::Circle => size.px() * 0.5,
        AvatarShape::Rounded => AVATAR_ROUNDED_RADIUS,
    }
}

pub(super) fn avatar_initials_font_size(size: AvatarSize) -> f32 {
    (size.px() * AVATAR_INITIALS_FONT_SCALE).max(AVATAR_INITIALS_MIN_FONT_SIZE)
}

pub(super) fn avatar_icon_font_size(size: AvatarSize) -> f32 {
    size.px() * AVATAR_ICON_FONT_SCALE
}

pub(super) fn avatar_status_color(status: AvatarStatus, theme: &DbProTheme) -> Color32 {
    match status {
        AvatarStatus::Online => theme.success,
        AvatarStatus::Busy => theme.danger,
        AvatarStatus::Away => theme.warning,
        AvatarStatus::Offline => theme.text_disabled,
    }
}

pub(super) fn avatar_status_dot(size: AvatarSize, avatar_rect: Rect) -> AvatarStatusDot {
    let radius = avatar_status_dot_radius(size);
    let center = Pos2::new(
        avatar_rect.right() - radius * AVATAR_STATUS_DOT_INSET_FACTOR,
        avatar_rect.bottom() - radius * AVATAR_STATUS_DOT_INSET_FACTOR,
    );

    AvatarStatusDot {
        center,
        radius,
        ring_radius: radius + AVATAR_STATUS_RING_PADDING,
    }
}

pub(super) fn skeleton_alpha_range() -> SkeletonAlphaRange {
    SkeletonAlphaRange {
        min: SKELETON_MIN_ALPHA,
        max: SKELETON_MAX_ALPHA,
    }
}

pub(super) fn resolve_skeleton_width(requested_width: f32, available_width: f32) -> f32 {
    let available_width = if available_width.is_finite() {
        available_width.max(0.0)
    } else {
        0.0
    };

    if !requested_width.is_finite() || requested_width <= 0.0 {
        return available_width;
    }

    requested_width.min(available_width)
}

// Invalid dimensions must not leak NaN or negative geometry into egui layout and painting.
pub(super) fn resolve_skeleton_height(requested_height: f32) -> f32 {
    if requested_height.is_finite() {
        requested_height.max(0.0)
    } else {
        super::config::SKELETON_DEFAULT_HEIGHT
    }
}

pub(super) fn resolve_skeleton_rounding(requested_rounding: f32) -> f32 {
    if requested_rounding.is_finite() {
        requested_rounding.max(0.0)
    } else {
        super::config::SKELETON_DEFAULT_ROUNDING
    }
}

pub(super) fn skeleton_shimmer_rect(container: Rect, height: f32, time_seconds: f64) -> Rect {
    let cycle = (time_seconds * SKELETON_SHIMMER_CYCLES_PER_SECOND).fract() as f32;
    let shimmer_x = container.left() + container.width() * cycle;
    let shimmer_width = (container.width() * SKELETON_SHIMMER_WIDTH_FACTOR).max(SKELETON_SHIMMER_MIN_WIDTH);

    Rect::from_min_size(
        Pos2::new(shimmer_x - shimmer_width * 0.5, container.top()),
        Vec2::new(shimmer_width, height),
    )
}

pub(super) fn visible_skeleton_shimmer_rect(container: Rect, height: f32, time_seconds: f64) -> Option<Rect> {
    let clipped = container.intersect(skeleton_shimmer_rect(container, height, time_seconds));
    clipped.is_positive().then_some(clipped)
}

fn avatar_status_dot_radius(size: AvatarSize) -> f32 {
    match size {
        AvatarSize::Sm => AVATAR_STATUS_DOT_SM,
        AvatarSize::Md => AVATAR_STATUS_DOT_MD,
        AvatarSize::Lg => AVATAR_STATUS_DOT_LG,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_metrics_preserve_existing_dimensions() {
        assert_eq!(AvatarSize::Sm.px(), 24.0);
        assert_eq!(AvatarSize::Md.px(), 32.0);
        assert_eq!(AvatarSize::Lg.px(), 40.0);
        assert_eq!(avatar_rounding_radius(AvatarSize::Md, AvatarShape::Circle), 16.0);
        assert_eq!(avatar_rounding_radius(AvatarSize::Md, AvatarShape::Rounded), 6.0);
        assert_eq!(avatar_initials_font_size(AvatarSize::Sm), 10.0);
        assert_eq!(avatar_icon_font_size(AvatarSize::Lg), 18.0);
    }

    #[test]
    fn avatar_status_dot_stays_anchored_to_bottom_right() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::splat(32.0));
        let dot = avatar_status_dot(AvatarSize::Md, rect);

        assert_eq!(dot.radius, 4.5);
        assert_eq!(dot.ring_radius, 6.0);
        assert_close(dot.center.x, 38.85);
        assert_close(dot.center.y, 48.85);
    }

    #[test]
    fn avatar_status_colors_use_semantic_theme_tokens() {
        let theme = DbProTheme::light();

        assert_eq!(avatar_status_color(AvatarStatus::Online, &theme), theme.success);
        assert_eq!(avatar_status_color(AvatarStatus::Busy, &theme), theme.danger);
        assert_eq!(avatar_status_color(AvatarStatus::Away, &theme), theme.warning);
        assert_eq!(avatar_status_color(AvatarStatus::Offline, &theme), theme.text_disabled);
    }

    #[test]
    fn skeleton_width_uses_available_width_only_for_fluid_requests() {
        assert_eq!(resolve_skeleton_width(120.0, 200.0), 120.0);
        assert_eq!(resolve_skeleton_width(240.0, 200.0), 200.0);
        assert_eq!(resolve_skeleton_width(0.0, 200.0), 200.0);
        assert_eq!(resolve_skeleton_width(-1.0, 200.0), 200.0);
        assert_eq!(resolve_skeleton_width(f32::NAN, 200.0), 200.0);
        assert_eq!(resolve_skeleton_width(120.0, f32::NAN), 0.0);
        assert_eq!(resolve_skeleton_width(120.0, f32::INFINITY), 0.0);
        assert_eq!(resolve_skeleton_width(120.0, -1.0), 0.0);
    }

    #[test]
    fn skeleton_height_rejects_negative_and_non_finite_values() {
        assert_eq!(resolve_skeleton_height(18.0), 18.0);
        assert_eq!(resolve_skeleton_height(-1.0), 0.0);
        assert_eq!(resolve_skeleton_height(f32::NAN), 12.0);
        assert_eq!(resolve_skeleton_height(f32::INFINITY), 12.0);
    }

    #[test]
    fn skeleton_rounding_rejects_negative_and_non_finite_values() {
        assert_eq!(resolve_skeleton_rounding(4.0), 4.0);
        assert_eq!(resolve_skeleton_rounding(-1.0), 0.0);
        assert_eq!(resolve_skeleton_rounding(f32::NAN), 6.0);
        assert_eq!(resolve_skeleton_rounding(f32::INFINITY), 6.0);
    }

    #[test]
    fn skeleton_alpha_range_preserves_pulse_bounds() {
        let alpha = skeleton_alpha_range();

        assert_eq!(alpha.min, 0.35);
        assert_eq!(alpha.max, 0.72);
    }

    #[test]
    fn skeleton_shimmer_geometry_preserves_band_motion_and_clipping() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 12.0));
        let raw = skeleton_shimmer_rect(rect, 12.0, 0.0);
        let visible = visible_skeleton_shimmer_rect(rect, 12.0, 0.0).expect("initial shimmer band should overlap");

        assert_close(raw.min.x, -5.0);
        assert_close(raw.min.y, 20.0);
        assert_close(raw.width(), 30.0);
        assert_close(raw.height(), 12.0);
        assert_close(visible.min.x, 10.0);
        assert_close(visible.min.y, 20.0);
        assert_close(visible.max.x, 25.0);
        assert_close(visible.max.y, 32.0);

        let advanced = skeleton_shimmer_rect(rect, 12.0, 0.625);
        assert_close(advanced.min.x, 45.0);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.000_1,
            "expected {actual} to equal {expected}"
        );
    }
}
