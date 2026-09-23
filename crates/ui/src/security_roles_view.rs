//! PostgreSQL role list and role-creation presentation.

use super::security_state::SecurityState;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SecurityRolesAction {
    Refresh,
    Select(String),
    RequestDrop(String),
    Create { name: String, login: bool },
}

pub(super) struct SecurityRolesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut SecurityState,
}

impl SecurityRolesContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRolesAction> {
        let mut actions = Vec::new();
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh roles", self.theme).clicked() {
            actions.push(SecurityRolesAction::Refresh);
        }
        ui.add_space(SPACE_MD);
        actions.extend(self.draw_users(ui));
        ui.add_space(SPACE_MD);
        actions.extend(self.draw_create_role(ui));
        actions
    }

    fn draw_users(&self, ui: &mut egui::Ui) -> Vec<SecurityRolesAction> {
        let mut actions = Vec::new();
        section_label(ui, "ROLES / USERS", self.theme);
        ui.add_space(SPACE_SM);
        if self.state.security_users.is_empty() {
            ui.label(
                RichText::new("No roles loaded yet — click Refresh.")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        for user in &self.state.security_users {
            self.draw_user(ui, user, &mut actions);
        }
        actions
    }

    fn draw_user(
        &self,
        ui: &mut egui::Ui,
        user: &db_pro_core::domain::user::DatabaseUser,
        actions: &mut Vec<SecurityRolesAction>,
    ) {
        let selected = self.state.security_selected_role.as_deref() == Some(user.name.as_str());
        ui.horizontal(|ui| {
            if ui.selectable_label(selected, &user.name).clicked() {
                actions.push(SecurityRolesAction::Select(user.name.clone()));
            }
            if user.can_login {
                badge(ui, "login", self.theme.surface_active, self.theme.text_secondary);
            }
            if user.is_super {
                badge(ui, "super", self.theme.warning, self.theme.text_primary);
            }
            if user.can_create_db {
                badge(ui, "createdb", self.theme.surface_active, self.theme.text_secondary);
            }
            if user.can_create_role {
                badge(ui, "createrole", self.theme.surface_active, self.theme.text_secondary);
            }
            if danger_button(ui, "Drop", self.theme).clicked() {
                actions.push(SecurityRolesAction::RequestDrop(user.name.clone()));
            }
        });
    }

    fn draw_create_role(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRolesAction> {
        section_label(ui, "CREATE ROLE", self.theme);
        ui.add_space(SPACE_SM);
        input_full_width(ui, &mut self.state.security_new_role, "role name", self.theme);
        ui.checkbox(&mut self.state.security_new_role_login, "LOGIN");
        if primary_button_with_icon(ui, Icon::Plus, "Create role", self.theme).clicked()
            && !self.state.security_new_role.trim().is_empty()
        {
            return vec![SecurityRolesAction::Create {
                name: self.state.security_new_role.trim().to_owned(),
                login: self.state.security_new_role_login,
            }];
        }
        Vec::new()
    }
}
