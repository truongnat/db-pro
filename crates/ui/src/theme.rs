use egui::{Color32, FontFamily, FontId, Margin, Rounding, Shadow, Stroke, TextStyle, Visuals};

const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const INTER_REGULAR_EXT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular-ext.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
const INTER_MEDIUM_EXT: &[u8] = include_bytes!("../assets/fonts/Inter-Medium-ext.ttf");

/// Product-owned visual tokens for the native shell.
#[derive(Debug, Clone, Copy)]
pub struct DbProTheme {
    pub dark_mode: bool,
    pub surface_app: Color32,
    pub surface_panel: Color32,
    pub surface_elevated: Color32,
    pub surface_floating: Color32,
    pub surface_editor: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub surface_2: Color32,
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub text_disabled: Color32,
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
}

impl Default for DbProTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl DbProTheme {
    pub fn light() -> Self {
        Self {
            dark_mode: false,
            // Open-api-style.md Warm Minimalism Light Tokens:
            surface_app: Color32::from_rgb(255, 255, 255), // --background: #ffffff
            surface_panel: Color32::from_rgb(247, 247, 247), // --surface: #f7f7f7
            surface_elevated: Color32::from_rgb(255, 255, 255), // card / white surface
            surface_floating: Color32::from_rgb(255, 255, 255), // dialog / popover
            surface_editor: Color32::from_rgb(250, 250, 250), // --background-subtle: #fafafa
            surface_hover: Color32::from_rgb(238, 238, 238), // --surface-hover: #eeeeee
            surface_active: Color32::from_rgb(232, 232, 232), // --surface-active: #e8e8e8
            surface_2: Color32::from_rgb(243, 243, 243),   // --surface-2: #f3f3f3
            border_subtle: Color32::from_rgb(238, 238, 238), // --border-subtle: #eeeeee
            border_default: Color32::from_rgb(226, 226, 226), // --border-default: #e2e2e2
            border_strong: Color32::from_rgb(210, 210, 210), // --border-strong: #d2d2d2
            text_primary: Color32::from_rgb(13, 13, 13),   // --text-primary: #0d0d0d
            text_secondary: Color32::from_rgb(95, 95, 95), // --text-secondary: #5f5f5f
            text_tertiary: Color32::from_rgb(138, 138, 138), // --text-tertiary: #8a8a8a
            text_disabled: Color32::from_rgb(179, 179, 179), // --text-disabled: #b3b3b3
            text_muted: Color32::from_rgb(138, 138, 138),  // alias to tertiary
            text_inverse: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(17, 17, 17), // --accent: #111111 (Editorial Black)
            accent_hover: Color32::from_rgb(34, 34, 34),
            accent_soft: Color32::from_rgb(243, 243, 243), // --surface-2
            accent_foreground: Color32::from_rgb(255, 255, 255), // #ffffff
            success: Color32::from_rgb(22, 163, 74),       // --success: #16a34a
            warning: Color32::from_rgb(217, 119, 6),       // --warning: #d97706
            danger: Color32::from_rgb(220, 38, 38),        // --danger: #dc2626
            info: Color32::from_rgb(37, 99, 235),          // --info: #2563eb
            overlay: Color32::from_black_alpha(38),        // scrim ~0.15 so the dialog stays the brightest surface
            code_keyword: Color32::from_rgb(17, 17, 17),
            code_string: Color32::from_rgb(22, 163, 74),
            code_number: Color32::from_rgb(217, 119, 6),
            code_comment: Color32::from_rgb(138, 138, 138),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            // Open-api-style.md Warm Minimalism Dark Tokens:
            surface_app: Color32::from_rgb(33, 33, 33), // --background: #212121
            surface_panel: Color32::from_rgb(42, 42, 42), // --surface: #2a2a2a
            surface_elevated: Color32::from_rgb(48, 48, 48), // --surface-2: #303030
            surface_floating: Color32::from_rgb(42, 42, 42), // dialog / popover
            surface_editor: Color32::from_rgb(28, 28, 28), // --background-subtle: #1c1c1c
            surface_hover: Color32::from_rgb(54, 54, 54), // --surface-hover: #363636
            surface_active: Color32::from_rgb(61, 61, 61), // --surface-active: #3d3d3d
            surface_2: Color32::from_rgb(48, 48, 48),   // --surface-2: #303030
            border_subtle: Color32::from_rgb(50, 50, 50), // --border-subtle: #323232
            border_default: Color32::from_rgb(65, 65, 65), // --border-default: #414141
            border_strong: Color32::from_rgb(80, 80, 80), // --border-strong: #505050
            text_primary: Color32::from_rgb(236, 236, 236), // --text-primary: #ececec
            text_secondary: Color32::from_rgb(185, 185, 185), // --text-secondary: #b9b9b9
            text_tertiary: Color32::from_rgb(141, 141, 141), // --text-tertiary: #8d8d8d
            text_disabled: Color32::from_rgb(102, 102, 102), // --text-disabled: #666666
            text_muted: Color32::from_rgb(141, 141, 141), // alias to tertiary
            text_inverse: Color32::from_rgb(17, 17, 17),
            accent: Color32::from_rgb(243, 243, 243), // --accent: #f3f3f3 (Editorial Light)
            accent_hover: Color32::from_rgb(255, 255, 255),
            accent_soft: Color32::from_rgb(48, 48, 48),       // --surface-2
            accent_foreground: Color32::from_rgb(17, 17, 17), // #111111
            success: Color32::from_rgb(34, 197, 94),          // --success: #22c55e
            warning: Color32::from_rgb(245, 158, 11),         // --warning: #f59e0b
            danger: Color32::from_rgb(239, 68, 68),           // --danger: #ef4444
            info: Color32::from_rgb(59, 130, 246),            // --info: #3b82f6
            overlay: Color32::from_black_alpha(64),           // scrim ~0.25, card stays fully opaque on top
            code_keyword: Color32::from_rgb(243, 243, 243),
            code_string: Color32::from_rgb(34, 197, 94),
            code_number: Color32::from_rgb(245, 158, 11),
            code_comment: Color32::from_rgb(141, 141, 141),
        }
    }
    /// Subtle tinted fill for badges, diff rows, and status cards (~10-12% opacity).
    pub fn soft_tint(self, color: Color32) -> Color32 {
        if self.dark_mode {
            Color32::from_rgba_premultiplied(
                (color.r() as f32 * 0.12) as u8,
                (color.g() as f32 * 0.12) as u8,
                (color.b() as f32 * 0.12) as u8,
                25,
            )
        } else {
            Color32::from_rgba_premultiplied(
                (color.r() as f32 * 0.08) as u8,
                (color.g() as f32 * 0.08) as u8,
                (color.b() as f32 * 0.08) as u8,
                18,
            )
        }
    }

