use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::tokens::*;
use crate::{DbProApp, UiCommand};
use egui::RichText;

impl DbProApp {
    /// Render connection deletion confirmation dialog.
    pub(crate) fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(connection_id) = self.delete_confirmation_id.clone() else {
            return;
        };
        let name = self
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "this connection".to_owned());
        let mut open = true;
        egui::Area::new(egui::Id::new("delete_conn_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, "Delete Connection", self.theme)
                    .width(420.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Delete \"{name}\" and its saved credentials?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "This action cannot be undone.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if Button::new(self.theme)
                                .text("Delete Connection")
                                .variant(ButtonVariant::Destructive)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::DeleteConnection {
                                    request_id,
                                    connection_id: connection_id.clone(),
                                });
                                self.pending_connection_request = Some(request_id);
                                self.runtime_message = format!("Deleting {name}…");
                                self.delete_confirmation_id = None;
                            }
                            if Button::new(self.theme)
                                .text("Cancel")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                self.delete_confirmation_id = None;
                            }
                        });
                    });
            });
        if !open {
            self.delete_confirmation_id = None;
        }
    }

    /// Render query folder deletion confirmation dialog.
    pub(crate) fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(folder_id) = self.folder_delete_confirmation.clone() else {
            return;
        };
        let folder_name = self
            .query_folders
            .iter()
            .find(|folder| folder.id == folder_id)
            .map(|folder| folder.name.clone())
            .unwrap_or_else(|| "this folder".to_owned());
        let mut open = true;
        egui::Area::new(egui::Id::new("delete_folder_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, "Delete Query Folder", self.theme)
                    .width(420.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("Delete \"{folder_name}\" and its saved-query links?"))
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(SPACE_SM);
                        ui.colored_label(self.theme.warning, "Saved queries in this folder will become unfiled.");
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            if Button::new(self.theme)
                                .text("Delete Folder")
                                .variant(ButtonVariant::Destructive)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::DeleteQueryFolder {
                                    request_id,
                                    id: folder_id.clone(),
                                });
                                self.folder_delete_confirmation = None;
                            }
                            if Button::new(self.theme)
                                .text("Cancel")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .show(ui)
                                .clicked()
                            {
                                self.folder_delete_confirmation = None;
                            }
                        });
                    });
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }
}
