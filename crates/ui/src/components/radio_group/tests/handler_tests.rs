//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn radio_group_item_spacing_matches_orientation() {
    assert_eq!(
        radio_group_item_spacing(true),
        Vec2::new(HORIZONTAL_ITEM_GAP, NO_ITEM_GAP)
    );
    assert_eq!(
        radio_group_item_spacing(false),
        Vec2::new(NO_ITEM_GAP, VERTICAL_ITEM_GAP)
    );
}

#[test]
fn keyboard_navigation_wraps_and_skips_disabled_options() {
    let disabled = [false, true, false];
    assert_eq!(next_enabled_option_index(&disabled, 0, true), Some(2));
    assert_eq!(next_enabled_option_index(&disabled, 2, true), Some(0));
    assert_eq!(next_enabled_option_index(&disabled, 0, false), Some(2));
    assert_eq!(next_enabled_option_index(&[true, true], 0, true), None);
    assert_eq!(next_enabled_option_index(&[], 0, true), None);
    assert_eq!(next_enabled_option_index(&disabled, 3, true), None);
}

#[test]
fn handle_radio_option_click_only_triggers_when_unselected_and_enabled() {
    let val = 42;

    // Valid selection change
    assert_eq!(handle_radio_option_click(true, false, false, &val), Some(42));

    // Already selected - no change event
    assert_eq!(handle_radio_option_click(true, true, false, &val), None);

    // Disabled option clicked - no change event
    assert_eq!(handle_radio_option_click(true, false, true, &val), None);

    // Not clicked
    assert_eq!(handle_radio_option_click(false, false, false, &val), None);
}
