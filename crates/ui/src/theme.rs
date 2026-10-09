// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Vec2, Visuals};

use crate::tokens::semantic::{subtle_wash, SemanticTokens};
use crate::tokens::{
    FONT_SIZE_BADGE, FONT_SIZE_MONO_UI, FONT_SIZE_UI_LABEL, RADIUS_BUTTON, RADIUS_DIALOG, RADIUS_POPOVER,
    SHADOW_ALPHA_DARK, SHADOW_ALPHA_LIGHT, SHADOW_BLUR, SHADOW_OFFSET_Y, SPACE_2XL, SPACE_MD, SPACE_SM, SPACE_XS,
    STROKE_THIN, WINDOW_SHADOW_ALPHA,
};

// Renderer adapter mapping: these translate design tokens into egui's own style
// slots and are deliberately private to this adapter rather than public tokens.
const RADIUS_EGUI_WINDOW: f32 = RADIUS_DIALOG;
const RADIUS_EGUI_MENU: f32 = RADIUS_POPOVER;
const RADIUS_EGUI_WIDGET: f32 = RADIUS_BUTTON;
const EGUI_ITEM_SPACING: Vec2 = Vec2::new(SPACE_SM, SPACE_XS);
const EGUI_BUTTON_PADDING: Vec2 = Vec2::new(10.0, SPACE_XS);
const EGUI_INTERACT_SIZE: Vec2 = Vec2::new(SPACE_2XL, SPACE_2XL);
const EGUI_WINDOW_MARGIN: f32 = SPACE_MD;
const EGUI_MENU_MARGIN: f32 = 5.0;
const EGUI_INDENT: f32 = 13.0;

const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const INTER_REGULAR_EXT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular-ext.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
const INTER_MEDIUM_EXT: &[u8] = include_bytes!("../assets/fonts/Inter-Medium-ext.ttf");

/// Product-owned visual tokens for the native shell.
///
/// Flat compatibility facade over [`SemanticTokens`]: every field is built from
/// the corresponding role, so existing callers keep working while the semantic
/// layer stays authoritative.
#[derive(Debug, Clone, Copy)]
pub struct DbProTheme {
    pub dark_mode: bool,
    /// Current accessibility preference, carried with the theme to shared widgets.
    pub reduce_motion: bool,
    /// Full role detail for callers that need more than one flat field at a time
    /// (e.g. status fill/border pairs).
    pub semantic: SemanticTokens,
    pub surface_app: Color32,
    pub surface_panel: Color32,
    pub surface_elevated: Color32,
    pub surface_floating: Color32,
    pub surface_editor: Color32,
    pub surface_hover: Color32,
    /// Compatibility alias of `background.selected` — active/pressed widget and
    /// selected-row wash.
    pub surface_active: Color32,
    /// Compatibility alias of `background.subtle` (disabled fills, card icon
    /// boxes); the semantic role is authoritative.
    pub surface_2: Color32,
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,
    /// Focus ring color (`border.focus`); equals `accent` in the shipped themes.
    pub border_focus: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub text_disabled: Color32,
    /// Compatibility alias of `foreground.muted` (hints, status bars) — its own
    /// role: light coincides with `text_tertiary`, dark deliberately differs.
    pub text_muted: Color32,
    pub text_inverse: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_soft: Color32,
    pub accent_foreground: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub overlay: Color32,
    pub code_keyword: Color32,
    pub code_string: Color32,
    pub code_number: Color32,
    pub code_comment: Color32,
    pub code_type: Color32,
    pub code_function: Color32,
    pub code_operator: Color32,
    pub code_punctuation: Color32,
    pub code_variable: Color32,
}

