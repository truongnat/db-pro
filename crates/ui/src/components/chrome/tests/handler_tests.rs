//! Tests for `../handler.rs` (moved out of the source file).
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
