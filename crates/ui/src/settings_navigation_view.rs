//! Settings navigation presentation and section-selection intents.

use super::{DbProTheme, SettingsSection};
use eframe::egui::{self, RichText};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SettingsNavigationAction {
    SelectSection(SettingsSection),
}

pub(crate) struct SettingsNavigationContext {
    pub(crate) theme: DbProTheme,
    pub(crate) selected: SettingsSection,
}

impl SettingsNavigationContext {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> Vec<SettingsNavigationAction> {
        let mut actions = Vec::new();
        ui.set_width(128.0);
        for section in SettingsSection::all() {
            let selected = self.selected == *section;
            let label = RichText::new(section.label()).color(if selected {
                self.theme.accent
            } else {
                self.theme.text_secondary
            });
            if ui.selectable_label(selected, label).clicked() {
                actions.push(SettingsNavigationAction::SelectSection(*section));
            }
        }
        actions
    }
}
