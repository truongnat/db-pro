//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn header_click_toggles_state_when_enabled() {
    let mut open = false;
    apply_header_click(&mut open, true, false);
    assert!(open);

    apply_header_click(&mut open, true, false);
    assert!(!open);
}

#[test]
fn header_click_ignored_when_disabled_or_not_clicked() {
    let mut open = false;
    apply_header_click(&mut open, true, true);
    assert!(!open, "Click must be ignored when disabled");

    apply_header_click(&mut open, false, false);
    assert!(!open, "No toggle when not clicked");
}

#[test]
fn calculate_badge_rect_matches_expected_dimensions() {
    let header = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 36.0));
    let badge_rect = calculate_badge_rect(header, 18.0, 30.0);

    assert_eq!(badge_rect.height(), BADGE_HEIGHT);
    assert_eq!(badge_rect.width(), 40.0);
    assert_eq!(badge_rect.center().y, 18.0);
    // badge right edge should be at header.right() - BADGE_RIGHT_MARGIN
    assert_eq!(badge_rect.right(), 200.0 - BADGE_RIGHT_MARGIN);
}

#[test]
fn badge_rect_stays_inside_narrow_header() {
    let header = Rect::from_min_size(Pos2::new(10.0, 0.0), Vec2::new(12.0, 36.0));
    let badge_rect = calculate_badge_rect(header, 18.0, 40.0);
    assert!(badge_rect.left() >= header.left());
    assert!(badge_rect.right() <= header.right());
}
