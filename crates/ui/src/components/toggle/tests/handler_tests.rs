//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn size_tokens_preserve_public_dimensions() {
    assert_eq!(ToggleSize::Sm.height(), config::SM_HEIGHT);
    assert_eq!(ToggleSize::Default.font_size(), config::DEFAULT_FONT_SIZE);
    assert_eq!(ToggleSize::Lg.icon_size(), config::LG_ICON_SIZE);
    assert_eq!(ToggleSize::Default.padding_x(), config::DEFAULT_PADDING_X);
}

#[test]
fn content_width_accounts_for_icon_label_gap_only_when_needed() {
    assert_eq!(content_width(false, None, 14.0), 0.0);
    assert_eq!(content_width(true, None, 14.0), 14.0);
    assert_eq!(
        content_width(true, Some(30.0), 14.0),
        14.0 + config::ICON_TEXT_GAP + 30.0
    );
}

#[test]
fn control_width_keeps_the_size_minimum() {
    assert_eq!(control_width(4.0, 11.0, 32.0), 32.0);
    assert_eq!(control_width(30.0, 11.0, 32.0), 52.0);
}

#[test]
fn selection_and_pressed_transitions_are_stable() {
    assert!(toggled_state(false));
    assert!(!toggled_state(true));

    let mut pressed = false;
    assert!(apply_toggle_click(&mut pressed, true, true));
    assert!(pressed);
    assert!(!apply_toggle_click(&mut pressed, false, true));
    assert!(!apply_toggle_click(&mut pressed, true, false));

    assert_eq!(selection_after_click(&1, &1), None);
    assert_eq!(selection_after_click(&1, &2), Some(2));

    let mut selected = 1;
    assert_eq!(apply_selection_click(&mut selected, &1, true), None);
    assert_eq!(apply_selection_click(&mut selected, &2, true), Some(2));
    assert_eq!(selected, 2);
    assert_eq!(apply_selection_click(&mut selected, &3, false), None);
}

#[test]
fn group_rounding_only_rounds_the_outer_edges() {
    assert_eq!(group_rounding(0, 1), CornerRadius::same(config::TOGGLE_ROUNDING as u8));
    assert_eq!(group_rounding(0, 3).ne, 0);
    assert_eq!(group_rounding(1, 3), CornerRadius::ZERO);
    assert_eq!(group_rounding(2, 3).se, config::TOGGLE_ROUNDING as u8);
}

#[test]
fn appearance_preserves_outline_and_disabled_states() {
    let theme = DbProTheme::dark();
    let outline = standalone_appearance(ToggleVariant::Outline, false, true, 0.0, theme);
    assert_eq!(
        outline.stroke,
        Stroke::new(config::OUTLINE_STROKE_WIDTH, theme.border_default)
    );
    let disabled = standalone_appearance(ToggleVariant::Default, false, false, 0.0, theme);
    assert_eq!(disabled.text_color, theme.text_disabled);
}
