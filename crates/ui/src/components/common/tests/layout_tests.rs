//! Tests for `../layout.rs` (moved out of the source file).
use super::*;

#[test]
fn calculate_dialog_layout_centers_correctly() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
    let layout = calculate_dialog_layout(screen, 420.0, Some(300.0), 16.0, 24.0, 220.0);

    assert_eq!(layout.width, 420.0);
    assert_eq!(layout.target_pos.x, (1280.0 - 420.0) / 2.0);
    assert_eq!(layout.target_pos.y, (800.0 - 300.0) / 2.0);
    assert_eq!(layout.max_body_height, 800.0 - 48.0 - 220.0);
}

#[test]
fn calculate_dialog_layout_handles_small_screens() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1024.0, 600.0));
    let layout = calculate_dialog_layout(screen, 880.0, Some(700.0), 16.0, 24.0, 220.0);

    assert_eq!(layout.width, 880.0);
    assert_eq!(layout.target_pos.y, 24.0);
    assert_eq!(layout.max_body_height, 600.0 - 48.0 - 220.0);
}

#[test]
fn dialog_body_cap_keeps_a_body_a_short_viewport_can_still_show() {
    assert_eq!(dialog_body_cap(1080.0, 24.0, 220.0), 1080.0 - 48.0 - 220.0);
    // Below the floor the card would have no usable body at all.
    assert_eq!(dialog_body_cap(300.0, 24.0, 220.0), 120.0);
}

#[test]
fn calculate_sheet_layout() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
    let (width, x) = super::calculate_sheet_layout(screen, 360.0, 1.0, 16.0, 16.0);
    assert_eq!(width, 360.0);
    assert_eq!(x, 1280.0 - 360.0);
}

#[test]
fn clamp_popup_to_screen() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0));
    let pos = super::clamp_popup_to_screen(Pos2::new(950.0, 780.0), Vec2::new(200.0, 100.0), screen, 10.0);
    assert!(pos.x + 200.0 <= 990.0);
    assert!(pos.y >= 10.0);
}