impl Default for DbProTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl DbProTheme {
    pub fn light() -> Self {
        let semantic = SemanticTokens::light();
        Self {
            dark_mode: false,
            reduce_motion: false,
            semantic,
            // Warm Minimalism roles; raw hex lives in `tokens::primitive::light`.
            surface_app: semantic.background.canvas,
            surface_panel: semantic.background.panel,
            surface_elevated: semantic.background.elevated,
            surface_floating: semantic.background.floating,
            surface_editor: semantic.background.editor,
            surface_hover: semantic.background.hover,
            surface_active: semantic.background.selected,
            surface_2: semantic.background.subtle,
            border_subtle: semantic.border.subtle,
            border_default: semantic.border.default,
            border_strong: semantic.border.strong,
            border_focus: semantic.border.focus,
            text_primary: semantic.foreground.primary,
            text_secondary: semantic.foreground.secondary,
            text_tertiary: semantic.foreground.tertiary,
            text_disabled: semantic.foreground.disabled,
            text_muted: semantic.foreground.muted,
            text_inverse: semantic.foreground.inverse,
            accent: semantic.accent.solid,
            accent_hover: semantic.accent.solid_hover,
            accent_soft: semantic.accent.subtle,
            accent_foreground: semantic.accent.foreground,
            success: semantic.status.success.solid,
            warning: semantic.status.warning.solid,
            danger: semantic.status.danger.solid,
            info: semantic.status.info.solid,
            overlay: semantic.background.overlay,
            code_keyword: semantic.syntax.keyword,
            code_string: semantic.syntax.string,
            code_number: semantic.syntax.number,
            code_comment: semantic.syntax.comment,
            code_type: semantic.syntax.type_,
            code_function: semantic.syntax.function,
            code_operator: semantic.syntax.operator,
            code_punctuation: semantic.syntax.punctuation,
            code_variable: semantic.syntax.variable,
        }
    }
    pub fn dark() -> Self {
        let semantic = SemanticTokens::dark();
        Self {
            dark_mode: true,
            reduce_motion: false,
            semantic,
            // Dark workstation roles; raw hex lives in `tokens::primitive::dark`.
            surface_app: semantic.background.canvas,
            surface_panel: semantic.background.panel,
            surface_elevated: semantic.background.elevated,
            surface_floating: semantic.background.floating,
            surface_editor: semantic.background.editor,
            surface_hover: semantic.background.hover,
            surface_active: semantic.background.selected,
            surface_2: semantic.background.subtle,
            border_subtle: semantic.border.subtle,
            border_default: semantic.border.default,
            border_strong: semantic.border.strong,
            border_focus: semantic.border.focus,
            text_primary: semantic.foreground.primary,
            text_secondary: semantic.foreground.secondary,
            text_tertiary: semantic.foreground.tertiary,
            text_disabled: semantic.foreground.disabled,
            text_muted: semantic.foreground.muted,
            text_inverse: semantic.foreground.inverse,
            accent: semantic.accent.solid,
            accent_hover: semantic.accent.solid_hover,
            accent_soft: semantic.accent.subtle,
            accent_foreground: semantic.accent.foreground,
            success: semantic.status.success.solid,
            warning: semantic.status.warning.solid,
            danger: semantic.status.danger.solid,
            info: semantic.status.info.solid,
            overlay: semantic.background.overlay,
            code_keyword: semantic.syntax.keyword,
            code_string: semantic.syntax.string,
            code_number: semantic.syntax.number,
            code_comment: semantic.syntax.comment,
            code_type: semantic.syntax.type_,
            code_function: semantic.syntax.function,
            code_operator: semantic.syntax.operator,
            code_punctuation: semantic.syntax.punctuation,
            code_variable: semantic.syntax.variable,
        }
    }
    /// Subtle tinted fill for badges, diff rows, and status cards (~10-12% opacity).
    /// Recipe owned by the semantic layer so badges and status roles cannot drift.
    pub fn soft_tint(self, color: Color32) -> Color32 {
        subtle_wash(color, self.dark_mode)
    }

    pub fn success_soft(self) -> Color32 {
        self.semantic.status.success.subtle
    }

    pub fn warning_soft(self) -> Color32 {
        self.semantic.status.warning.subtle
    }

    pub fn danger_soft(self) -> Color32 {
        self.semantic.status.danger.subtle
    }

    pub fn info_soft(self) -> Color32 {
        self.semantic.status.info.subtle
    }

    /// Readable foreground for an opaque solid fill, including animated fills.
    pub fn text_on_solid(self, fill: Color32) -> Color32 {
        // Black and white bracket every possible opaque fill: the crossover
        // luminance gives both at least 4.5:1 for normal text.
        if relative_luminance(fill) >= 0.179 {
            Color32::BLACK
        } else {
            Color32::WHITE
        }
    }

    /// Quiet gutter wash behind line numbers (Zed/DBeaver density).
    pub fn editor_gutter_fill(self) -> Color32 {
        if self.dark_mode {
            Color32::from_rgb(20, 20, 20)
        } else {
            Color32::from_rgb(248, 248, 248)
        }
    }

