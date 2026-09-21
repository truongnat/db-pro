use super::*;

#[path = "security_surface_view.rs"]
mod security_surface_view;

impl DbProApp {
    pub(super) fn draw_security_activity(&mut self, ui: &mut egui::Ui) {
        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let is_pg = self.active_driver().eq_ignore_ascii_case("postgresql")
            || self.active_driver().eq_ignore_ascii_case("postgres");
        let context = ui.ctx().clone();
        let actions = security_surface_view::SecuritySurfaceContext {
            theme: self.theme,
            state: &mut self.management.security,
            connected,
            is_postgres: is_pg,
        }
        .draw(ui, &context);
        self.apply_security_surface_actions(actions);
    }

    fn apply_security_surface_actions(&mut self, actions: Vec<security_surface_view::SecuritySurfaceAction>) {
        for action in actions {
            match action {
                security_surface_view::SecuritySurfaceAction::Roles(action) => {
                    self.apply_security_roles_actions(vec![action]);
                }
                security_surface_view::SecuritySurfaceAction::RoleDetails { role, action } => {
                    self.apply_security_role_details_actions(&role, vec![action]);
                }
                security_surface_view::SecuritySurfaceAction::Confirmation(action) => {
                    self.apply_security_confirmation_action(action);
                }
                security_surface_view::SecuritySurfaceAction::Rls(action) => {
                    self.apply_security_rls_actions(vec![action]);
                }
            }
        }
    }

