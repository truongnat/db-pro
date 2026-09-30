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
        Self {
            dark_mode: false,
            // Open-api-style.md Warm Minimalism Light Tokens:
            surface_app: Color32::from_rgb(255, 255, 255), // --background: #ffffff
            surface_panel: Color32::from_rgb(247, 247, 247), // --surface: #f7f7f7
            surface_elevated: Color32::from_rgb(255, 255, 255), // card / white surface
            surface_floating: Color32::from_rgb(255, 255, 255), // dialog / popover
            surface_editor: Color32::from_rgb(255, 255, 255), // flush with app — Zed/DBeaver blank buffer
            surface_hover: Color32::from_rgb(238, 238, 238), // --surface-hover: #eeeeee
            surface_active: Color32::from_rgb(232, 232, 232), // --surface-active: #e8e8e8
            surface_2: Color32::from_rgb(243, 243, 243),   // --surface-2: #f3f3f3
            border_subtle: Color32::from_rgb(238, 238, 238), // --border-subtle: #eeeeee
            border_default: Color32::from_rgb(226, 226, 226), // --border-default: #e2e2e2
            border_strong: Color32::from_rgb(210, 210, 210), // --border-strong: #d2d2d2
            text_primary: Color32::from_rgb(13, 13, 13),   // --text-primary: #0d0d0d
            text_secondary: Color32::from_rgb(95, 95, 95), // --text-secondary: #5f5f5f
            text_tertiary: Color32::from_rgb(138, 138, 138), // --text-tertiary: #8a8a8a
            text_disabled: Color32::from_rgb(148, 148, 148), // --text-disabled: #949494
            text_muted: Color32::from_rgb(138, 138, 138),  // alias to tertiary
            text_inverse: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(2, 133, 255), // --accent: #0285ff (Modern Blue)
            accent_hover: Color32::from_rgb(1, 105, 204),
            accent_soft: Color32::from_rgb(230, 242, 255), // slightly stronger for completion selection
            accent_foreground: Color32::from_rgb(255, 255, 255), // #ffffff
            success: Color32::from_rgb(22, 163, 74),       // --success: #16a34a
            warning: Color32::from_rgb(217, 119, 6),       // --warning: #d97706
            danger: Color32::from_rgb(220, 38, 38),        // --danger: #dc2626
            info: Color32::from_rgb(37, 99, 235),          // --info: #2563eb
            overlay: Color32::from_black_alpha(38),        // scrim ~0.15 so the dialog stays the brightest surface
            // SQL syntax — restrained Zed-like light palette (not UI accent clones).
            code_keyword: Color32::from_rgb(55, 65, 180),
            code_string: Color32::from_rgb(15, 118, 70),
            code_number: Color32::from_rgb(180, 83, 9),
            code_comment: Color32::from_rgb(120, 120, 120),
            code_type: Color32::from_rgb(126, 34, 206),
            code_function: Color32::from_rgb(14, 116, 144),
            code_operator: Color32::from_rgb(100, 100, 110),
            code_punctuation: Color32::from_rgb(95, 95, 95),
            code_variable: Color32::from_rgb(24, 24, 27),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            // Dark database workstation tokens: quiet charcoal surfaces, crisp borders, and a restrained blue action accent.
            surface_app: Color32::from_rgb(23, 25, 28),       // #17191c
            surface_panel: Color32::from_rgb(29, 32, 36),     // #1d2024
            surface_elevated: Color32::from_rgb(35, 39, 45),  // #23272d
            surface_floating: Color32::from_rgb(32, 36, 42),  // #20242a
            surface_editor: Color32::from_rgb(21, 23, 25),    // #151719
            surface_hover: Color32::from_rgb(40, 46, 53),     // #282e35
            surface_active: Color32::from_rgb(48, 57, 70),    // #303946
            surface_2: Color32::from_rgb(36, 41, 47),         // #24292f
            border_subtle: Color32::from_rgb(42, 48, 55),     // #2a3037
            border_default: Color32::from_rgb(56, 65, 75),    // #38414b
            border_strong: Color32::from_rgb(75, 88, 101),    // #4b5865
            text_primary: Color32::from_rgb(241, 243, 245),   // #f1f3f5
            text_secondary: Color32::from_rgb(180, 187, 196), // #b4bbc4
            text_tertiary: Color32::from_rgb(130, 140, 151),  // #828c97
            text_disabled: Color32::from_rgb(106, 116, 128),  // #6a7480
            text_muted: Color32::from_rgb(140, 150, 160),     // #8c96a0
            text_inverse: Color32::from_rgb(17, 19, 22),      // #111316
            accent: Color32::from_rgb(79, 140, 255),          // #4f8cff
            accent_hover: Color32::from_rgb(106, 160, 255),   // #6aa0ff
            accent_soft: Color32::from_rgb(27, 49, 88),       // #1b3158
            accent_foreground: Color32::from_rgb(255, 255, 255),
            success: Color32::from_rgb(58, 197, 121), // #3ac579
            warning: Color32::from_rgb(242, 180, 90), // #f2b45a
            danger: Color32::from_rgb(239, 107, 115), // #ef6b73
            info: Color32::from_rgb(88, 166, 255),    // #58a6ff
            overlay: Color32::from_black_alpha(74),
            code_keyword: Color32::from_rgb(199, 146, 234), // #C792EA (synKeyword)
            code_string: Color32::from_rgb(195, 232, 141),  // #C3E88D (synString)
            code_number: Color32::from_rgb(247, 140, 108),  // #F78C6C (synConst)
            code_comment: Color32::from_rgb(103, 110, 149), // #676E95 (synComment)
            code_type: Color32::from_rgb(255, 203, 107),    // #FFCB6B (synType)
            code_function: Color32::from_rgb(130, 170, 255), // #82AAFF (synFunc)
            code_operator: Color32::from_rgb(137, 221, 255), // #89DDFF (synPunct)
            code_punctuation: Color32::from_rgb(137, 221, 255), // #89DDFF (synPunct)
            code_variable: Color32::from_rgb(238, 255, 255), // #EEFFFF (synIdent)
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

    /// Quiet gutter wash behind line numbers (Zed/DBeaver density).
    pub fn editor_gutter_fill(self) -> Color32 {
        if self.dark_mode {
            Color32::from_rgb(22, 22, 22)
        } else {
            Color32::from_rgb(248, 248, 248)
        }
    }

    /// Soft current-line highlight — readable without shouting.
    pub fn editor_current_line_fill(self) -> Color32 {
        if self.dark_mode {
            Color32::from_rgba_unmultiplied(255, 255, 255, 22)
        } else {
            Color32::from_rgba_unmultiplied(15, 23, 42, 16)
        }
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
            self.text_secondary
        } else if self.dark_mode {
            Color32::from_rgb(120, 120, 120)
        } else {
            Color32::from_rgb(170, 170, 170)
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
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        let mut loaded_system_ui = false;
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
                fonts
                    .font_data
                    .insert("system_mono".to_owned(), egui::FontData::from_owned(bytes));
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
                fonts
                    .font_data
                    .insert("cjk_fallback".to_owned(), egui::FontData::from_owned(bytes));
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
        // Selection stays a muted blue wash so the active cell is visible without
        // turning a dense result grid into a wall of saturated color.
        visuals.selection.bg_fill = self.accent_soft;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        visuals.window_rounding = Rounding::same(6.0);
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
        assert_eq!(theme.accent, egui::Color32::from_rgb(2, 133, 255));
    }

    #[test]
    fn dark_tokens_follow_database_workstation_surface_contract() {
        let theme = DbProTheme::dark();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(23, 25, 28));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(29, 32, 36));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(48, 57, 70));
        assert_eq!(theme.accent, egui::Color32::from_rgb(79, 140, 255));
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
