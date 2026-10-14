//! Tests for `../input.rs` (moved out of the source file).
use super::*;

fn field_state(enabled: bool, has_error: bool, focused: bool, hovered: bool, bordered: bool) -> FieldChromeState {
    FieldChromeState {
        enabled,
        has_error,
        focused,
        hovered,
        bordered,
    }
}

#[test]
fn disabled_outranks_error_focus_and_hover() {
    let state = field_state(false, true, true, true, false);
    assert_eq!(resolve_chrome(state), FieldChrome::Disabled);
}

#[test]
fn error_outranks_focus_and_hover_but_not_disabled() {
    assert_eq!(
        resolve_chrome(field_state(true, true, true, true, false)),
        FieldChrome::Error
    );
    assert_eq!(
        resolve_chrome(field_state(true, true, false, true, false)),
        FieldChrome::Error
    );
}

#[test]
fn focus_outranks_hover_and_rest_follows() {
    let rest = field_state(true, false, false, false, false);
    assert_eq!(
        resolve_chrome(FieldChromeState { focused: true, ..rest }),
        FieldChrome::Focus
    );
    assert_eq!(
        resolve_chrome(FieldChromeState { hovered: true, ..rest }),
        FieldChrome::Hover
    );
    assert_eq!(resolve_chrome(rest), FieldChrome::Rest);
}

#[test]
fn bordered_field_paints_a_resting_outline_but_states_still_upgrade() {
    let bordered_rest = field_state(true, false, false, false, true);
    assert_eq!(resolve_chrome(bordered_rest), FieldChrome::Bordered);
    assert_eq!(
        resolve_chrome(FieldChromeState {
            hovered: true,
            ..bordered_rest
        }),
        FieldChrome::Hover
    );
    assert_eq!(
        resolve_chrome(FieldChromeState {
            focused: true,
            ..bordered_rest
        }),
        FieldChrome::Focus
    );
}

#[test]
fn stroke_widths_keep_the_existing_rendered_values() {
    assert_eq!(BORDER_STROKE_WIDTH, 1.0);
    assert_eq!(FOCUS_STROKE_WIDTH, 1.5);
    assert_eq!(ERROR_STROKE_WIDTH, 1.5);
}
