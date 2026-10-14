use egui::{Color32, Pos2, Rect, Vec2};

use super::config::{
    AVATAR_ICON_FONT_SCALE, AVATAR_INITIALS_FONT_SCALE, AVATAR_INITIALS_MIN_FONT_SIZE, AVATAR_LG, AVATAR_MD, AVATAR_SM,
    AVATAR_STATUS_DOT_INSET_FACTOR, AVATAR_STATUS_DOT_LG, AVATAR_STATUS_DOT_MD, AVATAR_STATUS_DOT_SM,
    AVATAR_STATUS_RING_PADDING, SKELETON_MAX_ALPHA, SKELETON_MIN_ALPHA, SKELETON_SHIMMER_CYCLES_PER_SECOND,
    SKELETON_SHIMMER_MIN_WIDTH, SKELETON_SHIMMER_WIDTH_FACTOR,
};
use crate::{tokens::RADIUS_SM, DbProTheme};

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
        AvatarShape::Rounded => RADIUS_SM,
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
        RADIUS_SM
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
#[path = "tests/handler_tests.rs"]
mod tests;
