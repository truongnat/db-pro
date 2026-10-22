//! Security confirmation dialogs and intent collection.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SecurityConfirmationAction {
    ConfirmDropRole(String),
    CancelDropRole,
    ConfirmPending,
    CancelPending,
}

pub(super) struct SecurityConfirmationContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) drop_role: Option<&'a str>,
    pub(super) pending: Option<&'a super::security_state::PendingSecurityAction>,
}

impl SecurityConfirmationContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Option<SecurityConfirmationAction> {
        if let Some(role) = self.drop_role {
            return self.draw_choice(
                ctx,
                "Drop role?",
                "security_drop_role_dialog",
                format!("Drop role `{role}`? This cannot be undone."),
                "Drop role",
                SecurityConfirmationAction::ConfirmDropRole(role.to_owned()),
                SecurityConfirmationAction::CancelDropRole,
            );
        }
        let pending = self.pending?;
        self.draw_choice(
            ctx,
            "Confirm security change?",
            "security_pending_dialog",
            pending.prompt(),
            "Confirm",
            SecurityConfirmationAction::ConfirmPending,
            SecurityConfirmationAction::CancelPending,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_choice(
        &self,
        ctx: &egui::Context,
        title: &str,
        id_salt: &str,
        body: String,
        confirm_label: &str,
        confirm: SecurityConfirmationAction,
        cancel: SecurityConfirmationAction,
    ) -> Option<SecurityConfirmationAction> {
        let mut action = None;
        let mut open = true;
        Dialog::new(&mut open, title, self.theme)
            .width(420.0)
            .id_salt(id_salt)
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(body);
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .text(confirm_label)
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(confirm.clone());
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(cancel.clone());
                    }
                });
            });
        if !open && action.is_none() {
            action = Some(cancel);
        }
        action
    }
}
