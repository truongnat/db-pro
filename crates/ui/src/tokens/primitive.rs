//! Primitive layer — raw values and shared scales for the native shell.
//!
//! Layer 1 of `Primitive → Semantic → Component`. Values here describe *how much*
//! (spacing, radius, type sizes, strokes, motion) or *which raw color* (`palette`),
//! never *what a value is for*. Scale constants are re-exported through the
//! `crate::tokens` facade so existing imports keep resolving; the raw palette is
//! intentionally private to this module tree so component rendering cannot read a
//! theme value without going through the semantic layer.

use egui::{FontFamily, FontId};

// ── 1. Spacing scale (4px base grid) ─────────────────────────────────────────

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

// Semantic spacing roles mapped onto the scale.
pub const ICON_TEXT_GAP: f32 = 8.0;
pub const LABEL_HELPER_GAP: f32 = 4.0;
pub const FORM_FIELD_GAP: f32 = 14.0;
pub const CONTROL_GROUP_GAP: f32 = 12.0;
pub const SECTION_GAP_SM: f32 = 24.0;
pub const SECTION_GAP_MD: f32 = 32.0;
pub const SECTION_GAP_LG: f32 = 48.0;
pub const CARD_INNER_PAD: f32 = 12.0;
/// Shared inset on both sides of the sidebar↔workspace splitter.
/// Matches sidebar `pad_left` (`SPACE_SM`) so navigator and body stay aligned.
pub const SHELL_SPLIT_INSET: f32 = SPACE_SM;

// ── 2. Corner radius scale ───────────────────────────────────────────────────

pub const RADIUS_XS: f32 = 4.0;
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;
pub const RADIUS_2XL: f32 = 24.0;
pub const RADIUS_FULL: f32 = 999.0;

// Radius role mappings (Stitch Native Spec: 4px controls/cards, 6px dialogs/popovers).
// Component contracts consume these roles; they are not renderer-specific.
pub const RADIUS_BUTTON: f32 = RADIUS_SM; // 6.0
pub const RADIUS_ICON_BUTTON: f32 = RADIUS_SM; // 6.0
pub const RADIUS_INPUT: f32 = RADIUS_MD; // 8.0
pub const RADIUS_DROPDOWN: f32 = RADIUS_MD; // 8.0
pub const RADIUS_POPOVER: f32 = RADIUS_MD; // 8.0
pub const RADIUS_CARD: f32 = RADIUS_MD; // 8.0
pub const RADIUS_DIALOG: f32 = RADIUS_LG; // 12.0
pub const RADIUS_COMPOSER: f32 = RADIUS_XL; // 16.0
pub const RADIUS_BADGE: f32 = RADIUS_FULL; // 999.0
pub const RADIUS_BADGE_PILL: f32 = RADIUS_FULL; // 999.0
pub const RADIUS_TOAST: f32 = RADIUS_MD; // 8.0
pub const RADIUS_CODE_BLOCK: f32 = RADIUS_MD; // 8.0

// ── 3. Typography scale & font helpers ───────────────────────────────────────

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

// ── 4. Icon glyph sizes (hit-box sizes are owned by each component contract) ─

pub const ICON_XS: f32 = 12.0;
pub const ICON_SM: f32 = 14.0;
pub const ICON_DEFAULT: f32 = 16.0;
pub const ICON_TOOLBAR: f32 = 18.0;
pub const ICON_LG: f32 = 20.0;
pub const ICON_XL: f32 = 24.0;

// ── 5. Stroke & elevation scales ─────────────────────────────────────────────

pub const STROKE_THIN: f32 = 1.0;
pub const STROKE_FOCUS: f32 = 1.5;
pub const STROKE_THICK: f32 = 2.0;

// Elevation values are shared by windows, menus, popovers, and dialogs.
pub const SHADOW_OFFSET_Y: f32 = 8.0;
pub const SHADOW_BLUR: f32 = 24.0;
pub const SHADOW_ALPHA_LIGHT: u8 = 20;
pub const SHADOW_ALPHA_DARK: u8 = 40;
pub const WINDOW_SHADOW_ALPHA: u8 = 28;

// ── 6. Motion durations ──────────────────────────────────────────────────────

pub const DURATION_PRESS_SECS: f32 = 0.080;
pub const DURATION_HOVER_SECS: f32 = 0.140;
pub const DURATION_OVERLAY_SECS: f32 = 0.180;
pub const DURATION_TAB_TRANSITION_SECS: f32 = 0.200;
pub const DURATION_TOAST_DEFAULT_SECS: f32 = 5.0;

// ── 7. Raw light/dark palettes ───────────────────────────────────────────────
//
// Raw values only — no role is implied here. `super::semantic` maps these onto
// purpose-based roles per theme. Neutrals are numbered as a ladder within each
// palette (lighter = lower); hue names carry their own step. The exact hex lives
// in the doc comment so a drift is visible at the definition site.

/// Raw palette values for the Codex-aligned light theme.
pub mod light {
    use egui::Color32;

