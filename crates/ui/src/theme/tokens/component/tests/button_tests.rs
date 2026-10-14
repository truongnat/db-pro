//! Tests for `../button.rs` (moved out of the source file).
use super::*;

#[test]
fn disabled_outranks_loading() {
    assert!(!shows_loading(false, true));
    assert!(!shows_loading(false, false));
}

#[test]
fn loading_shows_only_for_enabled_buttons() {
    assert!(shows_loading(true, true));
    assert!(!shows_loading(true, false));
}

#[test]
fn interaction_state_follows_contract_precedence() {
    assert_eq!(
        resolve_interaction_state(false, true, true, true, true),
        InteractionState::Disabled
    );
    assert_eq!(
        resolve_interaction_state(true, true, true, true, true),
        InteractionState::Loading
    );
    assert_eq!(
        resolve_interaction_state(true, false, true, true, true),
        InteractionState::Active
    );
    assert_eq!(
        resolve_interaction_state(true, false, false, true, true),
        InteractionState::Focus
    );
    assert_eq!(
        resolve_interaction_state(true, false, false, false, true),
        InteractionState::Hover
    );
    assert_eq!(
        resolve_interaction_state(true, false, false, false, false),
        InteractionState::Default
    );
}

/// Heights stay on the control scale. Padding and label sizes stay on the
/// shared spacing and type tokens so a second 0.5pt scale cannot return.
#[test]
fn dimensions_stay_on_the_shared_scale() {
    assert_eq!(BUTTON_HEIGHT_SM, 28.0);
    assert_eq!(BUTTON_HEIGHT_DEFAULT, 32.0);
    assert_eq!(BUTTON_HEIGHT_LG, 38.0);
    assert_eq!(BUTTON_DEFAULT_PADDING, Vec2::new(SPACE_MD, SPACE_XS));
    assert_eq!(BUTTON_SM_PADDING, Vec2::new(SPACE_SM, SPACE_XS));
    assert_eq!(BUTTON_LG_PADDING, Vec2::new(SPACE_LG, SPACE_SM));
    assert_eq!(BUTTON_ICON_SM_PADDING, Vec2::new(SPACE_XS, SPACE_XS));
    assert_eq!(BUTTON_FONT_SIZE_SM, FONT_SIZE_CAPTION);
    assert_eq!(BUTTON_FONT_SIZE_DEFAULT, FONT_SIZE_UI_LABEL);
    assert_eq!(BUTTON_FONT_SIZE_LG, FONT_SIZE_BODY_SM);
    assert_eq!(BUTTON_ICON_SIZE_SM, Vec2::new(26.0, 26.0));
}
