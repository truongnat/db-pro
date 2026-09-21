//! PostgreSQL security surface composition and intent collection.
use super::super::security_state::SecurityState;
use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SecuritySurfaceAction {
    Roles(security_roles_view::SecurityRolesAction),
    RoleDetails {
        role: String,
        action: security_role_details_view::SecurityRoleDetailsAction,
    },
    Confirmation(security_confirmation_view::SecurityConfirmationAction),
    Rls(security_rls_view::SecurityRlsAction),
}

pub(super) struct SecuritySurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut SecurityState,
    pub(super) connected: bool,
    pub(super) is_postgres: bool,
}

impl SecuritySurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Vec<SecuritySurfaceAction> {
        section_label(ui, "SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        if !self.connected {
            self.draw_notice(ui, "Connect a PostgreSQL database to manage roles and privileges.");
            return Vec::new();
        }
        if !self.is_postgres {
            self.draw_notice(ui, "User/role management is PostgreSQL-only (capability gated).");
            return Vec::new();
        }

        if let Some(error) = &self.state.security_error {
            ui.colored_label(self.theme.warning, error);
        }

        let mut actions = self.draw_roles(ui);
        let selected_role = self
            .pending_selected_role(&actions)
            .or(self.state.security_selected_role.as_deref())
            .map(str::to_owned);
        let drop_role = self
            .pending_drop_role(&actions)
            .or(self.state.security_drop_confirm.as_deref())
            .map(str::to_owned);
        actions.extend(self.draw_role_details(ui, selected_role.as_deref()));
        actions.extend(self.draw_confirmation(ctx, drop_role.as_deref()));
        actions.extend(self.draw_rls(ui));
        actions
    }

    fn draw_notice(&self, ui: &mut egui::Ui, message: &str) {
        ui.label(RichText::new(message).small().color(self.theme.text_muted));
    }

    fn draw_roles(&mut self, ui: &mut egui::Ui) -> Vec<SecuritySurfaceAction> {
        security_roles_view::SecurityRolesContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui)
        .into_iter()
        .map(SecuritySurfaceAction::Roles)
        .collect()
    }

    fn draw_role_details(&mut self, ui: &mut egui::Ui, selected_role: Option<&str>) -> Vec<SecuritySurfaceAction> {
        let Some(role) = selected_role.map(str::to_owned) else {
            return Vec::new();
        };
        security_role_details_view::SecurityRoleDetailsContext {
            theme: self.theme,
            state: self.state,
            role: &role,
        }
        .draw(ui)
        .into_iter()
        .map(|action| SecuritySurfaceAction::RoleDetails {
            role: role.clone(),
            action,
        })
        .collect()
    }

    fn draw_confirmation(&self, ctx: &egui::Context, drop_role: Option<&str>) -> Vec<SecuritySurfaceAction> {
        security_confirmation_view::SecurityConfirmationContext {
            theme: self.theme,
            drop_role,
        }
        .draw(ctx)
        .into_iter()
        .map(SecuritySurfaceAction::Confirmation)
        .collect()
    }

    fn pending_selected_role<'a>(&self, actions: &'a [SecuritySurfaceAction]) -> Option<&'a str> {
        actions.iter().find_map(|action| match action {
            SecuritySurfaceAction::Roles(security_roles_view::SecurityRolesAction::Select(role)) => Some(role.as_str()),
            _ => None,
        })
    }

    fn pending_drop_role<'a>(&self, actions: &'a [SecuritySurfaceAction]) -> Option<&'a str> {
        actions.iter().find_map(|action| match action {
            SecuritySurfaceAction::Roles(security_roles_view::SecurityRolesAction::RequestDrop(role)) => {
                Some(role.as_str())
            }
            _ => None,
        })
    }

    fn draw_rls(&mut self, ui: &mut egui::Ui) -> Vec<SecuritySurfaceAction> {
        security_rls_view::SecurityRlsContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui)
        .into_iter()
        .map(SecuritySurfaceAction::Rls)
        .collect()
    }
}
