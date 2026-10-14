//! Input component contract (single-line fields, textareas, password/search fields).
//!
//! States supported: default, hover, focus, error, disabled. Resting fields are
//! unoutlined; hover adds a subtle outline, while focus and errors use clear
//! semantic strokes. Text fields have no loading/active/selected states today.
//! Precedence (strongest first): `disabled → error → focus → hover → default`;
//! `components/input/layout.rs` resolves every painted border through
//! [`resolve_chrome`].
//!
//! Semantic roles consumed (via `DbProTheme`): `border.strong` for hover,
//! `border.focus` for the focus outline, `status.danger.solid` for errors,
//! `foreground.disabled` for disabled text, and `background.subtle` for disabled
//! fill.
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
    /// When true the field keeps a resting border (`border.default`) instead of
    /// relying on fill alone; hover/focus/error still upgrade the stroke.
    pub bordered: bool,
}

/// Resolved chrome state of a text field, strongest state first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldChrome {
    Disabled,
    Error,
    Focus,
    Hover,
    /// Bordered resting state (static outline).
    Bordered,
    /// Default resting state (no outline).
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
    } else if state.bordered {
        FieldChrome::Bordered
    } else {
        FieldChrome::Rest
    }
}

#[cfg(test)]
#[path = "tests/input_tests.rs"]
mod tests;
