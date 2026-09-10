use egui::{Color32, Stroke, Style, Visuals};

/// Product-owned visual tokens for the native shell.
#[derive(Debug, Clone, Copy)]
pub struct DbProTheme {
    pub surface_app: Color32,
    pub surface_panel: Color32,
    pub surface_elevated: Color32,
    pub surface_hover: Color32,
    pub border_subtle: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

impl Default for DbProTheme {
    fn default() -> Self {
        Self {
            surface_app: Color32::from_rgb(11, 13, 16),
            surface_panel: Color32::from_rgb(17, 21, 26),
            surface_elevated: Color32::from_rgb(23, 28, 35),
            surface_hover: Color32::from_rgb(29, 37, 48),
            border_subtle: Color32::from_rgb(37, 45, 56),
            text_primary: Color32::from_rgb(243, 245, 247),
            text_secondary: Color32::from_rgb(154, 165, 177),
            text_muted: Color32::from_rgb(105, 117, 134),
            accent: Color32::from_rgb(139, 140, 255),
            success: Color32::from_rgb(53, 196, 138),
            warning: Color32::from_rgb(231, 182, 90),
            danger: Color32::from_rgb(240, 106, 122),
        }
    }
}

impl DbProTheme {
    pub fn apply(self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(self.text_primary);
        visuals.panel_fill = self.surface_panel;
        visuals.window_fill = self.surface_elevated;
        visuals.faint_bg_color = self.surface_app;
        visuals.extreme_bg_color = self.surface_app;
        visuals.code_bg_color = self.surface_app;
        visuals.hyperlink_color = self.accent;
        visuals.selection.bg_fill = self.accent.linear_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.noninteractive.bg_fill = self.surface_panel;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.inactive.bg_fill = self.surface_elevated;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.hovered.bg_fill = self.surface_hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.active.bg_fill = self.accent.linear_multiply(0.8);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.open.bg_fill = self.surface_hover;
        visuals.window_stroke = Stroke::new(1.0, self.border_subtle);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        ctx.set_style(style);
    }
}
