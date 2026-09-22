use super::super::{FeedbackState, OverlayState};
use super::{ConnectionCatalogState, ConnectionLifecycleState, PendingConnectionOperation};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::tokens::*;
use crate::{DbProTheme, UiCommand};
use egui::RichText;

/// Render and reduce the connection deletion confirmation without reaching
/// through the composition root.
pub(crate) fn draw(
    ctx: &egui::Context,
    theme: DbProTheme,
    overlay: &mut OverlayState,
    catalog: &ConnectionCatalogState,
    lifecycle: &mut ConnectionLifecycleState,
    command_dispatcher: &mut super::super::command_dispatch::RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
) {
    let Some(connection_id) = overlay.delete_confirmation_id.clone() else {
        return;
    };
    let name = catalog
        .iter()
        .find(|connection| connection.id == connection_id)
        .map(|connection| connection.name.clone())
        .unwrap_or_else(|| "this connection".to_owned());
    let mut open = true;
    let mut confirmed = false;
    let mut cancelled = false;

    Dialog::new(&mut open, t!("connection.delete_connection"), theme)
        .width(420.0)
        .id_salt("delete_conn_modal_dialog")
        .show_framed_ctx(ctx, |frame| {
            frame.body(|ui| {
                ui.label(
                    RichText::new(t!("connection.delete_conn_confirm", name = name.as_str())).color(theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                ui.colored_label(theme.warning, t!("connection.action_undone"));
            });
            frame.footer(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Button::new(theme)
                        .text(t!("connection.delete_connection"))
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
        let command = UiCommand::DeleteConnection {
            request_id,
            connection_id: connection_id.clone(),
        };
        if !command_dispatcher.send_best_effort(command) {
            let message = "Runtime worker unavailable";
            feedback.set_runtime_message(message);
            feedback.show_error_toast(message);
        } else {
            // Track the target only after the runtime accepted the command so a
            // closed worker cannot leave a phantom delete operation pending.
            lifecycle.set_pending_request(Some(request_id));
            lifecycle.set_pending_operation(Some(PendingConnectionOperation::Delete));
            lifecycle.set_pending_connection_id(Some(connection_id));
            feedback.set_runtime_message(t!("status.deleting", name = name.as_str()));
            overlay.delete_confirmation_id = None;
        }
    } else if cancelled || !open {
        overlay.delete_confirmation_id = None;
    }
}
