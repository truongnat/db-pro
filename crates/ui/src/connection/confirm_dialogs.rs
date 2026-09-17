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
        Dialog::new(&mut open, t!("connection.delete_connection"), self.theme)
            .width(420.0)
            .id_salt("delete_conn_modal_dialog")
            .show_ctx(ctx, |ui| {
                ui.label(
                    RichText::new(t!("connection.delete_conn_confirm", name = name.as_str()))
                        .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                ui.colored_label(self.theme.warning, t!("connection.action_undone"));
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text(t!("connection.delete_connection"))
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
                        self.runtime_message = t!("status.deleting", name = name.as_str()).to_string();
                        self.delete_confirmation_id = None;
                    }
                    if Button::new(self.theme)
                        .text(t!("connection.cancel"))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.delete_confirmation_id = None;
                    }
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
        Dialog::new(&mut open, t!("connection.delete_folder"), self.theme)
            .width(420.0)
            .id_salt("delete_folder_modal_dialog")
            .show_ctx(ctx, |ui| {
                ui.label(
                    RichText::new(t!("connection.delete_folder_confirm", name = folder_name.as_str()))
                        .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                ui.colored_label(self.theme.warning, t!("connection.delete_folder_warning"));
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text(t!("connection.delete_folder"))
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
                        .text(t!("connection.cancel"))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.folder_delete_confirmation = None;
                    }
                });
            });
        if !open {
            self.folder_delete_confirmation = None;
        }
    }
}