    pub fn success_soft(self) -> Color32 {
        self.soft_tint(self.success)
    }

    pub fn warning_soft(self) -> Color32 {
        self.soft_tint(self.warning)
    }

    pub fn danger_soft(self) -> Color32 {
        self.soft_tint(self.danger)
    }

    pub fn info_soft(self) -> Color32 {
        self.soft_tint(self.info)
    }

    /// UI labels, badges, and controls — Inter Medium (500), matching OpenAI Sans Medium.
    pub fn ui_medium_font(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name("ui_medium".into()))
    }

    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lucide".to_owned(),
            egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES),
        );
        fonts
            .families
            .entry(FontFamily::Name("lucide".into()))
            .or_default()
            .insert(0, "lucide".to_owned());

        fonts
            .font_data
            .insert("inter".to_owned(), egui::FontData::from_static(INTER_REGULAR));
        fonts
            .font_data
            .insert("inter_ext".to_owned(), egui::FontData::from_static(INTER_REGULAR_EXT));
        fonts
            .font_data
            .insert("inter_medium".to_owned(), egui::FontData::from_static(INTER_MEDIUM));
        fonts.font_data.insert(
            "inter_medium_ext".to_owned(),
            egui::FontData::from_static(INTER_MEDIUM_EXT),
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
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        for path in system_font_paths {
            if let Ok(bytes) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("system_ui".to_owned(), egui::FontData::from_owned(bytes));
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
                break;
            }
        }

        ctx.set_fonts(fonts);
    }

    pub fn apply(self, ctx: &egui::Context) {
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
        // Selected rows/tabs in Codex stay neutral; blue is reserved for the
        // interaction accent and the active indicator rather than large fills.
        visuals.selection.bg_fill = self.surface_active;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        visuals.window_rounding = Rounding::same(7.0);
        visuals.window_shadow = Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(28),
        };
        visuals.menu_rounding = Rounding::same(6.0);
        visuals.popup_shadow = visuals.window_shadow;
        // Buttons opt into their own emphasis. Bare icon/ghost controls should
        // read as actions in the workspace, not as a wall of outlined fields.
        visuals.button_frame = false;
        visuals.collapsing_header_frame = false;
        visuals.striped = true;
        visuals.widgets.noninteractive.bg_fill = self.surface_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
        visuals.widgets.inactive.bg_fill = self.surface_panel;
        visuals.widgets.inactive.weak_bg_fill = self.surface_panel;
        // Resting inputs stay quiet; hover/focus still provide the interaction boundary.
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.inactive.rounding = Rounding::same(4.0);
        visuals.widgets.hovered.bg_fill = self.surface_hover;
        visuals.widgets.hovered.weak_bg_fill = self.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.border_strong);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        visuals.widgets.active.bg_fill = self.surface_active;
        visuals.widgets.active.weak_bg_fill = self.surface_active;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent_hover);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.active.rounding = Rounding::same(4.0);
        visuals.widgets.open.bg_fill = self.surface_hover;
        visuals.widgets.open.weak_bg_fill = self.surface_hover;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.border_strong);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.open.rounding = Rounding::same(4.0);
        visuals.window_stroke = Stroke::new(1.0, self.border_subtle);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size = egui::vec2(24.0, 24.0);
        style.spacing.window_margin = Margin::same(12.0);
        style.spacing.menu_margin = Margin::same(5.0);
        style.spacing.indent = 13.0;
        style.text_styles.insert(TextStyle::Body, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Button, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));
        style.text_styles.insert(TextStyle::Monospace, FontId::monospace(13.0));
        ctx.set_style(style);
    }

    pub fn floating_shadow(self) -> Shadow {
        Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(if self.dark_mode { 40 } else { 20 }),
        }
    }
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
    fn light_tokens_follow_warm_minimalism_surface_contract() {
        let theme = DbProTheme::light();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(255, 255, 255));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(247, 247, 247));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(232, 232, 232));
        assert_eq!(theme.accent, egui::Color32::from_rgb(17, 17, 17));
    }

    #[test]
    fn dark_tokens_follow_warm_minimalism_surface_contract() {
        let theme = DbProTheme::dark();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(33, 33, 33));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(42, 42, 42));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(61, 61, 61));
        assert_eq!(theme.accent, egui::Color32::from_rgb(243, 243, 243));
    }

    #[test]
    fn install_fonts_registers_inter_medium_for_ui_labels() {
        let ctx = egui::Context::default();
        super::DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "GPT-4o".to_owned(),
                    super::DbProTheme::ui_medium_font(12.0),
                    egui::Color32::WHITE,
                );
                assert!(galley.size().x > 8.0);
                assert!(galley.size().y > 8.0);
            });
        });
    }
}
