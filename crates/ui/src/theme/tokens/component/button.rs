//! Button component contract.
//!
//! States supported: default, hover, focus, active (press), disabled, loading.
//! Precedence (strongest first): `disabled → loading → active → focus → hover →
//! default` — declared in [`crate::tokens::component::STATE_PRECEDENCE`] and
//! enforced by [`shows_loading`] plus the disabled palette in
//! `components/button/ui/button.rs`.
//!
//! Semantic roles consumed (via `DbProTheme`): `accent.solid/solid_hover`
//! for the primary variant, `status.danger.solid` for destructive, `background.subtle`
//! at rest and `background.hover` while hovered for secondary, `background.subtle`
//! for the disabled fill, `border.subtle/default/strong`
//! for outline and disabled strokes, `foreground.primary/disabled` for labels,
//! `text_on_solid` for filled labels, and
//! `border.focus` painted at `STROKE_THICK` by `components/interact.rs`.
//!
//! Owned values: height, padding, label font size, glyph size, hit-box size, and
//! fallback width per size variant — the single source of truth for button
//! dimensions. Radius and group gap come from shared tokens; link underline
//! width remains in `components/button/config.rs` as local geometry.

use egui::Vec2;

use super::super::primitive::{
    FONT_SIZE_BADGE, FONT_SIZE_BODY_SM, FONT_SIZE_CAPTION, FONT_SIZE_UI_LABEL, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS,
};

// Height (min) per size variant, in egui points.
pub const BUTTON_HEIGHT_DEFAULT: f32 = 32.0;
pub const BUTTON_HEIGHT_SM: f32 = 28.0;
pub const BUTTON_HEIGHT_LG: f32 = 38.0;

// Square fallback widths for icon-only variants, in egui points.
pub const BUTTON_DEFAULT_WIDTH: f32 = 32.0;
pub const BUTTON_LG_WIDTH: f32 = 38.0;

// Padding (x, y) per size variant, taken from the 4px spacing scale.
pub const BUTTON_SM_PADDING: Vec2 = Vec2::new(SPACE_SM, SPACE_XS);
pub const BUTTON_DEFAULT_PADDING: Vec2 = Vec2::new(SPACE_MD, SPACE_XS);
pub const BUTTON_LG_PADDING: Vec2 = Vec2::new(SPACE_LG, SPACE_SM);
pub const BUTTON_ICON_PADDING: Vec2 = Vec2::new(SPACE_SM, SPACE_SM);
pub const BUTTON_ICON_SM_PADDING: Vec2 = Vec2::new(SPACE_XS, SPACE_XS);

// Label font sizes per size variant, taken from the shared type scale.
pub const BUTTON_FONT_SIZE_SM: f32 = FONT_SIZE_CAPTION;
pub const BUTTON_FONT_SIZE_DEFAULT: f32 = FONT_SIZE_UI_LABEL;
pub const BUTTON_FONT_SIZE_LG: f32 = FONT_SIZE_BODY_SM;
pub const BUTTON_ICON_SM_FONT_SIZE: f32 = FONT_SIZE_BADGE;

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
#[path = "tests/button_tests.rs"]
mod tests;
