//! Core token facade for the native shell — `Primitive → Semantic → Component`.
//!
//! - `primitive` (private): raw palettes and the shared scales. Palettes are not
//!   re-exported; component rendering must never read a raw theme value.
//! - `semantic`: purpose-based roles with explicit light/dark treatments,
//!   flattened into `DbProTheme` for compatibility.
//! - `component`: token contracts for the representative components (button,
//!   input, dialog/overlay, table/grid, feedback/status) — sizes they own plus
//!   their state precedence.
//!
//! Existing `use crate::tokens::*` imports keep resolving: the scale constants
//! and font helpers below are single-definition re-exports of the primitive layer.

pub mod component;
mod primitive;
pub mod semantic;

pub use primitive::{
    font_body, font_body_sm, font_caption, font_display, font_icon, font_mono_sm, font_mono_ui, font_page_title,
    font_section_title, font_subheading, font_ui_label, CARD_INNER_PAD, CONTROL_GROUP_GAP, DURATION_HOVER_SECS,
    DURATION_OVERLAY_SECS, DURATION_PRESS_SECS, DURATION_TAB_TRANSITION_SECS, DURATION_TOAST_DEFAULT_SECS,
    FONT_SIZE_BADGE, FONT_SIZE_BODY, FONT_SIZE_BODY_SM, FONT_SIZE_CAPTION, FONT_SIZE_DISPLAY, FONT_SIZE_KBD,
    FONT_SIZE_MONO_SM, FONT_SIZE_MONO_UI, FONT_SIZE_PAGE_TITLE, FONT_SIZE_SECTION_TITLE, FONT_SIZE_SUBHEADING,
    FONT_SIZE_UI_LABEL, FORM_FIELD_GAP, ICON_DEFAULT, ICON_LG, ICON_SM, ICON_TEXT_GAP, ICON_TOOLBAR, ICON_XL, ICON_XS,
    LABEL_HELPER_GAP, RADIUS_2XL, RADIUS_BADGE, RADIUS_BADGE_PILL, RADIUS_BUTTON, RADIUS_CARD, RADIUS_CODE_BLOCK,
    RADIUS_COMPOSER, RADIUS_DIALOG, RADIUS_DROPDOWN, RADIUS_FULL, RADIUS_ICON_BUTTON, RADIUS_INPUT, RADIUS_LG,
    RADIUS_MD, RADIUS_POPOVER, RADIUS_SM, RADIUS_TOAST, RADIUS_XL, RADIUS_XS, SECTION_GAP_LG, SECTION_GAP_MD,
    SECTION_GAP_SM, SHADOW_ALPHA_DARK, SHADOW_ALPHA_LIGHT, SHADOW_BLUR, SHADOW_OFFSET_Y, SHELL_SPLIT_INSET, SPACE_2XL,
    SPACE_3XL, SPACE_4XL, SPACE_5XL, SPACE_6XL, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXS,
    STROKE_FOCUS, STROKE_THICK, STROKE_THIN, WINDOW_SHADOW_ALPHA,
};

// ── Shell chrome sizes (app-frame geometry; not part of a component contract) ──

pub const TREE_ROW_HEIGHT: f32 = 28.0;
pub const SIDEBAR_ROW_HEIGHT: f32 = 32.0;
pub const STATUS_BAR_HEIGHT: f32 = 26.0;
pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;
pub const TOOLBAR_HEIGHT: f32 = 38.0;
