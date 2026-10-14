//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn leap_year_and_month_lengths_follow_gregorian_rules() {
    assert!(is_leap_year(2000));
    assert!(!is_leap_year(1900));
    assert!(is_leap_year(2024));
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2025, 2), 28);
    assert_eq!(days_in_month(2025, 13), 30);
}

#[test]
fn month_transitions_wrap_years() {
    assert_eq!(previous_month(2026, 1), (2025, 12));
    assert_eq!(previous_month(2026, 6), (2026, 5));
    assert_eq!(next_month(2026, 12), (2027, 1));
    assert_eq!(next_month(2026, 6), (2026, 7));
}

#[test]
fn date_picker_popup_closes_when_disabled_or_escape_is_pressed() {
    assert!(date_picker_popup_open(true, true, false));
    assert!(!date_picker_popup_open(false, true, false));
    assert!(!date_picker_popup_open(true, true, true));
    assert!(!date_picker_popup_open(true, false, false));
}

#[test]
fn calendar_keyboard_activation_requires_focus() {
    assert!(should_activate_focused_control(true, true, false));
    assert!(should_activate_focused_control(true, false, true));
    assert!(!should_activate_focused_control(false, true, true));
    assert!(!should_activate_focused_control(true, false, false));
}

#[test]
fn weekday_matches_known_dates_and_negative_year_math() {
    assert_eq!(day_of_week(2026, 1, 1), 4);
    assert_eq!(day_of_week(2024, 2, 29), 4);
    assert_eq!(day_of_week(2026, 13, 1), 0);
    assert_eq!(day_of_week(-1, 1, 1), 6);
}
