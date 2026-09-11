use egui::{Color32, FontId, Margin, Rounding, Shadow, Stroke, TextStyle, Visuals};

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
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
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
    pub code_keyword: Color32,
    pub code_string: Color32,
    pub code_number: Color32,
    pub code_comment: Color32,
}

impl Default for DbProTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl DbProTheme {
    pub fn light() -> Self {
        Self {
            dark_mode: false,
            surface_app: Color32::from_rgb(249, 249, 249),
            surface_panel: Color32::from_rgb(255, 255, 255),
            surface_elevated: Color32::from_rgb(250, 250, 250),
            surface_floating: Color32::from_rgb(255, 255, 255),
            surface_editor: Color32::from_rgb(247, 247, 247),
            surface_hover: Color32::from_rgb(242, 242, 242),
            surface_active: Color32::from_rgb(242, 242, 242),
            border_subtle: Color32::from_rgb(242, 242, 242),
            border_default: Color32::from_rgb(235, 235, 235),
            border_strong: Color32::from_rgb(225, 225, 225),
            text_primary: Color32::from_rgb(26, 28, 31),
            text_secondary: Color32::from_rgb(93, 93, 93),
            text_muted: Color32::from_rgb(143, 143, 143),
            text_inverse: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(2, 133, 255),
            accent_hover: Color32::from_rgb(1, 105, 204),
            accent_soft: Color32::from_rgb(229, 243, 255),
            accent_foreground: Color32::from_rgb(255, 255, 255),
            success: Color32::from_rgb(0, 162, 64),
            warning: Color32::from_rgb(226, 85, 7),
            danger: Color32::from_rgb(224, 46, 42),
            info: Color32::from_rgb(51, 156, 255),
            code_keyword: Color32::from_rgb(213, 53, 56),
            code_string: Color32::from_rgb(0, 136, 9),
            code_number: Color32::from_rgb(0, 113, 234),
            code_comment: Color32::from_rgb(102, 102, 102),
        }
    }

    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            surface_app: Color32::from_rgb(24, 24, 24),
            surface_panel: Color32::from_rgb(33, 33, 33),
            surface_elevated: Color32::from_rgb(40, 40, 40),
            surface_floating: Color32::from_rgb(48, 48, 48),
            surface_editor: Color32::from_rgb(33, 33, 33),
            surface_hover: Color32::from_rgb(48, 48, 48),
            surface_active: Color32::from_rgb(57, 57, 57),
            border_subtle: Color32::from_rgb(43, 43, 43),
            border_default: Color32::from_rgb(52, 52, 52),
            border_strong: Color32::from_rgb(65, 65, 65),
            text_primary: Color32::from_rgb(223, 223, 223),
            text_secondary: Color32::from_rgb(179, 179, 179),
            text_muted: Color32::from_rgb(153, 153, 153),
            text_inverse: Color32::from_rgb(13, 13, 13),
            accent: Color32::from_rgb(51, 156, 255),
            accent_hover: Color32::from_rgb(102, 181, 255),
            accent_soft: Color32::from_rgb(0, 40, 77),
            accent_foreground: Color32::from_rgb(13, 13, 13),
            success: Color32::from_rgb(64, 201, 119),
            warning: Color32::from_rgb(255, 133, 73),
            danger: Color32::from_rgb(255, 103, 100),
            info: Color32::from_rgb(51, 156, 255),
            code_keyword: Color32::from_rgb(246, 117, 118),
            code_string: Color32::from_rgb(133, 223, 123),
            code_number: Color32::from_rgb(109, 203, 244),
            code_comment: Color32::from_rgb(153, 153, 153),
        }
    }

    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lucide".to_owned(),
            egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES),
        );
        fonts
            .families
            .entry(egui::FontFamily::Name("lucide".into()))
            .or_default()
            .insert(0, "lucide".to_owned());
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
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        visuals.widgets.active.bg_fill = self.surface_active;
        visuals.widgets.active.weak_bg_fill = self.surface_active;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent_hover);
        // egui uses the active foreground for `RichText::strong()` too; keep
        // headings readable and let accent buttons opt into their own color.
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
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
    fn light_tokens_follow_codex_neutral_surface_contract() {
        let theme = DbProTheme::light();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(249, 249, 249));
        assert_eq!(theme.surface_panel, egui::Color32::WHITE);
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(242, 242, 242));
        assert_eq!(theme.accent, egui::Color32::from_rgb(2, 133, 255));
    }

    #[test]
    fn dark_tokens_follow_codex_neutral_surface_contract() {
        let theme = DbProTheme::dark();

        assert_eq!(theme.surface_app, egui::Color32::from_rgb(24, 24, 24));
        assert_eq!(theme.surface_panel, egui::Color32::from_rgb(33, 33, 33));
        assert_eq!(theme.surface_active, egui::Color32::from_rgb(57, 57, 57));
        assert_eq!(theme.accent, egui::Color32::from_rgb(51, 156, 255));
    }
}
