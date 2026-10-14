//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn clamp_page_handles_zero_and_overflow() {
    assert_eq!(clamp_page(0, 5), 1);
    assert_eq!(clamp_page(3, 5), 3);
    assert_eq!(clamp_page(10, 5), 5);
    assert_eq!(clamp_page(0, 0), 1);
}

#[test]
fn navigation_predicates_check_boundaries_and_enabled_flag() {
    assert!(!can_navigate_prev(true, 1));
    assert!(can_navigate_prev(true, 2));
    assert!(!can_navigate_prev(false, 2));

    assert!(can_navigate_next(true, 1, 5));
    assert!(!can_navigate_next(true, 5, 5));
    assert!(!can_navigate_next(false, 1, 5));
}

#[test]
fn prev_and_next_page_bounds() {
    assert_eq!(prev_page(1), 1);
    assert_eq!(prev_page(5), 4);

    assert_eq!(next_page(1, 5), 2);
    assert_eq!(next_page(5, 5), 5);
    assert_eq!(next_page(usize::MAX, usize::MAX), usize::MAX);
}

#[test]
fn format_page_label_renders_ratios() {
    assert_eq!(format_page_label(2, 8), "2 / 8");
    assert_eq!(format_page_label(1, 0), "1 / 1");
}

#[test]
fn pagination_buttons_have_directional_accessible_names() {
    assert_eq!(page_button_accessible_label(Icon::ChevronLeft), "Previous page");
    assert_eq!(page_button_accessible_label(Icon::ChevronRight), "Next page");
}

#[test]
fn breadcrumb_item_color_picks_primary_or_secondary() {
    let theme = DbProTheme::light();
    assert_eq!(breadcrumb_item_color(true, &theme), theme.text_primary);
    assert_eq!(breadcrumb_item_color(false, &theme), theme.text_secondary);
}

#[test]
fn page_button_colors_computes_palette() {
    let theme = DbProTheme::light();
    let (fill_enabled_hover, text_enabled_hover) = page_button_colors(true, true, &theme);
    assert_eq!(fill_enabled_hover, theme.surface_hover);
    assert_eq!(text_enabled_hover, theme.text_primary);

    let (fill_disabled, text_disabled) = page_button_colors(false, false, &theme);
    assert_eq!(fill_disabled, theme.surface_panel);
    assert_eq!(text_disabled, theme.text_disabled);
}