    /// Soft current-line highlight — accent-tinted wash per the Stitch
    /// active-line spec, readable without shouting.
    pub fn editor_current_line_fill(self) -> Color32 {
        self.soft_tint(self.accent)
    }

    /// Selection wash over SQL text.
    pub fn editor_selection_fill(self) -> Color32 {
        if self.dark_mode {
            Color32::from_rgba_unmultiplied(51, 156, 255, 55)
        } else {
            Color32::from_rgba_unmultiplied(2, 133, 255, 42)
        }
    }

    pub fn editor_line_number(self, current: bool) -> Color32 {
        if current {
            self.accent
        } else if self.dark_mode {
            Color32::from_rgb(120, 120, 120)
        } else {
            Color32::from_rgb(128, 128, 128)
        }
    }

    /// UI labels, badges, and controls — Inter Medium (500), matching OpenAI Sans Medium.
    pub fn ui_medium_font(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("ui_medium".into()))
    }

    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lucide".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES)),
        );
        fonts
            .families
            .entry(FontFamily::Name("lucide".into()))
            .or_default()
            .insert(0, "lucide".to_owned());

        fonts.font_data.insert(
            "inter".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_REGULAR)),
        );
        fonts.font_data.insert(
            "inter_ext".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_REGULAR_EXT)),
        );
        fonts.font_data.insert(
            "inter_medium".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_MEDIUM)),
        );
        fonts.font_data.insert(
            "inter_medium_ext".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_MEDIUM_EXT)),
        );

        let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
        proportional.insert(0, "inter_ext".to_owned());
        proportional.insert(0, "inter".to_owned());

        let medium = fonts.families.entry(FontFamily::Name("ui_medium".into())).or_default();
        medium.insert(0, "inter_medium_ext".to_owned());
        medium.insert(0, "inter_medium".to_owned());

        let system_font_paths = [
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFCompact.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        let mut loaded_system_ui = false;
        for path in system_font_paths {
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
            if let Ok(bytes) = std::fs::read(path) {
                fonts.font_data.insert(
                    "system_ui".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .push("system_ui".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Name("ui_medium".into()))
                    .or_default()
                    .push("system_ui".to_owned());
                loaded_system_ui = true;
                break;
            }
        }

        let mono_system_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf",
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/Monaco.ttf",
            "C:\\Windows\\Fonts\\cascadiacode.ttf",
            "C:\\Windows\\Fonts\\consola.ttf",
        ];
        for path in mono_system_paths {
            if let Ok(bytes) = std::fs::read(path) {
                fonts.font_data.insert(
                    "system_mono".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .push("system_mono".to_owned());
                break;
            }
        }

        // CJK / Japanese / Asian font fallback
        let cjk_font_paths = [
            "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf",
            "/usr/share/fonts/opentype/ipafont-gothic/ipagp.ttf",
            "/usr/share/fonts/truetype/fonts-japanese-gothic.ttf",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "C:\\Windows\\Fonts\\msgothic.ttc",
            "C:\\Windows\\Fonts\\meiryo.ttc",
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\yugothr.ttc",
        ];
        for path in cjk_font_paths {
            if let Ok(bytes) = std::fs::read(path) {
                fonts.font_data.insert(
                    "cjk_fallback".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .push("cjk_fallback".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Name("ui_medium".into()))
                    .or_default()
                    .push("cjk_fallback".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .push("cjk_fallback".to_owned());
                break;
            }
        }

        // Fallback for monospace: inter_ext, inter, and system_ui so full Unicode / Vietnamese diacritics / CJK render
        let monospace = fonts.families.entry(FontFamily::Monospace).or_default();
        monospace.push("inter_ext".to_owned());
        monospace.push("inter".to_owned());
        if loaded_system_ui {
            monospace.push("system_ui".to_owned());
        }

        ctx.set_fonts(fonts);
    }

    pub fn apply(self, ctx: &egui::Context) {
        crate::text_selection_style::install(ctx);
        let mut visuals = if self.dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.dark_mode = self.dark_mode;
        visuals.override_text_color = Some(self.text_primary);
        visuals.panel_fill = self.surface_panel;
        visuals.window_fill = self.surface_floating;
        visuals.faint_bg_color = self.surface_app;
        visuals.extreme_bg_color = self.surface_editor;
        visuals.code_bg_color = self.surface_editor;
        visuals.hyperlink_color = self.accent;
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.danger;
        // Text selection uses the same solid blue/white pair in both themes.
        let selection = Self::light();
        visuals.selection.bg_fill = selection.accent_hover;
        visuals.selection.stroke = Stroke::new(STROKE_THIN, selection.accent_foreground);
        visuals.window_corner_radius = CornerRadius::same(RADIUS_EGUI_WINDOW as u8);
        visuals.window_shadow = Shadow {
            offset: [0, SHADOW_OFFSET_Y as i8],
            blur: SHADOW_BLUR as u8,
            spread: 0,
            color: Color32::from_black_alpha(WINDOW_SHADOW_ALPHA),
        };
        visuals.menu_corner_radius = CornerRadius::same(RADIUS_EGUI_MENU as u8);
        visuals.popup_shadow = visuals.window_shadow;
        // Buttons opt into their own emphasis. Bare icon/ghost controls should
        // read as actions in the workspace, not as a wall of outlined fields.
        visuals.button_frame = false;
        visuals.collapsing_header_frame = false;
        visuals.striped = true;
        visuals.widgets.noninteractive.bg_fill = self.surface_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(STROKE_THIN, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(STROKE_THIN, self.text_secondary);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_EGUI_WIDGET as u8);
        visuals.widgets.inactive.bg_fill = self.surface_panel;
        visuals.widgets.inactive.weak_bg_fill = self.surface_panel;
        // Resting inputs stay quiet; hover/focus still provide the interaction boundary.
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.fg_stroke = Stroke::new(STROKE_THIN, self.text_secondary);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_EGUI_WIDGET as u8);
        visuals.widgets.hovered.bg_fill = self.surface_hover;
        visuals.widgets.hovered.weak_bg_fill = self.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(STROKE_THIN, self.border_strong);
        visuals.widgets.hovered.fg_stroke = Stroke::new(STROKE_THIN, self.accent);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_EGUI_WIDGET as u8);
        visuals.widgets.active.bg_fill = self.surface_active;
        visuals.widgets.active.weak_bg_fill = self.surface_active;
        visuals.widgets.active.bg_stroke = Stroke::new(STROKE_THIN, self.accent_hover);
        visuals.widgets.active.fg_stroke = Stroke::new(STROKE_THIN, self.accent);
        visuals.widgets.active.corner_radius = CornerRadius::same(RADIUS_EGUI_WIDGET as u8);
        visuals.widgets.open.bg_fill = self.surface_hover;
        visuals.widgets.open.weak_bg_fill = self.surface_hover;
        visuals.widgets.open.bg_stroke = Stroke::new(STROKE_THIN, self.border_strong);
        visuals.widgets.open.fg_stroke = Stroke::new(STROKE_THIN, self.text_primary);
        visuals.widgets.open.corner_radius = CornerRadius::same(RADIUS_EGUI_WIDGET as u8);
        visuals.window_stroke = Stroke::new(STROKE_THIN, self.border_subtle);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = EGUI_ITEM_SPACING;
        style.spacing.button_padding = EGUI_BUTTON_PADDING;
        style.spacing.interact_size = EGUI_INTERACT_SIZE;
        style.spacing.window_margin = Margin::same(EGUI_WINDOW_MARGIN as i8);
        style.spacing.menu_margin = Margin::same(EGUI_MENU_MARGIN as i8);
        style.spacing.indent = EGUI_INDENT;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(FONT_SIZE_UI_LABEL));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(FONT_SIZE_UI_LABEL));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(FONT_SIZE_BADGE));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(FONT_SIZE_MONO_UI));
        ctx.set_global_style(style);
    }

    pub fn floating_shadow(self) -> Shadow {
        Shadow {
            offset: [0, SHADOW_OFFSET_Y as i8],
            blur: SHADOW_BLUR as u8,
            spread: 0,
            color: Color32::from_black_alpha(if self.dark_mode {
                SHADOW_ALPHA_DARK
            } else {
                SHADOW_ALPHA_LIGHT
            }),
        }
    }
}

