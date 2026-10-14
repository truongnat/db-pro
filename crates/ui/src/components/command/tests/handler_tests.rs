//! Tests for `../handler.rs` (moved out of the source file).
use egui::{Pos2, Rect, Vec2};

use super::*;

#[test]
fn disabled_items_cannot_trigger_hover_highlight_or_actions() {
    assert!(!item_is_actionable(true));
    assert!(!item_highlight_target(true, true, true));
    assert!(!item_highlight_target(false, false, true));
}

#[test]
fn enabled_selection_and_hover_drive_the_highlight_target() {
    assert!(item_is_actionable(false));
    assert!(item_highlight_target(false, true, false));
    assert!(item_highlight_target(true, false, false));
    assert!(!item_highlight_target(false, false, false));
}

#[test]
fn selected_rows_keep_the_background_and_hover_uses_a_threshold() {
    assert!(item_has_background(true, false, 0.0));
    assert!(!item_has_background(false, false, 0.001));
    assert!(item_has_background(false, false, 0.002));
    assert!(!item_has_background(true, true, 0.0));
}

#[test]
fn title_and_icon_colors_match_disabled_and_highlight_states() {
    let theme = DbProTheme::light();

    assert_eq!(item_title_color(&theme, true, true), theme.text_disabled);
    assert_eq!(item_title_color(&theme, false, true), theme.text_primary);
    assert_eq!(
        item_title_color(&theme, false, false),
        theme.text_primary.linear_multiply(ITEM_DEFAULT_TITLE_ALPHA)
    );
    assert_eq!(item_icon_color(&theme, true, true), theme.text_disabled);
    assert_eq!(item_icon_color(&theme, false, true), theme.text_primary);
    assert_eq!(item_icon_color(&theme, false, false), theme.text_secondary);
}

#[test]
fn input_and_shortcut_geometry_preserve_the_existing_insets() {
    let input = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(300.0, 44.0));
    let icon_x = command_input_icon_x(input);
    let text = command_input_text_rect(input, icon_x);
    let row = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 36.0));
    let shortcut = shortcut_rect(row);
    let text_clip = command_text_clip_rect(row, 10.0, true);
    let narrow_row = Rect::from_min_size(Pos2::ZERO, Vec2::new(24.0, 36.0));
    let narrow_clip = command_text_clip_rect(narrow_row, 20.0, true);

    assert_eq!(icon_x, 24.0);
    assert_eq!(text.min, Pos2::new(48.0, 28.0));
    assert_eq!(text.max, Pos2::new(278.0, 56.0));
    assert_eq!(shortcut.min, Pos2::new(220.0, 6.0));
    assert_eq!(shortcut.max, Pos2::new(292.0, 30.0));
    assert_eq!(text_clip.max.x, 212.0);
    assert_eq!(narrow_clip.width(), 0.0);
    assert_eq!(subtitle_x(30.0, 80.0, 8.0), 118.0);
    assert_eq!(
        accessible_item_label("Open query", Some("Recent"), Some("⌘K")),
        "Open query, Recent, shortcut ⌘K"
    );
}
