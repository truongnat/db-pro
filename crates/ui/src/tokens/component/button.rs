//! Button component contract.
//!
//! States supported: default, hover, focus, active (press), disabled, loading.
//! Precedence (strongest first): `disabled → loading → active → focus → hover →
//! default` — declared in [`crate::tokens::component::STATE_PRECEDENCE`] and
//! enforced by [`shows_loading`] plus the disabled palette in
//! `components/button/ui/button.rs`.
//!
//! Semantic roles consumed (via `DbProTheme`): `accent.solid/solid_hover`
//! for the primary variant, `status.danger.solid` for destructive, `background.hover/
//! selected/subtle` for secondary and disabled fills, `border.subtle/default/strong`
//! for outline and disabled strokes, `foreground.primary/disabled` for labels,
//! `text_on_solid` for filled labels, and
//! `border.focus` painted at `STROKE_THICK` by `components/interact.rs`.
//!
//! Owned values: height, padding, label font size, glyph size, hit-box size, and
//! fallback width per size variant — the single source of truth for button
//! dimensions. Radius and group gap come from shared tokens; link underline
//! width remains in `components/button/config.rs` as local geometry.

use egui::Vec2;

// Height (min) per size variant, in egui points.
pub const BUTTON_HEIGHT_DEFAULT: f32 = 32.0;
pub const BUTTON_HEIGHT_SM: f32 = 28.0;
pub const BUTTON_HEIGHT_LG: f32 = 38.0;

// Square fallback widths for icon-only variants, in egui points.
pub const BUTTON_DEFAULT_WIDTH: f32 = 32.0;
pub const BUTTON_LG_WIDTH: f32 = 38.0;

// Padding (x, y) per size variant, in egui points.
pub const BUTTON_SM_PADDING: Vec2 = Vec2::new(8.0, 4.0);
pub const BUTTON_DEFAULT_PADDING: Vec2 = Vec2::new(10.0, 5.0);
pub const BUTTON_LG_PADDING: Vec2 = Vec2::new(14.0, 7.0);
pub const BUTTON_ICON_PADDING: Vec2 = Vec2::new(8.0, 8.0);
pub const BUTTON_ICON_SM_PADDING: Vec2 = Vec2::new(6.0, 6.0);

// Label font sizes per size variant, in egui points.
pub const BUTTON_FONT_SIZE_SM: f32 = 11.5;
pub const BUTTON_FONT_SIZE_DEFAULT: f32 = 12.5;
pub const BUTTON_FONT_SIZE_LG: f32 = 13.5;
pub const BUTTON_ICON_SM_FONT_SIZE: f32 = 11.0;

// Glyph sizes: the lucide glyph itself, in egui points.
pub const BUTTON_ICON_SIZE_SM_GLYPH: f32 = 13.0;
pub const BUTTON_ICON_SIZE_DEFAULT_GLYPH: f32 = 14.5;
pub const BUTTON_ICON_SIZE_LG_GLYPH: f32 = 16.0;
pub const BUTTON_ICON_SIZE_GLYPH: f32 = 15.0;

// Hit-box sizes for square icon-only buttons (allocation box, not glyph size),
// in egui points. Kept separate from the glyph sizes above.
pub const BUTTON_ICON_SIZE_DEFAULT: Vec2 = Vec2::new(32.0, 32.0);
pub const BUTTON_ICON_SIZE_SM: Vec2 = Vec2::new(26.0, 26.0);
pub const BUTTON_ICON_SIZE_LG: Vec2 = Vec2::new(38.0, 38.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionState {
    Disabled,
    Loading,
    Active,
    Focus,
    Hover,
    Default,
}

/// Resolves the button state in the contract order before painting.
pub fn resolve_interaction_state(
    enabled: bool,
    loading: bool,
    active: bool,
    focused: bool,
    hovered: bool,
) -> InteractionState {
    if !enabled {
        InteractionState::Disabled
    } else if loading {
        InteractionState::Loading
    } else if active {
        InteractionState::Active
    } else if focused {
        InteractionState::Focus
    } else if hovered {
        InteractionState::Hover
    } else {
        InteractionState::Default
    }
}

/// Whether the loading spinner may replace the interactive surface. `disabled`
/// outranks `loading` in the state precedence, so a button that is both disabled
/// and loading renders its disabled state instead of the spinner.
pub fn shows_loading(enabled: bool, loading: bool) -> bool {
    enabled && loading
}

#[cfg(test)]
mod tests {
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

    /// Existing rendered dimensions must not drift: these values were previously
    /// spread across `components/button/config.rs` literals and `tokens.rs`.
    #[test]
    fn dimensions_preserve_the_existing_rendered_values() {
        assert_eq!(BUTTON_HEIGHT_SM, 28.0);
        assert_eq!(BUTTON_HEIGHT_DEFAULT, 32.0);
        assert_eq!(BUTTON_HEIGHT_LG, 38.0);
        assert_eq!(BUTTON_DEFAULT_PADDING, Vec2::new(10.0, 5.0));
        assert_eq!(BUTTON_ICON_SIZE_SM, Vec2::new(26.0, 26.0));
    }
}