    fn apply_security_confirmation_action(&mut self, action: security_confirmation_view::SecurityConfirmationAction) {
        match action {
            security_confirmation_view::SecurityConfirmationAction::ConfirmDropRole(name) => {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(
                        self.management
                            .security
                            .drop_role_command(request_id, connection_id, name),
                    );
                }
                self.management.security.security_drop_confirm = None;
            }
            security_confirmation_view::SecurityConfirmationAction::CancelDropRole => {
                self.management.security.security_drop_confirm = None;
            }
        }
    }

    fn apply_security_roles_actions(&mut self, actions: Vec<security_roles_view::SecurityRolesAction>) {
        for action in actions {
            match action {
                security_roles_view::SecurityRolesAction::Refresh => self.request_security_users(),
                security_roles_view::SecurityRolesAction::Select(role_name) => {
                    self.management.security.security_selected_role = Some(role_name.clone());
                    self.request_security_role_details(&role_name);
                }
                security_roles_view::SecurityRolesAction::RequestDrop(role_name) => {
                    self.management.security.security_drop_confirm = Some(role_name);
                }
                security_roles_view::SecurityRolesAction::Create { name, login } => {
                    let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned)
                    else {
                        continue;
                    };
                    self.management.security.security_new_role_login = login;
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.management.security.create_role_command(
                        request_id,
                        connection_id,
                        name,
                    ));
                    self.management.security.security_new_role.clear();
                }
            }
        }
    }

    fn apply_security_role_details_actions(
        &mut self,
        role: &str,
        actions: Vec<security_role_details_view::SecurityRoleDetailsAction>,
    ) {
        for action in actions {
            let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
                continue;
            };
            let request_id = self.task_bridge.next_request_id();
            match action {
                security_role_details_view::SecurityRoleDetailsAction::AlterRole(attributes) => {
                    self.dispatch_command(self.management.security.alter_role_command(
                        request_id,
                        connection_id,
                        role.to_owned(),
                        attributes,
                    ));
                }
                security_role_details_view::SecurityRoleDetailsAction::UpdatePassword(password) => {
                    self.dispatch_command(self.management.security.update_password_command(
                        request_id,
                        connection_id,
                        role.to_owned(),
                        password,
                    ));
                    self.management.security.security_password.clear();
                }
                security_role_details_view::SecurityRoleDetailsAction::RevokeMembership(member_role) => {
                    self.dispatch_command(self.management.security.revoke_membership_command(
                        request_id,
                        connection_id,
                        member_role,
                        role.to_owned(),
                    ));
                }
                security_role_details_view::SecurityRoleDetailsAction::GrantMembership(member_role) => {
                    self.dispatch_command(self.management.security.grant_membership_command(
                        request_id,
                        connection_id,
                        member_role,
                        role.to_owned(),
                    ));
                    self.management.security.security_membership_role.clear();
                }
                security_role_details_view::SecurityRoleDetailsAction::RevokePrivilege(privilege) => {
                    self.dispatch_command(self.management.security.revoke_privilege_command(
                        request_id,
                        connection_id,
                        role.to_owned(),
                        privilege,
                    ));
                }
                security_role_details_view::SecurityRoleDetailsAction::GrantPrivilege {
                    kind,
                    schema,
                    object_name,
                    privilege,
                } => {
                    self.management.security.security_grant_kind = kind;
                    self.management.security.security_grant_schema = schema;
                    self.management.security.security_grant_object = object_name;
                    self.management.security.security_grant_privilege = privilege;
                    self.dispatch_command(self.management.security.grant_privilege_command(
                        request_id,
                        connection_id,
                        role.to_owned(),
                    ));
                }
            }
        }
    }

    pub(crate) fn request_security_users(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.security.list_users_command(request_id, connection_id));
    }

    pub(crate) fn request_security_role_details(&mut self, role_name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.security.list_privileges_command(
            request_id,
            connection_id.clone(),
            role_name.to_owned(),
        ));
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.security.list_memberships_command(
            request_id,
            connection_id,
            role_name.to_owned(),
        ));
    }

    pub(crate) fn request_security_rls(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        match self
            .management
            .security
            .list_table_rls_command(request_id, connection_id)
        {
            Ok(command) => {
                self.dispatch_command(command);
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn preview_table_rls(&mut self, force: bool, enable: bool) {
        match security_rls::plan_table_rls(security_rls::TableRlsPreviewRequest {
            schema: &self.management.security.security_rls_schema,
            table: &self.management.security.security_rls_table,
            force,
            enable,
        }) {
            Ok(sql) => {
                self.management.security.security_rls_preview_sql = sql;
                self.management.security.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn preview_rls_policy(&mut self, action: db_pro_core::domain::object_mutation::ObjectAction) {
        match security_rls::plan_policy(security_rls::PolicyPreviewRequest {
            action,
            schema: &self.management.security.security_rls_schema,
            table: &self.management.security.security_rls_table,
            name: &self.management.security.security_rls_policy_name,
            command: &self.management.security.security_rls_command,
            roles_csv: &self.management.security.security_rls_roles,
            using_expr: &self.management.security.security_rls_using,
            with_check_expr: &self.management.security.security_rls_with_check,
        }) {
            Ok(sql) => {
                self.management.security.security_rls_preview_sql = sql;
                self.management.security.security_rls_confirm_apply = false;
            }
            Err(error) => {
                self.management.security.security_rls_preview_sql.clear();
                self.feedback.runtime_message = error;
            }
        }
    }

    fn preview_drop_rls_policy(&mut self, policy_name: &str) {
        match security_rls::plan_drop_policy(
            &self.management.security.security_rls_schema,
            &self.management.security.security_rls_table,
            policy_name,
        ) {
            Ok(sql) => {
                self.management.security.security_rls_preview_sql = sql;
                self.management.security.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn apply_security_rls_actions(&mut self, actions: Vec<security_rls_view::SecurityRlsAction>) {
        for action in actions {
            match action {
                security_rls_view::SecurityRlsAction::Inspect => self.request_security_rls(),
                security_rls_view::SecurityRlsAction::PreviewTable { force, enable } => {
                    self.preview_table_rls(force, enable);
                }
                security_rls_view::SecurityRlsAction::DropPolicy(name) => {
                    self.preview_drop_rls_policy(&name);
                }
                security_rls_view::SecurityRlsAction::LoadPolicy {
                    name,
                    command,
                    roles,
                    using_expression,
                    with_check,
                } => {
                    self.management.security.security_rls_policy_name = name;
                    self.management.security.security_rls_command = command;
                    self.management.security.security_rls_roles = roles;
                    self.management.security.security_rls_using = using_expression;
                    self.management.security.security_rls_with_check = with_check;
                }
                security_rls_view::SecurityRlsAction::PreviewPolicy(action) => {
                    self.preview_rls_policy(action);
                }
                security_rls_view::SecurityRlsAction::ApplyPreview => {
                    if !self.management.security.security_rls_confirm_apply {
                        self.feedback.runtime_message = "Confirm RLS apply checkbox first".into();
                    } else {
                        self.apply_security_rls_preview();
                    }
                }
            }
        }
    }

    fn apply_security_rls_preview(&mut self) {
        if self.table.state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        if let Some(command) = self
            .management
            .security
            .apply_rls_preview_command(request_id, connection_id)
        {
            if self.dispatch_command(command) {
                self.table.state.ddl_execution_request = Some(request_id);
                self.feedback.runtime_message = "Applying RLS mutation…".into();
            }
        }
    }
}
