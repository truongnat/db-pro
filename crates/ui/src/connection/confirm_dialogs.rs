use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::tokens::*;
use crate::{DbProApp, UiCommand};
use egui::RichText;

impl DbProApp {
    /// Render query folder deletion confirmation dialog.
    pub(crate) fn draw_folder_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(folder_id) = self.overlay.folder_delete_confirmation.clone() else {
            return;
        };
        let folder_name = self
            .query_library
            .query_folders
            .iter()
            .find(|folder| folder.id == folder_id)
            .map(|folder| folder.name.clone())
            .unwrap_or_else(|| "this folder".to_owned());
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;

        Dialog::new(&mut open, t!("connection.delete_folder"), self.theme)
            .width(420.0)
            .id_salt("delete_folder_modal_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(
                        RichText::new(t!("connection.delete_folder_confirm", name = folder_name.as_str()))
                            .color(self.theme.text_primary),
                    );
                    ui.add_space(SPACE_SM);
                    ui.colored_label(self.theme.warning, t!("connection.delete_folder_warning"));
                });
                frame.footer(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::new(self.theme)
                            .text(t!("connection.delete_folder"))
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            confirmed = true;
                        }
                        if Button::new(self.theme)
                            .text(t!("connection.cancel"))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            cancelled = true;
                        }
                    });
                });
            });

        if confirmed {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::DeleteQueryFolder {
                request_id,
                id: folder_id,
            });
            self.overlay.folder_delete_confirmation = None;
        } else if cancelled || !open {
            self.overlay.folder_delete_confirmation = None;
        }
    }
}
