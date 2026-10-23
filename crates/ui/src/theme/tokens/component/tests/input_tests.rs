//! Tests for `../input.rs` (moved out of the source file).
use super::*;

fn field_state(enabled: bool, has_error: bool, focused: bool, hovered: bool) -> FieldChromeState {
    FieldChromeState {
        enabled,
        has_error,
        focused,
        hovered,
    }
}

#[test]
fn disabled_outranks_error_focus_and_hover() {
    let state = field_state(false, true, true, true);
    assert_eq!(resolve_chrome(state), FieldChrome::Disabled);
}

#[test]
fn error_outranks_focus_and_hover_but_not_disabled() {
    assert_eq!(resolve_chrome(field_state(true, true, true, true)), FieldChrome::Error);
    assert_eq!(resolve_chrome(field_state(true, true, false, true)), FieldChrome::Error);
}

#[test]
fn focus_outranks_hover_and_rest_follows() {
    let rest = field_state(true, false, false, false);
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

/// `Rest` is a painted state, not "paint nothing": it owns the `border.default`
/// outline that gives a resting field its boundary. Only `Disabled` is outline-free.
#[test]
fn rest_is_the_only_bounded_default_and_disabled_is_outline_free() {
    assert_eq!(
        resolve_chrome(field_state(true, false, false, false)),
        FieldChrome::Rest
    );
    assert_eq!(
        resolve_chrome(field_state(false, false, false, false)),
        FieldChrome::Disabled
    );
    assert_ne!(FieldChrome::Rest, FieldChrome::Disabled);
}

#[test]
fn stroke_widths_keep_the_existing_rendered_values() {
    assert_eq!(BORDER_STROKE_WIDTH, 1.0);
    assert_eq!(FOCUS_STROKE_WIDTH, 1.5);
    assert_eq!(ERROR_STROKE_WIDTH, 1.5);
}
