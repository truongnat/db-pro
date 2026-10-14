//! Tests for `../shell.rs` (moved out of the source file).
use super::*;

#[test]
fn activity_rail_width_matches_scale_step() {
    // The rail is intentionally a spacing-scale value, not a free pixel pick.
    assert_eq!(ACTIVITY_RAIL_WIDTH, 48.0);
}

#[test]
fn settings_reserve_covers_button_and_margin() {
    // Reserve must exceed the IconSm hit box (26px) — otherwise the settings
    // button would clip or overlap the rail's bottom edge on short windows.
    assert!(ACTIVITY_RAIL_SETTINGS_RESERVE > 26.0);
}
