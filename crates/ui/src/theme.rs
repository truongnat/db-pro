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
            code_keyword: Color32::from_rgb(243, 243, 243),
            code_string: Color32::from_rgb(34, 197, 94),
            code_number: Color32::from_rgb(245, 158, 11),
            code_comment: Color32::from_rgb(141, 141, 141),
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

        let system_font_paths = [
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        for path in system_font_paths {
            if let Ok(bytes) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("system_font".to_owned(), egui::FontData::from_owned(bytes));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "system_font".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("system_font".to_owned());
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
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.weak_bg_fill = self.accent;
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
