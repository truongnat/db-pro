use super::*;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsBackupAction {
    PickBackup,
    CreateBackup,
    PickRestore,
    ConfirmRestore,
}

pub(super) struct SettingsBackupContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) overlay: &'a mut OverlayState,
}

impl SettingsBackupContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SettingsBackupAction> {
        let mut actions = Vec::new();
        self.draw_backup_form(ui, &mut actions);
        ui.add_space(14.0);
        self.draw_restore_form(ui, &mut actions);
        actions
    }

    fn draw_backup_form(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsBackupAction>) {
        ui.label(
            RichText::new("Backup destination")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(
            ui,
            &mut self.overlay.backup_output_path,
            "Choose a .sql backup path",
            self.theme,
        );
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose path", self.theme).clicked() {
                actions.push(SettingsBackupAction::PickBackup);
            }
            if secondary_button_with_icon(ui, Icon::Archive, "Create backup", self.theme).clicked() {
                actions.push(SettingsBackupAction::CreateBackup);
            }
        });
    }

    fn draw_restore_form(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsBackupAction>) {
        ui.label(
            RichText::new("Restore from backup")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(
            ui,
            &mut self.overlay.restore_input_path,
            "Choose a backup file",
            self.theme,
        );
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose file", self.theme).clicked() {
                actions.push(SettingsBackupAction::PickRestore);
            }
            if secondary_button_with_icon(ui, Icon::RotateCcw, "Restore database", self.theme).clicked() {
                self.overlay.restore_confirmation = true;
            }
        });
        if self.overlay.restore_confirmation {
            self.draw_restore_confirmation(ui, actions);
        }
    }

    fn draw_restore_confirmation(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsBackupAction>) {
        ui.add_space(10.0);
        ui.colored_label(self.theme.warning, "Overwrite the active database?");
        ui.horizontal(|ui| {
            if danger_button(ui, "Confirm restore", self.theme).clicked() {
                self.overlay.restore_confirmation = false;
                actions.push(SettingsBackupAction::ConfirmRestore);
            }
            if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                self.overlay.restore_confirmation = false;
            }
        });
    }
}
