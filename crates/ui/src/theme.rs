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
        Self::light()
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
            surface_app: Color32::from_rgb(24, 25, 29),
            surface_panel: Color32::from_rgb(31, 32, 38),
            surface_elevated: Color32::from_rgb(37, 38, 45),
            surface_floating: Color32::from_rgb(37, 38, 45),
            surface_editor: Color32::from_rgb(27, 28, 33),
            surface_hover: Color32::from_rgb(44, 45, 53),
            surface_active: Color32::from_rgb(56, 50, 99),
            border_subtle: Color32::from_rgb(48, 49, 57),
            border_default: Color32::from_rgb(65, 66, 77),
            border_strong: Color32::from_rgb(89, 90, 104),
            text_primary: Color32::from_rgb(245, 245, 247),
            text_secondary: Color32::from_rgb(192, 193, 200),
            text_muted: Color32::from_rgb(149, 151, 163),
            text_inverse: Color32::from_rgb(24, 25, 29),
            accent: Color32::from_rgb(155, 140, 255),
            accent_hover: Color32::from_rgb(180, 170, 255),
            accent_soft: Color32::from_rgb(56, 50, 99),
            accent_foreground: Color32::from_rgb(24, 21, 40),
            success: Color32::from_rgb(88, 201, 145),
            warning: Color32::from_rgb(240, 189, 92),
            danger: Color32::from_rgb(241, 125, 147),
            info: Color32::from_rgb(128, 166, 255),
            code_keyword: Color32::from_rgb(183, 170, 255),
            code_string: Color32::from_rgb(240, 189, 92),
            code_number: Color32::from_rgb(103, 217, 160),
            code_comment: Color32::from_rgb(133, 136, 150),
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
        visuals.window_rounding = Rounding::same(10.0);
        visuals.window_shadow = Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(28),
        };
        visuals.menu_rounding = Rounding::same(8.0);
        visuals.popup_shadow = visuals.window_shadow;
        visuals.button_frame = true;
        visuals.collapsing_header_frame = false;
        visuals.striped = true;
        visuals.widgets.noninteractive.bg_fill = self.surface_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        visuals.widgets.inactive.bg_fill = self.surface_panel;
        visuals.widgets.inactive.weak_bg_fill = self.surface_panel;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_default);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.bg_fill = self.surface_hover;
        visuals.widgets.hovered.weak_bg_fill = self.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.border_strong);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.weak_bg_fill = self.accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent_hover);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.accent_foreground);
        visuals.widgets.active.rounding = Rounding::same(6.0);
        visuals.widgets.open.bg_fill = self.surface_hover;
        visuals.widgets.open.weak_bg_fill = self.surface_hover;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.border_strong);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.open.rounding = Rounding::same(6.0);
        visuals.window_stroke = Stroke::new(1.0, self.border_default);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.interact_size = egui::vec2(24.0, 24.0);
        style.spacing.window_margin = Margin::same(16.0);
        style.spacing.menu_margin = Margin::same(6.0);
        style.spacing.indent = 16.0;
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
}