    // Neutrals.
    pub const NEUTRAL_0: Color32 = Color32::from_rgb(255, 255, 255); // #ffffff
    pub const NEUTRAL_50: Color32 = Color32::from_rgb(249, 249, 249); // #f9f9f9
    pub const NEUTRAL_75: Color32 = Color32::from_rgb(246, 246, 246); // #f6f6f6
    pub const NEUTRAL_100: Color32 = Color32::from_rgb(241, 241, 241); // #f1f1f1
    pub const NEUTRAL_125: Color32 = Color32::from_rgb(238, 238, 238); // #eeeeee
    pub const NEUTRAL_150: Color32 = Color32::from_rgb(229, 229, 229); // #e5e5e5
    pub const NEUTRAL_175: Color32 = Color32::from_rgb(212, 212, 212); // #d4d4d4
    pub const NEUTRAL_450: Color32 = Color32::from_rgb(138, 138, 138); // #8a8a8a
    pub const NEUTRAL_500: Color32 = Color32::from_rgb(112, 112, 112); // #707070
    pub const NEUTRAL_550: Color32 = Color32::from_rgb(110, 110, 110); // #6e6e6e
    pub const NEUTRAL_575: Color32 = Color32::from_rgb(100, 100, 110); // #64646e
    pub const NEUTRAL_600: Color32 = Color32::from_rgb(95, 95, 95); // #5f5f5f
    pub const NEUTRAL_900: Color32 = Color32::from_rgb(26, 28, 31); // #1a1c1f
    pub const NEUTRAL_950: Color32 = Color32::from_rgb(26, 28, 31); // #1a1c1f

    // Accent (Modern Blue).
    pub const BLUE_50: Color32 = Color32::from_rgb(230, 242, 255); // #e6f2ff
    pub const BLUE_500: Color32 = Color32::from_rgb(0, 111, 204); // #006fcc
    pub const BLUE_600: Color32 = Color32::from_rgb(1, 105, 204); // #0169cc
    pub const BLUE_700: Color32 = Color32::from_rgb(37, 99, 235); // #2563eb

    // Shipped status colors.
    pub const GREEN_600: Color32 = Color32::from_rgb(21, 128, 61); // #15803d
    pub const AMBER_600: Color32 = Color32::from_rgb(180, 83, 9); // #b45309
    pub const RED_600: Color32 = Color32::from_rgb(185, 28, 28); // #b91c1c

    // SQL syntax hues (restrained Zed-like light palette; not UI accent clones).
    pub const INDIGO_700: Color32 = Color32::from_rgb(55, 65, 180); // #3741b4
    pub const GREEN_700: Color32 = Color32::from_rgb(15, 118, 70); // #0f7646
    pub const AMBER_700: Color32 = Color32::from_rgb(180, 83, 9); // #b45309
    pub const PURPLE_700: Color32 = Color32::from_rgb(126, 34, 206); // #7e22ce
    pub const CYAN_700: Color32 = Color32::from_rgb(14, 116, 144); // #0e7490

    /// Modal scrim: black at ~15% so the dialog stays the brightest surface.
    pub const SCRIM: Color32 = Color32::from_black_alpha(38);
}

/// Raw palette values for the ChatGPT / OpenAI monochrome dark theme.
pub mod dark {
    use egui::Color32;

    // Neutrals.
    pub const NEUTRAL_0: Color32 = Color32::from_rgb(255, 255, 255); // #ffffff
    pub const NEUTRAL_50: Color32 = Color32::from_rgb(236, 236, 236); // #ececec (primary text & white pill)
    pub const NEUTRAL_200: Color32 = Color32::from_rgb(180, 180, 180); // #b4b4b4 (secondary text)
    pub const NEUTRAL_350: Color32 = Color32::from_rgb(160, 160, 160); // #a0a0a0 (tertiary text)
    pub const NEUTRAL_450: Color32 = Color32::from_rgb(142, 142, 142); // #8e8e8e (muted text)
    pub const NEUTRAL_500: Color32 = Color32::from_rgb(90, 90, 90); // #5a5a5a (disabled)
    pub const NEUTRAL_650: Color32 = Color32::from_rgb(74, 74, 74); // #4a4a4a (border strong)
    pub const NEUTRAL_700: Color32 = Color32::from_rgb(58, 58, 58); // #3a3a3a (border default)
    pub const NEUTRAL_740: Color32 = Color32::from_rgb(44, 44, 44); // #2c2c2c (border subtle)
    pub const NEUTRAL_750: Color32 = Color32::from_rgb(52, 52, 52); // #343434 (surface hover)
    pub const NEUTRAL_775: Color32 = Color32::from_rgb(40, 40, 40); // #282828 (surface subtle)
    pub const NEUTRAL_780: Color32 = Color32::from_rgb(47, 47, 47); // #2f2f2f (surface elevated & cards)
    pub const NEUTRAL_810: Color32 = Color32::from_rgb(23, 23, 23); // #171717 (surface panel / sidebar)
    pub const NEUTRAL_850: Color32 = Color32::from_rgb(33, 33, 33); // #212121 (surface canvas / app)
    pub const NEUTRAL_900: Color32 = Color32::from_rgb(13, 13, 13); // #0d0d0d (pure dark)

    // Status / Accent.
    pub const GREEN_500: Color32 = Color32::from_rgb(16, 163, 127); // #10a37f (OpenAI Emerald)
    pub const AMBER_400: Color32 = Color32::from_rgb(234, 179, 8); // #eab308
    pub const RED_400: Color32 = Color32::from_rgb(239, 68, 68); // #ef4444
    pub const BLUE_450: Color32 = Color32::from_rgb(96, 165, 250); // #60a5fa

    // SQL syntax hues.
    pub const PURPLE_400: Color32 = Color32::from_rgb(199, 146, 234); // #c792ea
    pub const CYAN_300: Color32 = Color32::from_rgb(137, 221, 255); // #89ddff

    /// Modal scrim: black at ~29% to dim the workstation surfaces.
    pub const SCRIM: Color32 = Color32::from_black_alpha(74);
}
