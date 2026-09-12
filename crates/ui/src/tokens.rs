//! Design Token System for DB Pro (Native Shell).
//!
//! Implements the Warm Minimalism + Editorial UI + Developer Tool Aesthetic
//! specified in `open-api-style.md`.
//! Single source of truth for typography, spacing, radius, stroke, icons, motion, and elevations.

use egui::{FontFamily, FontId, Vec2};

// ── 1. Spacing Tokens (4px base grid) ────────────────────────────────────────

pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 20.0;
pub const SPACE_2XL: f32 = 24.0;
pub const SPACE_3XL: f32 = 32.0;
pub const SPACE_4XL: f32 = 40.0;
pub const SPACE_5XL: f32 = 48.0;
pub const SPACE_6XL: f32 = 64.0;

// Semantic Spacing
pub const ICON_TEXT_GAP: f32 = 8.0;
pub const LABEL_HELPER_GAP: f32 = 4.0;
pub const FORM_FIELD_GAP: f32 = 14.0;
pub const CONTROL_GROUP_GAP: f32 = 16.0;
pub const SECTION_GAP_SM: f32 = 24.0;
pub const SECTION_GAP_MD: f32 = 32.0;
pub const SECTION_GAP_LG: f32 = 48.0;
pub const CARD_INNER_PAD: f32 = 14.0;

// ── 2. Corner Radius Tokens ──────────────────────────────────────────────────

pub const RADIUS_XS: f32 = 4.0;
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;
pub const RADIUS_2XL: f32 = 24.0;
pub const RADIUS_FULL: f32 = 999.0;

// Semantic Radius Mappings
pub const RADIUS_BUTTON: f32 = RADIUS_MD; // 8.0
pub const RADIUS_ICON_BUTTON: f32 = RADIUS_SM; // 6.0
pub const RADIUS_INPUT: f32 = RADIUS_MD; // 8.0
pub const RADIUS_DROPDOWN: f32 = 10.0;
pub const RADIUS_POPOVER: f32 = RADIUS_LG; // 12.0
pub const RADIUS_CARD: f32 = RADIUS_LG; // 12.0
pub const RADIUS_DIALOG: f32 = RADIUS_XL; // 16.0
pub const RADIUS_COMPOSER: f32 = 20.0;
pub const RADIUS_BADGE_PILL: f32 = RADIUS_FULL; // 999.0
pub const RADIUS_TOAST: f32 = 10.0;
pub const RADIUS_CODE_BLOCK: f32 = RADIUS_MD; // 8.0

// ── 3. Typography Scale & Font Helpers ───────────────────────────────────────

pub const FONT_SIZE_DISPLAY: f32 = 32.0;
pub const FONT_SIZE_PAGE_TITLE: f32 = 24.0;
pub const FONT_SIZE_SECTION_TITLE: f32 = 18.0;
pub const FONT_SIZE_SUBHEADING: f32 = 16.0;
pub const FONT_SIZE_BODY: f32 = 15.0;
pub const FONT_SIZE_BODY_SM: f32 = 14.0;
pub const FONT_SIZE_UI_LABEL: f32 = 13.0;
pub const FONT_SIZE_CAPTION: f32 = 12.0;
pub const FONT_SIZE_MONO_UI: f32 = 13.0;
pub const FONT_SIZE_MONO_SM: f32 = 11.5;
pub const FONT_SIZE_BADGE: f32 = 11.0;
pub const FONT_SIZE_KBD: f32 = 11.0;

pub fn font_display() -> FontId {
    FontId::new(FONT_SIZE_DISPLAY, FontFamily::Name("ui_medium".into()))
}

pub fn font_page_title() -> FontId {
    FontId::new(FONT_SIZE_PAGE_TITLE, FontFamily::Name("ui_medium".into()))
}

pub fn font_section_title() -> FontId {
    FontId::new(FONT_SIZE_SECTION_TITLE, FontFamily::Name("ui_medium".into()))
}

pub fn font_subheading() -> FontId {
    FontId::new(FONT_SIZE_SUBHEADING, FontFamily::Name("ui_medium".into()))
}

pub fn font_body() -> FontId {
    FontId::proportional(FONT_SIZE_BODY)
}

pub fn font_body_sm() -> FontId {
    FontId::proportional(FONT_SIZE_BODY_SM)
}

pub fn font_ui_label() -> FontId {
    FontId::new(FONT_SIZE_UI_LABEL, FontFamily::Name("ui_medium".into()))
}

pub fn font_caption() -> FontId {
    FontId::proportional(FONT_SIZE_CAPTION)
}

pub fn font_mono_ui() -> FontId {
    FontId::monospace(FONT_SIZE_MONO_UI)
}

pub fn font_mono_sm() -> FontId {
    FontId::monospace(FONT_SIZE_MONO_SM)
}

pub fn font_icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("lucide".into()))
}

// ── 4. Icon Size Tokens ──────────────────────────────────────────────────────

pub const ICON_XS: f32 = 12.0;
pub const ICON_SM: f32 = 14.0;
pub const ICON_DEFAULT: f32 = 16.0;
pub const ICON_TOOLBAR: f32 = 18.0;
pub const ICON_LG: f32 = 20.0;
pub const ICON_XL: f32 = 24.0;

// ── 5. Stroke & Border Tokens ────────────────────────────────────────────────

pub const STROKE_THIN: f32 = 1.0;
pub const STROKE_FOCUS: f32 = 1.5;
pub const STROKE_THICK: f32 = 2.0;

// ── 6. Motion & Animation Tokens ─────────────────────────────────────────────

pub const DURATION_PRESS_SECS: f32 = 0.080;
pub const DURATION_HOVER_SECS: f32 = 0.140;
pub const DURATION_OVERLAY_SECS: f32 = 0.180;
pub const DURATION_TAB_TRANSITION_SECS: f32 = 0.200;
pub const DURATION_TOAST_DEFAULT_SECS: f32 = 5.0;

// ── 7. Component Size Constants ──────────────────────────────────────────────

pub const BUTTON_HEIGHT_DEFAULT: f32 = 36.0;
pub const BUTTON_HEIGHT_SM: f32 = 28.0;
pub const BUTTON_HEIGHT_LG: f32 = 42.0;
pub const BUTTON_ICON_SIZE_DEFAULT: Vec2 = Vec2::new(32.0, 32.0);
pub const BUTTON_ICON_SIZE_SM: Vec2 = Vec2::new(26.0, 26.0);
pub const BUTTON_ICON_SIZE_LG: Vec2 = Vec2::new(38.0, 38.0);

pub const INPUT_HEIGHT_DEFAULT: f32 = 38.0;
pub const INPUT_HEIGHT_SM: f32 = 30.0;

pub const TABLE_ROW_HEIGHT_DEFAULT: f32 = 40.0;
pub const TABLE_ROW_HEIGHT_COMPACT: f32 = 32.0;
pub const TABLE_HEADER_HEIGHT: f32 = 36.0;

pub const TREE_ROW_HEIGHT: f32 = 28.0;
pub const SIDEBAR_ROW_HEIGHT: f32 = 32.0;
pub const STATUS_BAR_HEIGHT: f32 = 26.0;
pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;
pub const TOOLBAR_HEIGHT: f32 = 38.0;
