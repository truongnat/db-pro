use super::command_dispatch::RuntimeCommandDispatcher;
use super::security_state::SecurityState;
use super::*;

#[path = "security_surface_view.rs"]
mod security_surface_view;

pub(super) struct SecurityActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut SecurityState,
    pub(super) table_state: &'a mut TableState,
    pub(super) connected: bool,
    pub(super) is_postgres: bool,
    pub(super) connection_id: Option<&'a str>,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl SecurityActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        let context = ui.ctx().clone();
        let actions = security_surface_view::SecuritySurfaceContext {
            theme: self.theme,
            state: self.state,
            connected: self.connected,
            is_postgres: self.is_postgres,
        }
        .draw(ui, &context);
        self.apply_actions(actions);
    }

    fn apply_actions(&mut self, actions: Vec<security_surface_view::SecuritySurfaceAction>) {
        for action in actions {
            match action {
                security_surface_view::SecuritySurfaceAction::Roles(action) => {
                    self.apply_roles_actions(vec![action]);
                }
                security_surface_view::SecuritySurfaceAction::RoleDetails { role, action } => {
                    self.apply_role_details_actions(&role, vec![action]);
                }
                security_surface_view::SecuritySurfaceAction::Confirmation(action) => {
                    self.apply_confirmation_action(action);
                }
                security_surface_view::SecuritySurfaceAction::Rls(action) => {
                    self.apply_rls_actions(vec![action]);
                }
            }
        }
    }

    fn apply_confirmation_action(&mut self, action: security_confirmation_view::SecurityConfirmationAction) {
        match action {
            security_confirmation_view::SecurityConfirmationAction::ConfirmDropRole(name) => {
                let Some(connection_id) = self.connection_id else {
                    return;
                };
                let request_id = self.command_dispatcher.next_request_id();
                if self.dispatch(drop_role_command(
                    security_request(request_id, connection_id.to_owned()),
                    name,
                )) {
                    self.state.security_drop_confirm = None;
                }
            }
            security_confirmation_view::SecurityConfirmationAction::CancelDropRole => {
                self.state.security_drop_confirm = None;
            }
            security_confirmation_view::SecurityConfirmationAction::ConfirmPending => {
                self.confirm_pending_security_action();
            }
            security_confirmation_view::SecurityConfirmationAction::CancelPending => {
                self.state.security_pending = None;
            }
        }
    }

    fn confirm_pending_security_action(&mut self) {
        let Some(pending) = self.state.security_pending.clone() else {
            return;
        };
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        let connection_id = connection_id.to_owned();
        let command = match pending {
            security_state::PendingSecurityAction::EnableAttribute { role, attributes } => {
                alter_role_command(security_request(request_id, connection_id), role, attributes)
            }
            security_state::PendingSecurityAction::RevokeMembership { role, member_role } => {
                revoke_membership_command(security_request(request_id, connection_id), member_role, role)
            }
            security_state::PendingSecurityAction::RevokePrivilege { role, privilege } => {
                revoke_privilege_command(security_request(request_id, connection_id), role, privilege)
            }
        };
        if self.dispatch(command) {
            self.state.security_pending = None;
        }
    }

    fn apply_roles_actions(&mut self, actions: Vec<security_roles_view::SecurityRolesAction>) {
        for action in actions {
            match action {
                security_roles_view::SecurityRolesAction::Refresh => self.request_users(),
                security_roles_view::SecurityRolesAction::Select(role_name) => {
                    self.state.security_selected_role = Some(role_name.clone());
                    self.request_role_details(&role_name);
                }
                security_roles_view::SecurityRolesAction::RequestDrop(role_name) => {
                    self.state.security_drop_confirm = Some(role_name);
                }
                security_roles_view::SecurityRolesAction::Create { name, login } => {
                    let Some(connection_id) = self.connection_id else {
                        continue;
                    };
                    self.state.security_new_role_login = login;
                    let request_id = self.command_dispatcher.next_request_id();
                    if self.dispatch(create_role_command(
                        self.state,
                        security_request(request_id, connection_id.to_owned()),
                        name,
                    )) {
                        self.state.security_new_role.clear();
                    }
                }
            }
        }
    }

    fn apply_role_details_actions(
        &mut self,
        role: &str,
        actions: Vec<security_role_details_view::SecurityRoleDetailsAction>,
    ) {
        for action in actions {
            let Some(connection_id) = self.connection_id else {
                continue;
            };
            let request_id = self.command_dispatcher.next_request_id();
            let connection_id = connection_id.to_owned();
            match action {
                security_role_details_view::SecurityRoleDetailsAction::AlterRole(attributes) => {
                    if attribute_enable_needs_confirm(&attributes) {
                        self.state.security_pending = Some(security_state::PendingSecurityAction::EnableAttribute {
                            role: role.to_owned(),
                            attributes,
                        });
                        self.feedback.runtime_message = "Confirm before enabling CREATEDB or CREATEROLE".into();
                    } else {
                        self.dispatch(alter_role_command(
                            security_request(request_id, connection_id),
                            role.to_owned(),
                            attributes,
                        ));
                    }
                }
                security_role_details_view::SecurityRoleDetailsAction::UpdatePassword(password) => {
                    if self.dispatch(update_password_command(
                        security_request(request_id, connection_id),
                        role.to_owned(),
                        password,
                    )) {
                        self.state.security_password.clear();
                    }
                }
                security_role_details_view::SecurityRoleDetailsAction::RevokeMembership(member_role) => {
                    self.state.security_pending = Some(security_state::PendingSecurityAction::RevokeMembership {
                        role: role.to_owned(),
                        member_role,
                    });
                    self.feedback.runtime_message = "Confirm before revoking membership".into();
                }
                security_role_details_view::SecurityRoleDetailsAction::GrantMembership(member_role) => {
                    if self.dispatch(grant_membership_command(
                        security_request(request_id, connection_id),
                        member_role,
                        role.to_owned(),
                    )) {
                        self.state.security_membership_role.clear();
                    }
                }
                security_role_details_view::SecurityRoleDetailsAction::RevokePrivilege(privilege) => {
                    self.state.security_pending = Some(security_state::PendingSecurityAction::RevokePrivilege {
                        role: role.to_owned(),
                        privilege,
                    });
                    self.feedback.runtime_message = "Confirm before revoking the privilege".into();
                }
                security_role_details_view::SecurityRoleDetailsAction::GrantPrivilege {
                    kind,
                    schema,
                    object_name,
                    privilege,
                } => {
                    self.state.security_grant_kind = kind;
                    self.state.security_grant_schema = schema;
                    self.state.security_grant_object = object_name;
                    self.state.security_grant_privilege = privilege;
                    self.dispatch(grant_privilege_command(
                        self.state,
                        security_request(request_id, connection_id),
                        role.to_owned(),
                    ));
                }
            }
        }
    }

    fn request_users(&mut self) {
        request_security_users(self.connection_id, self.command_dispatcher, self.feedback);
    }

    fn request_role_details(&mut self, role_name: &str) {
        request_security_role_details(
            self.state,
            self.connection_id,
            role_name,
            self.command_dispatcher,
            self.feedback,
        );
    }

    fn request_rls(&mut self) {
        request_security_rls(
            self.state,
            self.connection_id,
            self.command_dispatcher,
            self.feedback,
        );
    }

    fn preview_table_rls(&mut self, force: bool, enable: bool) {
        match security_rls::plan_table_rls(security_rls::TableRlsPreviewRequest {
            schema: &self.state.security_rls_schema,
            table: &self.state.security_rls_table,
            force,
            enable,
        }) {
            Ok(sql) => {
                self.state.security_rls_preview_sql = sql;
                self.state.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn preview_rls_policy(&mut self, action: db_pro_core::domain::object_mutation::ObjectAction) {
        match security_rls::plan_policy(security_rls::PolicyPreviewRequest {
            action,
            schema: &self.state.security_rls_schema,
            table: &self.state.security_rls_table,
            name: &self.state.security_rls_policy_name,
            command: &self.state.security_rls_command,
            roles_csv: &self.state.security_rls_roles,
            using_expr: &self.state.security_rls_using,
            with_check_expr: &self.state.security_rls_with_check,
            restrictive: self.state.security_rls_restrictive,
        }) {
            Ok(sql) => {
                self.state.security_rls_preview_sql = sql;
                self.state.security_rls_confirm_apply = false;
            }
            Err(error) => {
                self.state.security_rls_preview_sql.clear();
                self.feedback.runtime_message = error;
            }
        }
    }

    fn preview_drop_rls_policy(&mut self, policy_name: &str) {
        match security_rls::plan_drop_policy(
            &self.state.security_rls_schema,
            &self.state.security_rls_table,
            policy_name,
        ) {
            Ok(sql) => {
                self.state.security_rls_preview_sql = sql;
                self.state.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn apply_rls_actions(&mut self, actions: Vec<security_rls_view::SecurityRlsAction>) {
        for action in actions {
            match action {
                security_rls_view::SecurityRlsAction::Inspect => self.request_rls(),
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
                    self.state.security_rls_policy_name = name;
                    self.state.security_rls_command = command;
                    self.state.security_rls_roles = roles;
                    self.state.security_rls_using = using_expression;
                    self.state.security_rls_with_check = with_check;
                }
                security_rls_view::SecurityRlsAction::PreviewPolicy(action) => {
                    self.preview_rls_policy(action);
                }
                security_rls_view::SecurityRlsAction::ApplyPreview => {
                    if !self.state.security_rls_confirm_apply {
                        self.feedback.runtime_message = "Confirm RLS apply checkbox first".into();
                    } else {
                        self.apply_rls_preview();
                    }
                }
            }
        }
    }

    fn apply_rls_preview(&mut self) {
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        if let Some(command) = apply_rls_preview_command(
            self.state,
            security_request(request_id, connection_id.to_owned()),
        ) {
            if self.dispatch(command) {
                self.table_state.ddl_execution_request = Some(request_id);
                self.feedback.runtime_message = "Applying RLS mutation…".into();
            }
        }
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn request_security_users(
    connection_id: Option<&str>,
    command_dispatcher: &mut RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
) {
    let Some(connection_id) = connection_id else {
        return;
    };
    let request_id = command_dispatcher.next_request_id();
    dispatch(
        command_dispatcher,
        feedback,
        list_users_command(security_request(request_id, connection_id.to_owned())),
    );
}

fn attribute_enable_needs_confirm(attributes: &db_pro_core::domain::user::RoleAttributes) -> bool {
    attributes.createdb == Some(true) || attributes.createrole == Some(true)
}

pub(super) fn request_security_role_details(
    state: &mut SecurityState,
    connection_id: Option<&str>,
    role_name: &str,
    command_dispatcher: &mut RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
) {
    let Some(connection_id) = connection_id else {
        return;
    };
    let request_id = command_dispatcher.next_request_id();
    state.security_privilege_request = Some(request_id.0);
    state.security_error = None;
    dispatch(
        command_dispatcher,
        feedback,
        list_privileges_command(
            security_request(request_id, connection_id.to_owned()),
            role_name.to_owned(),
        ),
    );
    let request_id = command_dispatcher.next_request_id();
    dispatch(
        command_dispatcher,
        feedback,
        list_memberships_command(
            security_request(request_id, connection_id.to_owned()),
            role_name.to_owned(),
        ),
    );
}

pub(super) fn request_security_rls(
    state: &SecurityState,
    connection_id: Option<&str>,
    command_dispatcher: &mut RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
) {
    let Some(connection_id) = connection_id else {
        return;
    };
    let request_id = command_dispatcher.next_request_id();
    match list_table_rls_command(
        state,
        security_request(request_id, connection_id.to_owned()),
    ) {
        Ok(command) => {
            dispatch(command_dispatcher, feedback, command);
        }
        Err(error) => feedback.runtime_message = error,
    }
}

fn dispatch(
    command_dispatcher: &mut RuntimeCommandDispatcher<'_>,
    feedback: &mut FeedbackState,
    command: UiCommand,
) -> bool {
    command_dispatcher.dispatch(command, feedback)
}

struct SecurityCommandRequest {
    request_id: RequestId,
    connection_id: String,
}

fn security_request(request_id: RequestId, connection_id: String) -> SecurityCommandRequest {
    SecurityCommandRequest {
        request_id,
        connection_id,
    }
}

fn list_users_command(request: SecurityCommandRequest) -> UiCommand {
    UiCommand::ListUsers {
        request_id: request.request_id,
        connection_id: request.connection_id,
    }
}

fn list_privileges_command(request: SecurityCommandRequest, role_name: String) -> UiCommand {
    UiCommand::ListPrivileges {
        request_id: request.request_id,
        connection_id: request.connection_id,
        role_name,
    }
}

fn list_memberships_command(request: SecurityCommandRequest, member: String) -> UiCommand {
    UiCommand::ListMemberships {
        request_id: request.request_id,
        connection_id: request.connection_id,
        member,
    }
}

fn list_table_rls_command(state: &SecurityState, request: SecurityCommandRequest) -> Result<UiCommand, String> {
    let (schema, table) = state.table_rls_target()?;
    Ok(UiCommand::ListTableRls {
        request_id: request.request_id,
        connection_id: request.connection_id,
        schema,
        table,
    })
}

fn create_role_command(state: &SecurityState, request: SecurityCommandRequest, name: String) -> UiCommand {
    UiCommand::CreateRole {
        request_id: request.request_id,
        connection_id: request.connection_id,
        name,
        login: state.security_new_role_login,
    }
}

fn drop_role_command(request: SecurityCommandRequest, name: String) -> UiCommand {
    UiCommand::DropRole {
        request_id: request.request_id,
        connection_id: request.connection_id,
        name,
    }
}

fn alter_role_command(
    request: SecurityCommandRequest,
    name: String,
    attributes: db_pro_core::domain::user::RoleAttributes,
) -> UiCommand {
    UiCommand::AlterRole {
        request_id: request.request_id,
        connection_id: request.connection_id,
        name,
        attributes,
    }
}

fn update_password_command(request: SecurityCommandRequest, name: String, password: String) -> UiCommand {
    UiCommand::UpdateRolePassword {
        request_id: request.request_id,
        connection_id: request.connection_id,
        name,
        password,
    }
}

fn grant_membership_command(request: SecurityCommandRequest, role: String, member: String) -> UiCommand {
    UiCommand::GrantMembership {
        request_id: request.request_id,
        connection_id: request.connection_id,
        role,
        member,
    }
}

fn revoke_membership_command(request: SecurityCommandRequest, role: String, member: String) -> UiCommand {
    UiCommand::RevokeMembership {
        request_id: request.request_id,
        connection_id: request.connection_id,
        role,
        member,
    }
}

fn grant_privilege_command(state: &SecurityState, request: SecurityCommandRequest, role_name: String) -> UiCommand {
    UiCommand::GrantPrivilege {
        request_id: request.request_id,
        connection_id: request.connection_id,
        role_name,
        object_kind: state.security_grant_kind,
        schema: state.security_grant_schema.trim().to_owned(),
        object_name: state.security_grant_object.trim().to_owned(),
        privilege: state.security_grant_privilege.trim().to_owned(),
    }
}

fn revoke_privilege_command(
    request: SecurityCommandRequest,
    role_name: String,
    privilege: db_pro_core::domain::user::Privilege,
) -> UiCommand {
    UiCommand::RevokePrivilege {
        request_id: request.request_id,
        connection_id: request.connection_id,
        role_name,
        object_kind: privilege.object_kind,
        schema: privilege.schema,
        object_name: privilege.object_name,
        privilege: privilege.privilege_type,
    }
}

fn apply_rls_preview_command(state: &SecurityState, request: SecurityCommandRequest) -> Option<UiCommand> {
    Some(UiCommand::ExecuteDdl {
        request_id: request.request_id,
        connection_id: request.connection_id,
        sql: state.rls_preview_sql()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        security_role_details_view::SecurityRoleDetailsAction, SecurityActivityContext, SecurityState,
    };

    #[test]
    fn failed_security_dispatch_preserves_password_draft() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        drop(command_rx);
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = SecurityState {
            security_password: "draft-password".to_owned(),
            ..SecurityState::default()
        };
        let mut table_state = crate::app::TableState::default();
        let mut feedback = crate::app::FeedbackState::default();

        SecurityActivityContext {
            theme: crate::DbProTheme::default(),
            state: &mut state,
            table_state: &mut table_state,
            connected: true,
            is_postgres: true,
            connection_id: Some("conn-1"),
            command_dispatcher: &mut dispatcher,
            feedback: &mut feedback,
        }
        .apply_role_details_actions(
            "app_user",
            vec![SecurityRoleDetailsAction::UpdatePassword("new-password".to_owned())],
        );

        assert_eq!(state.security_password, "draft-password");
    }

    #[test]
    fn role_attribute_and_revoke_wait_for_confirm_and_privilege_errors_are_shown() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = SecurityState::default();
        let mut table_state = crate::app::TableState::default();
        let mut feedback = crate::app::FeedbackState::default();

        {
            let mut activity = SecurityActivityContext {
                theme: crate::DbProTheme::default(),
                state: &mut state,
                table_state: &mut table_state,
                connected: true,
                is_postgres: true,
                connection_id: Some("conn-1"),
                command_dispatcher: &mut dispatcher,
                feedback: &mut feedback,
            };

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createdb: Some(true),
                        ..Default::default()
                    },
                )],
            );
            assert!(
                matches!(command_rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)),
                "enabling CREATEDB must wait for confirmation"
            );
            assert!(matches!(
                activity.state.security_pending,
                Some(super::super::security_state::PendingSecurityAction::EnableAttribute { .. })
            ));

            activity.apply_confirmation_action(
                super::super::security_confirmation_view::SecurityConfirmationAction::ConfirmPending,
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::AlterRole { name, attributes, .. }) => {
                    assert_eq!(name, "analyst");
                    assert_eq!(attributes.createdb, Some(true));
                }
                Ok(_) => panic!("confirm did not send AlterRole"),
                Err(error) => panic!("confirm sent no command: {error}"),
            }
            assert!(activity.state.security_pending.is_none());

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createdb: Some(false),
                        ..Default::default()
                    },
                )],
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::AlterRole { attributes, .. }) => {
                    assert_eq!(attributes.createdb, Some(false));
                }
                Ok(_) => panic!("turning CREATEDB off did not send AlterRole"),
                Err(error) => panic!("turning CREATEDB off sent no command: {error}"),
            }
            assert!(activity.state.security_pending.is_none());

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::RevokePrivilege(
                    db_pro_core::domain::user::Privilege {
                        object_kind: db_pro_core::domain::user::PrivilegeObjectKind::Table,
                        schema: "public".to_owned(),
                        object_name: "orders".to_owned(),
                        privilege_type: "SELECT".to_owned(),
                    },
                )],
            );
            assert!(matches!(
                command_rx.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
            activity.apply_confirmation_action(
                super::super::security_confirmation_view::SecurityConfirmationAction::ConfirmPending,
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::RevokePrivilege {
                    role_name,
                    object_name,
                    privilege,
                    ..
                }) => {
                    assert_eq!(role_name, "analyst");
                    assert_eq!(object_name, "orders");
                    assert_eq!(privilege, "SELECT");
                }
                Ok(_) => panic!("confirm did not send RevokePrivilege"),
                Err(error) => panic!("revoke confirm sent no command: {error}"),
            }

            activity.state.security_rls_restrictive = true;
            activity.state.security_rls_table = "orders".to_owned();
            activity.state.security_rls_policy_name = "tenant_iso".to_owned();
            activity.state.security_rls_using = "tenant_id = current_setting('app.tenant')".to_owned();
            activity.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Create);
            let sql = activity.state.security_rls_preview_sql.as_str();
            assert!(sql.contains("AS RESTRICTIVE"), "{sql}");
            assert!(!sql.contains("AS PERMISSIVE"), "{sql}");
        }

        super::request_security_role_details(
            &mut state,
            Some("conn-1"),
            "analyst",
            &mut dispatcher,
            &mut feedback,
        );
        let mut privilege_request = None;
        while let Ok(command) = command_rx.try_recv() {
            if let crate::UiCommand::ListPrivileges {
                request_id, role_name, ..
            } = command
            {
                assert_eq!(role_name, "analyst");
                privilege_request = Some(request_id.0);
            }
        }
        let request_id = privilege_request.expect("ListPrivileges was not requested");
        assert_eq!(state.security_privilege_request, Some(request_id));
        let message = "permission denied for relation usage_privileges";
        assert!(state.note_privilege_failure(request_id, message));
        assert_eq!(state.security_error.as_deref(), Some(message));
        assert!(state.security_privileges.is_empty());

        let privileges_view = include_str!("security_role_details_view.rs");
        let error_at = privileges_view
            .find("security_error")
            .expect("privilege view shows security_error");
        let empty_at = privileges_view
            .find("No privileges listed")
            .expect("empty privilege copy");
        assert!(error_at < empty_at);

        let manager = include_str!("../../../infrastructure/src/postgres/user_manager.rs");
        assert!(
            !manager.contains("return Vec::new()"),
            "privilege reads must return the query error"
        );
    }
}
