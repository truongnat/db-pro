//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn copy_button_state_switches_to_success_when_recently_copied() {
    let theme = DbProTheme::light();
    let idle = copy_button_state(false, &theme);
    assert_eq!(char::from(idle.icon), char::from(Icon::Copy));
    assert_eq!(idle.label, "Copy");
    assert_eq!(idle.color, theme.text_secondary);

    let copied = copy_button_state(true, &theme);
    assert_eq!(char::from(copied.icon), char::from(Icon::Check));
    assert_eq!(copied.label, "Copied");
    assert_eq!(copied.color, theme.success);
}

#[test]
fn is_copy_active_checks_time_interval() {
    assert!(is_copy_active(Some(10.0), 11.5, 2.0));
    assert!(!is_copy_active(Some(10.0), 12.5, 2.0));
    assert!(!is_copy_active(None, 10.0, 2.0));
    assert!(!is_copy_active(Some(12.0), 10.0, 2.0));
}

#[test]
fn copy_feedback_remaining_expires_and_ignores_future_time() {
    assert_eq!(copy_feedback_remaining(Some(10.0), 11.25, 2.0), Some(0.75));
    assert_eq!(copy_feedback_remaining(Some(10.0), 12.0, 2.0), None);
    assert_eq!(copy_feedback_remaining(Some(12.0), 10.0, 2.0), None);
    assert_eq!(copy_feedback_remaining(None, 10.0, 2.0), None);
}

#[test]
fn code_body_content_width_reserves_long_lines_and_min_width() {
    assert_eq!(code_body_content_width([20.0, 40.0], 28.0, 120.0), 120.0);
    assert_eq!(code_body_content_width([200.0], 28.0, 120.0), 244.0);
}

#[test]
fn copy_button_rect_stays_inside_narrow_header() {
    let header = Rect::from_min_size(Pos2::ZERO, Vec2::new(24.0, 18.0));
    let copy = copy_button_rect(header, Vec2::new(68.0, 22.0));
    assert!(header.contains_rect(copy));
}

#[test]
fn header_language_clip_stays_left_of_copy_button() {
    let header = Rect::from_min_size(Pos2::ZERO, Vec2::new(96.0, 32.0));
    let copy = copy_button_rect(header, Vec2::new(68.0, 22.0));
    let clip = header_language_clip_rect(header, copy);
    assert!(clip.right() <= copy.left());
    assert_eq!(clip.left(), header.left());
    assert_eq!(clip.bottom(), header.bottom());
}

#[test]
fn calculate_gutter_width_accounts_for_digit_count() {
    assert_eq!(calculate_gutter_width(false, 100), CODE_BLOCK_NO_GUTTER_WIDTH);
    // 1-digit line numbers (1..9): 1 * 8.0 + 20.0 = 28.0
    assert_eq!(calculate_gutter_width(true, 5), 28.0);
    // 3-digit line numbers (1..100): 3 * 8.0 + 20.0 = 44.0
    assert_eq!(calculate_gutter_width(true, 100), 44.0);
}

#[test]
fn inline_code_geometry_helpers() {
    let galley_size = Vec2::new(50.0, 14.0);
    let size = inline_code_size(galley_size);
    assert_eq!(size, Vec2::new(62.0, 18.0));

    let text_pos = inline_code_text_pos(Pos2::new(10.0, 20.0));
    assert_eq!(text_pos, Pos2::new(16.0, 22.0));
}

#[test]
fn format_line_number_aligns_properly() {
    assert_eq!(format_line_number(1, 9), "1");
    assert_eq!(format_line_number(1, 100), "  1");
    assert_eq!(format_line_number(42, 100), " 42");
    assert_eq!(format_line_number(100, 100), "100");
}
