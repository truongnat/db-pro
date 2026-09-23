//! Keybinding settings presentation and state-local intents.

use super::settings_model::{default_keybinding_catalog, KeybindingCommand};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsKeybindingsAction {
    ResetAll,
}

pub(crate) struct SettingsKeybindingsContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) preferences: &'a mut PreferencesState,
}

impl SettingsKeybindingsContext<'_> {
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SettingsKeybindingsAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            actions.extend(self.draw_header(ui));
            self.draw_command_list(ui);
        });
        actions
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) -> Vec<SettingsKeybindingsAction> {
        let mut actions = Vec::new();
        section_label(ui, "KEYBINDINGS", self.theme);
        ui.add_space(10.0);
        input_full_width(
            ui,
            &mut self.preferences.keybindings_filter,
            "Search commands…",
            self.theme,
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if compact_button(ui, "Reset all to defaults", self.theme).clicked() {
                self.preferences.settings.keybindings.reset_all();
                self.preferences.keybinding_edit_id = None;
                actions.push(SettingsKeybindingsAction::ResetAll);
            }
        });
        let conflicts = self.preferences.settings.keybindings.conflict_ids();
        if !conflicts.is_empty() {
            ui.colored_label(
                self.theme.warning,
                format!("{} keybinding conflict(s) detected", conflicts.len()),
            );
        }
        ui.add_space(8.0);
        actions
    }

    fn draw_command_list(&mut self, ui: &mut egui::Ui) {
        let filter = self.preferences.keybindings_filter.trim().to_ascii_lowercase();
        let catalog: Vec<_> = default_keybinding_catalog()
            .iter()
            .filter(|command| {
                filter.is_empty()
                    || command.title.to_ascii_lowercase().contains(&filter)
                    || command.id.contains(&filter)
            })
            .copied()
            .collect();
        let conflicts = self.preferences.settings.keybindings.conflict_ids();
        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            for command in catalog {
                self.draw_command_row(ui, command, conflicts.contains(command.id));
                ui.add_space(4.0);
            }
        });
    }

    fn draw_command_row(&mut self, ui: &mut egui::Ui, command: KeybindingCommand, is_conflict: bool) {
        let resolved = self.preferences.settings.keybindings.resolved(command.id);
        let editing = self.preferences.keybinding_edit_id.as_deref() == Some(command.id);
        ui.horizontal(|ui| {
            ui.label(RichText::new(command.title).color(if is_conflict {
                self.theme.warning
            } else {
                self.theme.text_primary
            }));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if editing {
                    self.draw_editing_row(ui, command);
                } else {
                    self.draw_read_only_row(ui, command, resolved);
                }
            });
        });
    }

    fn draw_editing_row(&mut self, ui: &mut egui::Ui, command: KeybindingCommand) {
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.preferences.keybinding_edit_draft)
                .desired_width(120.0)
                .hint_text("mod+k"),
        );
        if (response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)))
            || compact_button(ui, "Save", self.theme).clicked()
        {
            let draft = self.preferences.keybinding_edit_draft.trim().to_ascii_lowercase();
            if draft.is_empty() || draft == command.default_shortcut {
                self.preferences.settings.keybindings.reset_one(command.id);
            } else {
                self.preferences
                    .settings
                    .keybindings
                    .overrides
                    .insert(command.id.to_owned(), draft);
            }
            self.preferences.keybinding_edit_id = None;
        }
        if compact_button(ui, "Cancel", self.theme).clicked() {
            self.preferences.keybinding_edit_id = None;
        }
    }

    fn draw_read_only_row(&mut self, ui: &mut egui::Ui, command: KeybindingCommand, resolved: String) {
        if compact_button(ui, "Edit", self.theme).clicked() {
            self.preferences.keybinding_edit_id = Some(command.id.to_owned());
            self.preferences.keybinding_edit_draft = resolved.clone();
        }
        if self.preferences.settings.keybindings.overrides.contains_key(command.id)
            && compact_button(ui, "Reset", self.theme).clicked()
        {
            self.preferences.settings.keybindings.reset_one(command.id);
        }
        ui.label(RichText::new(resolved).monospace().color(self.theme.text_secondary));
    }
}
