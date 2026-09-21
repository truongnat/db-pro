//! Security confirmation dialogs and intent collection.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SecurityConfirmationAction {
    ConfirmDropRole(String),
    CancelDropRole,
}

pub(super) struct SecurityConfirmationContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) drop_role: Option<&'a str>,
}

impl SecurityConfirmationContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Option<SecurityConfirmationAction> {
        let role = self.drop_role?;
        let mut action = None;
        egui::Window::new("Drop role?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!("Drop role `{role}`? This cannot be undone."));
                ui.horizontal(|ui| {
                    if danger_button(ui, "Drop role", self.theme).clicked() {
                        action = Some(SecurityConfirmationAction::ConfirmDropRole(role.to_owned()));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        action = Some(SecurityConfirmationAction::CancelDropRole);
                    }
                });
            });
        action
    }
}
