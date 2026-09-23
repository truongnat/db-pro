//! Selected-role details presentation and user intents.

use super::security_state::SecurityState;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SecurityRoleDetailsAction {
    AlterRole(db_pro_core::domain::user::RoleAttributes),
    UpdatePassword(String),
    RevokeMembership(String),
    GrantMembership(String),
    RevokePrivilege(db_pro_core::domain::user::Privilege),
    GrantPrivilege {
        kind: db_pro_core::domain::user::PrivilegeObjectKind,
        schema: String,
        object_name: String,
        privilege: String,
    },
}

pub(super) struct SecurityRoleDetailsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut SecurityState,
    pub(super) role: &'a str,
}

impl SecurityRoleDetailsContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        actions.extend(self.draw_attributes(ui));
        actions.extend(self.draw_password(ui));
        actions.extend(self.draw_memberships(ui));
        actions.extend(self.draw_privileges(ui));
        actions
    }

    fn draw_attributes(&self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_MD);
        section_label(ui, format!("ATTRIBUTES · {}", self.role), self.theme);
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::Check, "LOGIN", self.theme).clicked() {
                actions.push(SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        login: Some(true),
                        ..Default::default()
                    },
                ));
            }
            if secondary_button_with_icon(ui, Icon::X, "NOLOGIN", self.theme).clicked() {
                actions.push(SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        login: Some(false),
                        ..Default::default()
                    },
                ));
            }
            if secondary_button_with_icon(ui, Icon::Database, "CREATEDB", self.theme).clicked() {
                actions.push(SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createdb: Some(true),
                        ..Default::default()
                    },
                ));
            }
            if secondary_button_with_icon(ui, Icon::Users, "CREATEROLE", self.theme).clicked() {
                actions.push(SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createrole: Some(true),
                        ..Default::default()
                    },
                ));
            }
        });
        actions
    }

    fn draw_password(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_MD);
        section_label(ui, format!("PASSWORD · {}", self.role), self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Password is never logged or shown in runtime events.")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.state.security_password)
                .password(true)
                .hint_text("new password")
                .desired_width(f32::INFINITY),
        );
        if primary_button_with_icon(ui, Icon::Key, "Update password", self.theme).clicked()
            && !self.state.security_password.is_empty()
        {
            actions.push(SecurityRoleDetailsAction::UpdatePassword(
                self.state.security_password.clone(),
            ));
        }
        actions
    }

    fn draw_memberships(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_MD);
        section_label(ui, format!("MEMBERSHIPS · {}", self.role), self.theme);
        ui.add_space(SPACE_SM);
        if self.state.security_memberships.is_empty() {
            ui.label(
                RichText::new("No role memberships.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for membership in &self.state.security_memberships {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("member of {}", membership.role))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                    if danger_button(ui, "Revoke", self.theme).clicked() {
                        actions.push(SecurityRoleDetailsAction::RevokeMembership(membership.role.clone()));
                    }
                });
            }
        }
        input_full_width(
            ui,
            &mut self.state.security_membership_role,
            "grant role name",
            self.theme,
        );
        if secondary_button_with_icon(ui, Icon::Plus, "Grant membership", self.theme).clicked()
            && !self.state.security_membership_role.trim().is_empty()
        {
            actions.push(SecurityRoleDetailsAction::GrantMembership(
                self.state.security_membership_role.trim().to_owned(),
            ));
        }
        actions
    }

    fn draw_privileges(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_MD);
        section_label(ui, format!("PRIVILEGES · {}", self.role), self.theme);
        ui.add_space(SPACE_SM);
        if self.state.security_privileges.is_empty() {
            ui.label(
                RichText::new("No privileges listed for this role.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for privilege in &self.state.security_privileges {
                self.draw_privilege(ui, privilege, &mut actions);
            }
        }
        actions.extend(self.draw_grant_form(ui));
        actions
    }

    fn draw_privilege(
        &self,
        ui: &mut egui::Ui,
        privilege: &db_pro_core::domain::user::Privilege,
        actions: &mut Vec<SecurityRoleDetailsAction>,
    ) {
        let target = match privilege.object_kind {
            db_pro_core::domain::user::PrivilegeObjectKind::Database
            | db_pro_core::domain::user::PrivilegeObjectKind::Schema => privilege.object_name.clone(),
            _ => format!("{}.{}", privilege.schema, privilege.object_name),
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} · {} · {}",
                    privilege.object_kind.as_label(),
                    target,
                    privilege.privilege_type
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
            if danger_button(ui, "Revoke", self.theme).clicked() {
                actions.push(SecurityRoleDetailsAction::RevokePrivilege(privilege.clone()));
            }
        });
    }

    fn draw_grant_form(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRoleDetailsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_SM);
        section_label(ui, "GRANT PRIVILEGE", self.theme);
        ui.horizontal(|ui| {
            use db_pro_core::domain::user::PrivilegeObjectKind;
            for (label, kind) in [
                ("table", PrivilegeObjectKind::Table),
                ("schema", PrivilegeObjectKind::Schema),
                ("database", PrivilegeObjectKind::Database),
                ("sequence", PrivilegeObjectKind::Sequence),
            ] {
                if ui
                    .selectable_label(self.state.security_grant_kind == kind, label)
                    .clicked()
                {
                    self.state.security_grant_kind = kind;
                }
            }
        });
        if !matches!(
            self.state.security_grant_kind,
            db_pro_core::domain::user::PrivilegeObjectKind::Database
                | db_pro_core::domain::user::PrivilegeObjectKind::Schema
        ) {
            input_full_width(ui, &mut self.state.security_grant_schema, "schema", self.theme);
        }
        let object_hint = self.state.security_grant_kind.as_label();
        input_full_width(ui, &mut self.state.security_grant_object, object_hint, self.theme);
        input_full_width(
            ui,
            &mut self.state.security_grant_privilege,
            "privilege (SELECT/USAGE/CONNECT/…)",
            self.theme,
        );
        if primary_button_with_icon(ui, Icon::Plus, "Grant privilege", self.theme).clicked()
            && !self.state.security_grant_object.trim().is_empty()
            && !self.state.security_grant_privilege.trim().is_empty()
        {
            actions.push(SecurityRoleDetailsAction::GrantPrivilege {
                kind: self.state.security_grant_kind,
                schema: self.state.security_grant_schema.trim().to_owned(),
                object_name: self.state.security_grant_object.trim().to_owned(),
                privilege: self.state.security_grant_privilege.trim().to_owned(),
            });
        }
        actions
    }
}
