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
            surface_app: Color32::from_rgb(247, 248, 250),
            surface_panel: Color32::from_rgb(255, 255, 255),
            surface_elevated: Color32::from_rgb(255, 255, 255),
            surface_floating: Color32::from_rgb(255, 255, 255),
            surface_editor: Color32::from_rgb(252, 252, 253),
            surface_hover: Color32::from_rgb(245, 246, 249),
            surface_active: Color32::from_rgb(239, 237, 255),
            border_subtle: Color32::from_rgb(235, 237, 241),
            border_default: Color32::from_rgb(221, 224, 231),
            border_strong: Color32::from_rgb(195, 199, 209),
            text_primary: Color32::from_rgb(31, 35, 40),
            text_secondary: Color32::from_rgb(88, 96, 105),
            text_muted: Color32::from_rgb(128, 137, 148),
            text_inverse: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(109, 94, 245),
            accent_hover: Color32::from_rgb(91, 75, 232),
            accent_soft: Color32::from_rgb(240, 238, 255),
            accent_foreground: Color32::from_rgb(255, 255, 255),
            success: Color32::from_rgb(25, 135, 84),
            warning: Color32::from_rgb(154, 103, 0),
            danger: Color32::from_rgb(197, 57, 82),
            info: Color32::from_rgb(58, 105, 199),
            code_keyword: Color32::from_rgb(91, 75, 232),
            code_string: Color32::from_rgb(154, 103, 0),
            code_number: Color32::from_rgb(25, 135, 84),
            code_comment: Color32::from_rgb(128, 137, 148),
        }
    }

    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            surface_app: Color32::from_rgb(14, 16, 21),
            surface_panel: Color32::from_rgb(19, 22, 29),
            surface_elevated: Color32::from_rgb(25, 29, 38),
            surface_floating: Color32::from_rgb(29, 34, 44),
            surface_editor: Color32::from_rgb(16, 19, 25),
            surface_hover: Color32::from_rgb(34, 40, 51),
            surface_active: Color32::from_rgb(43, 36, 73),
            border_subtle: Color32::from_rgb(35, 40, 50),
            border_default: Color32::from_rgb(52, 58, 70),
            border_strong: Color32::from_rgb(76, 84, 100),
            text_primary: Color32::from_rgb(238, 240, 245),
            text_secondary: Color32::from_rgb(178, 184, 196),
            text_muted: Color32::from_rgb(117, 125, 141),
            text_inverse: Color32::from_rgb(14, 16, 21),
            accent: Color32::from_rgb(168, 148, 255),
            accent_hover: Color32::from_rgb(193, 178, 255),
            accent_soft: Color32::from_rgb(43, 36, 73),
            accent_foreground: Color32::from_rgb(21, 17, 36),
            success: Color32::from_rgb(92, 207, 150),
            warning: Color32::from_rgb(242, 190, 91),
            danger: Color32::from_rgb(242, 122, 145),
            info: Color32::from_rgb(128, 170, 255),
            code_keyword: Color32::from_rgb(194, 177, 255),
            code_string: Color32::from_rgb(242, 190, 91),
            code_number: Color32::from_rgb(107, 220, 164),
            code_comment: Color32::from_rgb(122, 131, 147),
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
        visuals.selection.bg_fill = self.accent.linear_multiply(0.16);
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        visuals.window_rounding = Rounding::same(9.0);
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
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.weak_bg_fill = self.accent;
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
        style.spacing.item_spacing = egui::vec2(8.0, 5.0);
        style.spacing.button_padding = egui::vec2(9.0, 5.0);
        style.spacing.interact_size = egui::vec2(24.0, 24.0);
        style.spacing.window_margin = Margin::same(14.0);
        style.spacing.menu_margin = Margin::same(5.0);
        style.spacing.indent = 14.0;
        style.text_styles.insert(TextStyle::Body, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Button, FontId::proportional(12.5));
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
}
