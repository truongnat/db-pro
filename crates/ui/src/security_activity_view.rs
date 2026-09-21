use super::*;

impl DbProApp {
    pub(super) fn draw_security_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let is_pg = self.active_driver().eq_ignore_ascii_case("postgresql")
            || self.active_driver().eq_ignore_ascii_case("postgres");

        if !connected {
            ui.label(
                RichText::new("Connect a PostgreSQL database to manage roles and privileges.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }
        if !is_pg {
            ui.label(
                RichText::new("User/role management is PostgreSQL-only (capability gated).")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }

        if let Some(error) = &self.management.security.security_error {
            ui.colored_label(self.theme.warning, error);
        }

        let role_actions = security_roles_view::SecurityRolesContext {
            theme: self.theme,
            state: &mut self.management.security,
        }
        .draw(ui);
        self.apply_security_roles_actions(role_actions);

        if let Some(role) = self.management.security.security_selected_role.clone() {
            ui.add_space(SPACE_MD);
            section_label(ui, format!("ATTRIBUTES · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                if secondary_button_with_icon(ui, Icon::Check, "LOGIN", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            login: Some(true),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::X, "NOLOGIN", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            login: Some(false),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::Database, "CREATEDB", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            createdb: Some(true),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::Users, "CREATEROLE", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            createrole: Some(true),
                            ..Default::default()
                        },
                    );
                }
            });

            ui.add_space(SPACE_MD);
            section_label(ui, format!("PASSWORD · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new("Password is never logged or shown in runtime events.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.management.security.security_password)
                    .password(true)
                    .hint_text("new password")
                    .desired_width(f32::INFINITY),
            );
            if primary_button_with_icon(ui, Icon::Key, "Update password", self.theme).clicked()
                && !self.management.security.security_password.is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let password = std::mem::take(&mut self.management.security.security_password);
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.management.security.update_password_command(
                        request_id,
                        connection_id,
                        role.clone(),
                        password,
                    ));
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, format!("MEMBERSHIPS · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            if self.management.security.security_memberships.is_empty() {
                ui.label(
                    RichText::new("No role memberships.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for membership in self.management.security.security_memberships.clone() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("member of {}", membership.role))
                                .small()
                                .monospace()
                                .color(self.theme.text_secondary),
                        );
                        if danger_button(ui, "Revoke", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(self.management.security.revoke_membership_command(
                                    request_id,
                                    connection_id,
                                    membership.role,
                                    role.clone(),
                                ));
                            }
                        }
                    });
                }
            }
            input_full_width(
                ui,
                &mut self.management.security.security_membership_role,
                "grant role name",
                self.theme,
            );
            if secondary_button_with_icon(ui, Icon::Plus, "Grant membership", self.theme).clicked()
                && !self.management.security.security_membership_role.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.management.security.grant_membership_command(
                        request_id,
                        connection_id,
                        self.management.security.security_membership_role.trim().to_owned(),
                        role.clone(),
                    ));
                    self.management.security.security_membership_role.clear();
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, format!("PRIVILEGES · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            if self.management.security.security_privileges.is_empty() {
                ui.label(
                    RichText::new("No privileges listed for this role.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for privs in self.management.security.security_privileges.clone() {
                    ui.horizontal(|ui| {
                        let target = match privs.object_kind {
                            db_pro_core::domain::user::PrivilegeObjectKind::Database => privs.object_name.clone(),
                            db_pro_core::domain::user::PrivilegeObjectKind::Schema => privs.object_name.clone(),
                            _ => format!("{}.{}", privs.schema, privs.object_name),
                        };
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                privs.object_kind.as_label(),
                                target,
                                privs.privilege_type
                            ))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                        );
                        if danger_button(ui, "Revoke", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(self.management.security.revoke_privilege_command(
                                    request_id,
                                    connection_id,
                                    role.clone(),
                                    privs,
                                ));
                            }
                        }
                    });
                }
            }

            ui.add_space(SPACE_SM);
            section_label(ui, "GRANT PRIVILEGE", self.theme);
            ui.horizontal(|ui| {
                for (label, kind) in [
                    ("table", db_pro_core::domain::user::PrivilegeObjectKind::Table),
                    ("schema", db_pro_core::domain::user::PrivilegeObjectKind::Schema),
                    ("database", db_pro_core::domain::user::PrivilegeObjectKind::Database),
                    ("sequence", db_pro_core::domain::user::PrivilegeObjectKind::Sequence),
                ] {
                    if ui
                        .selectable_label(self.management.security.security_grant_kind == kind, label)
                        .clicked()
                    {
                        self.management.security.security_grant_kind = kind;
                    }
                }
            });
            if !matches!(
                self.management.security.security_grant_kind,
                db_pro_core::domain::user::PrivilegeObjectKind::Database
                    | db_pro_core::domain::user::PrivilegeObjectKind::Schema
            ) {
                input_full_width(
                    ui,
                    &mut self.management.security.security_grant_schema,
                    "schema",
                    self.theme,
                );
            }
            let object_hint = match self.management.security.security_grant_kind {
                db_pro_core::domain::user::PrivilegeObjectKind::Table => "table",
                db_pro_core::domain::user::PrivilegeObjectKind::Schema => "schema",
                db_pro_core::domain::user::PrivilegeObjectKind::Database => "database",
                db_pro_core::domain::user::PrivilegeObjectKind::Sequence => "sequence",
            };
            input_full_width(
                ui,
                &mut self.management.security.security_grant_object,
                object_hint,
                self.theme,
            );
            input_full_width(
                ui,
                &mut self.management.security.security_grant_privilege,
                "privilege (SELECT/USAGE/CONNECT/…)",
                self.theme,
            );
            if primary_button_with_icon(ui, Icon::Plus, "Grant privilege", self.theme).clicked()
                && !self.management.security.security_grant_object.trim().is_empty()
                && !self.management.security.security_grant_privilege.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.management.security.grant_privilege_command(
                        request_id,
                        connection_id,
                        role,
                    ));
                }
            }
        }

        if let Some(name) = self.management.security.security_drop_confirm.clone() {
            egui::Window::new("Drop role?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Drop role `{name}`? This cannot be undone."));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop role", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(self.management.security.drop_role_command(
                                    request_id,
                                    connection_id,
                                    name,
                                ));
                            }
                            self.management.security.security_drop_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.management.security.security_drop_confirm = None;
                        }
                    });
                });
        }

        let rls_actions = security_rls_view::SecurityRlsContext {
            theme: self.theme,
            state: &mut self.management.security,
        }
        .draw(ui);
        self.apply_security_rls_actions(rls_actions);
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
            self.dispatch_command(command);
            self.table.state.ddl_execution_request = Some(request_id);
            self.feedback.runtime_message = "Applying RLS mutation…".into();
        }
    }

    fn dispatch_alter_role(&mut self, name: &str, attributes: db_pro_core::domain::user::RoleAttributes) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.security.alter_role_command(
            request_id,
            connection_id,
            name.to_owned(),
            attributes,
        ));
    }
}
