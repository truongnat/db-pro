//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn navigation_wraps_and_ignores_empty_or_out_of_range_tracks() {
    let left = NavigationKeys {
        arrow_left: true,
        ..Default::default()
    };
    let right = NavigationKeys {
        arrow_right: true,
        ..Default::default()
    };
    assert_eq!(next_selection(None, Some(0), 3, left), Some(2));
    assert_eq!(next_selection(None, Some(2), 3, right), Some(0));
    assert_eq!(next_selection(None, Some(0), 0, right), None);
    assert_eq!(next_selection(None, Some(3), 3, right), None);
}

#[test]
fn keyboard_navigation_precedes_click_and_activation_keeps_focus() {
    let right = NavigationKeys {
        arrow_right: true,
        ..Default::default()
    };
    let activate = NavigationKeys {
        activate: true,
        ..Default::default()
    };
    assert_eq!(next_selection(Some(1), Some(0), 3, right), Some(1));
    assert_eq!(next_selection(None, Some(2), 3, activate), Some(2));
    assert_eq!(next_selection(Some(1), None, 3, right), Some(1));
}
