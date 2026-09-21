use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use egui::RichText;

/// Render and reduce query-folder deletion confirmation with explicit state
/// and runtime dependencies.
pub(crate) fn draw(
    ctx: &egui::Context,
    theme: DbProTheme,
    overlay: &mut OverlayState,
    query_library: &QueryLibraryState,
    command_dispatcher: &mut command_dispatch::RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
) {
    let Some(folder_id) = overlay.folder_delete_confirmation.clone() else {
        return;
    };
    let folder_name = query_library
        .query_folders
        .iter()
        .find(|folder| folder.id == folder_id)
        .map(|folder| folder.name.clone())
        .unwrap_or_else(|| "this folder".to_owned());
    let mut open = true;
    let mut confirmed = false;
    let mut cancelled = false;

    Dialog::new(&mut open, t!("connection.delete_folder"), theme)
        .width(420.0)
        .id_salt("delete_folder_modal_dialog")
        .show_framed_ctx(ctx, |frame| {
            frame.body(|ui| {
                ui.label(
                    RichText::new(t!("connection.delete_folder_confirm", name = folder_name.as_str()))
                        .color(theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                ui.colored_label(theme.warning, t!("connection.delete_folder_warning"));
            });
            frame.footer(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Button::new(theme)
                        .text(t!("connection.delete_folder"))
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        confirmed = true;
                    }
                    if Button::new(theme)
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
        let request_id = command_dispatcher.next_request_id();
        if !command_dispatcher.send_best_effort(UiCommand::DeleteQueryFolder {
            request_id,
            id: folder_id,
        }) {
            let message = "Runtime worker unavailable";
            feedback.set_runtime_message(message);
            feedback.show_error_toast(message);
        }
        overlay.folder_delete_confirmation = None;
    } else if cancelled || !open {
        overlay.folder_delete_confirmation = None;
    }
}
