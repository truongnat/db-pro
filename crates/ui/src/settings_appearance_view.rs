use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct SettingsAppearanceContext<'a> {
    pub(super) theme: &'a mut DbProTheme,
    pub(super) preferences: &'a mut PreferencesState,
}

impl SettingsAppearanceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        card_frame(*self.theme).show(ui, |ui| {
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
                if ui
                    .selectable_value(&mut self.preferences.dark_mode, false, "Light")
                    .changed()
                {
                    self.preferences.settings.appearance.dark_mode = false;
                    *self.theme = DbProTheme::light();
                }
                if ui
                    .selectable_value(&mut self.preferences.dark_mode, true, "Dark")
                    .changed()
                {
                    self.preferences.settings.appearance.dark_mode = true;
                    *self.theme = DbProTheme::dark();
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
            }
            ui.label(
                RichText::new("Loading states keep a static status icon instead of a spinner.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }
}
