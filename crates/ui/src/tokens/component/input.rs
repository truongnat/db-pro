//! Input component contract (single-line fields, textareas, password/search fields).
//!
//! States supported: default, hover, focus, error, disabled. Text fields have no
//! loading/active/selected states today. Precedence (strongest first):
//! `disabled → error → focus → hover → default`; `error` is a display status
//! inserted directly after `disabled`, so a disabled field never wears a focus
//! ring or an error border. `components/input/layout.rs` resolves every painted
//! border through [`resolve_chrome`].
//!
//! Semantic roles consumed (via `DbProTheme`): `border.default/strong` for the
//! resting/hover border, `border.focus` for the focus outline, `status.danger.solid`
//! for errors, `foreground.disabled` for disabled text, `background.subtle` for the
//! disabled fill.
//!
//! Owned values: field heights and chrome stroke widths. Min width, inner margins,
//! and icon/gap sizes stay in `components/input/config.rs` as component geometry;
//! corner radius and spacing come directly from the shared radius/spacing tokens.

use crate::tokens::{STROKE_FOCUS, STROKE_THIN};

/// Default/single-line field height, in egui points.
pub const INPUT_HEIGHT_DEFAULT: f32 = 38.0;
/// Compact field height, in egui points.
pub const INPUT_HEIGHT_SM: f32 = 30.0;

/// Resting and hover border width, in egui points.
pub const BORDER_STROKE_WIDTH: f32 = STROKE_THIN;
/// Focus outline width, in egui points.
pub const FOCUS_STROKE_WIDTH: f32 = STROKE_FOCUS;
/// Error outline width, in egui points (same weight as focus by design).
pub const ERROR_STROKE_WIDTH: f32 = STROKE_FOCUS;

/// Raw state flags of one field, as collected by the layout before painting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldChromeState {
    pub focused: bool,
    pub hovered: bool,
    pub enabled: bool,
    pub has_error: bool,
}

/// Resolved chrome state of a text field, strongest state first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldChrome {
    Disabled,
    Error,
    Focus,
    Hover,
    Rest,
}

/// Apply the contract precedence to a field's raw state flags.
pub fn resolve_chrome(state: FieldChromeState) -> FieldChrome {
    if !state.enabled {
        FieldChrome::Disabled
    } else if state.has_error {
        FieldChrome::Error
    } else if state.focused {
        FieldChrome::Focus
    } else if state.hovered {
        FieldChrome::Hover
    } else {
        FieldChrome::Rest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_outranks_error_focus_and_hover() {
        let state = FieldChromeState {
            enabled: false,
            has_error: true,
            focused: true,
            hovered: true,
        };
        assert_eq!(resolve_chrome(state), FieldChrome::Disabled);
    }

    #[test]
    fn error_outranks_focus_and_hover_but_not_disabled() {
        let focused_error = FieldChromeState {
            enabled: true,
            has_error: true,
            focused: true,
            hovered: true,
        };
        assert_eq!(resolve_chrome(focused_error), FieldChrome::Error);
        let hovered_error = FieldChromeState {
            enabled: true,
            has_error: true,
            focused: false,
            hovered: true,
        };
        assert_eq!(resolve_chrome(hovered_error), FieldChrome::Error);
    }

    #[test]
    fn focus_outranks_hover_and_rest_follows() {
        let base = FieldChromeState {
            enabled: true,
            has_error: false,
            focused: false,
            hovered: false,
        };
        assert_eq!(
            resolve_chrome(FieldChromeState { focused: true, ..base }),
            FieldChrome::Focus
        );
        assert_eq!(
            resolve_chrome(FieldChromeState { hovered: true, ..base }),
            FieldChrome::Hover
        );
        assert_eq!(resolve_chrome(base), FieldChrome::Rest);
    }

    #[test]
    fn stroke_widths_keep_the_existing_rendered_values() {
        assert_eq!(BORDER_STROKE_WIDTH, 1.0);
        assert_eq!(FOCUS_STROKE_WIDTH, 1.5);
        assert_eq!(ERROR_STROKE_WIDTH, 1.5);
    }
}
