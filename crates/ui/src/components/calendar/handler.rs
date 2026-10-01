//! Pure calendar date calculations used by the interactive renderer.

/// Returns whether a year follows the Gregorian leap-year rule.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in a 1-based month, defaulting invalid months to 30 days.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

/// Computes the weekday where Sunday is zero, using the Sakamoto calendar algorithm.
pub fn previous_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

pub fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

/// Decides whether a date picker popup remains open after input handling.
///
/// Disabled pickers and Escape always close the popup without touching the date.
pub fn date_picker_popup_open(enabled: bool, is_open: bool, escape_pressed: bool) -> bool {
    enabled && is_open && !escape_pressed
}

/// Activates a focused custom-painted calendar control with Enter or Space.
pub fn should_activate_focused_control(focused: bool, enter_pressed: bool, space_pressed: bool) -> bool {
    focused && (enter_pressed || space_pressed)
}

pub fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    const MONTH_OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if !(1..=12).contains(&month) {
        return 0;
    }

    let adjusted_year = if month < 3 { year - 1 } else { year };
    let offset = MONTH_OFFSETS[(month - 1) as usize];
    (adjusted_year + adjusted_year / 4 - adjusted_year / 100 + adjusted_year / 400 + offset + day as i32).rem_euclid(7)
        as u32
}

#[cfg(test)]
mod tests {
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
}
