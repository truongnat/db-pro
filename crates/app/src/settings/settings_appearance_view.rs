use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct SettingsAppearanceContext<'a> {
    pub(super) theme: &'a mut DbProTheme,
    pub(super) preferences: &'a mut PreferencesState,
}

impl SettingsAppearanceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        Card::new(*self.theme).frame().show(ui, |ui| {
            section_label(ui, "APPEARANCE", *self.theme);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(
                    if self.preferences.dark_mode {
                        Icon::Moon
                    } else {
                        Icon::Sun
                    },
                    "",
                    self.theme.accent,
                ));
                if crate::components::RadioGroup::new(*self.theme)
                    .horizontal(true)
                    .option(crate::components::RadioGroupOption::new(false, "Light"))
                    .option(crate::components::RadioGroupOption::new(true, "Dark"))
                    .show(ui, &mut self.preferences.dark_mode)
                    .is_some()
                {
                    self.preferences.settings.appearance.dark_mode = self.preferences.dark_mode;
                    *self.theme = if self.preferences.dark_mode {
                        DbProTheme::dark()
                    } else {
                        DbProTheme::light()
                    };
                    self.theme.reduce_motion = self.preferences.reduce_motion;
                    ui.ctx().request_repaint();
                }
            });
            ui.label(
                RichText::new(
                    "Quiet surfaces, violet focus states, and high-contrast data. The choice is saved locally.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            if ui
                .checkbox(&mut self.preferences.reduce_motion, "Reduce motion")
                .changed()
            {
                self.preferences.settings.appearance.reduce_motion = self.preferences.reduce_motion;
                self.theme.reduce_motion = self.preferences.reduce_motion;
            }
            ui.label(
                RichText::new("Loading states keep a static status icon instead of a spinner.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }
}