pub(crate) fn relative_luminance(color: Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

#[cfg(test)]
mod tests {
    use super::DbProTheme;

    #[test]
    fn light_and_dark_themes_keep_distinct_surface_tokens() {
        let light = DbProTheme::light();
        let dark = DbProTheme::dark();

        assert!(!light.dark_mode);
        assert!(dark.dark_mode);
        assert_ne!(light.surface_app, dark.surface_app);
        assert_ne!(light.text_primary, dark.text_primary);
    }

    #[test]
    fn light_tokens_follow_codex_surface_contract() {
        let theme = DbProTheme::light();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(255, 255, 255));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(249, 249, 249));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(238, 238, 238));
        assert_eq!(theme.text_primary, egui::Color32::from_rgb(26, 28, 31));
        assert_eq!(theme.accent, egui::Color32::from_rgb(0, 111, 204));
    }

    #[test]
    fn dark_tokens_follow_codex_surface_contract() {
        let theme = DbProTheme::dark();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(24, 24, 24));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(33, 33, 33));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(48, 48, 48));
        assert_eq!(theme.text_primary, egui::Color32::from_rgb(223, 223, 223));
        assert_eq!(theme.accent, egui::Color32::from_rgb(51, 156, 255));
        assert_eq!(theme.accent_foreground, egui::Color32::from_rgb(18, 18, 18));
    }

    #[test]
    fn theme_text_and_accent_foregrounds_meet_normal_text_contrast() {
        for theme in [DbProTheme::light(), DbProTheme::dark()] {
            for foreground in [theme.text_primary, theme.text_secondary, theme.text_tertiary] {
                assert!(
                    contrast_ratio(foreground, theme.surface_app) >= 4.5,
                    "{foreground:?} on app {:?}",
                    theme.surface_app
                );
                assert!(
                    contrast_ratio(foreground, theme.surface_panel) >= 4.5,
                    "{foreground:?} on panel {:?}",
                    theme.surface_panel
                );
            }
            assert!(
                contrast_ratio(theme.accent_foreground, theme.accent) >= 4.5,
                "accent foreground {:?} on {:?}",
                theme.accent_foreground,
                theme.accent
            );
        }
    }

    fn contrast_ratio(left: egui::Color32, right: egui::Color32) -> f32 {
        let (lighter, darker) = if super::relative_luminance(left) >= super::relative_luminance(right) {
            (left, right)
        } else {
            (right, left)
        };
        (super::relative_luminance(lighter) + 0.05) / (super::relative_luminance(darker) + 0.05)
    }

    #[test]
    fn flat_facade_fields_are_built_from_the_semantic_roles() {
        for theme in [DbProTheme::light(), DbProTheme::dark()] {
            assert_eq!(theme.surface_app, theme.semantic.background.canvas);
            assert_eq!(theme.surface_2, theme.semantic.background.subtle);
            assert_eq!(theme.surface_active, theme.semantic.background.selected);
            assert_eq!(theme.border_focus, theme.semantic.border.focus);
            assert_eq!(theme.text_muted, theme.semantic.foreground.muted);
            assert_eq!(theme.accent, theme.semantic.accent.solid);
            assert_eq!(theme.success, theme.semantic.status.success.solid);
            assert_eq!(theme.success_soft(), theme.semantic.status.success.subtle);
            assert_eq!(theme.overlay, theme.semantic.background.overlay);
            // Focus ring color stays the accent color in both shipped themes.
            assert_eq!(theme.border_focus, theme.accent);
        }
    }

    #[test]
    fn soft_tint_matches_the_semantic_wash_recipe() {
        let light = DbProTheme::light();
        let dark = DbProTheme::dark();
        assert_eq!(light.soft_tint(light.danger), light.semantic.status.danger.subtle);
        assert_eq!(dark.soft_tint(dark.warning), dark.semantic.status.warning.subtle);
    }

    #[test]
    fn install_fonts_registers_inter_medium_for_ui_labels() {
        let ctx = egui::Context::default();
        super::DbProTheme::install_fonts(&ctx);
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "GPT-4o".to_owned(),
                    super::DbProTheme::ui_medium_font(12.0),
                    egui::Color32::WHITE,
                );
                assert!(galley.size().x > 8.0);
                assert!(galley.size().y > 8.0);

                let mono_vietnamese = ui.painter().layout_no_wrap(
                    "SELECT * FROM bảng_dữ_liệu WHERE tên = 'tiếng Việt'".to_owned(),
                    egui::FontId::monospace(13.0),
                    egui::Color32::WHITE,
                );
                assert!(mono_vietnamese.size().x > 50.0);
            });
        });
    }
}
