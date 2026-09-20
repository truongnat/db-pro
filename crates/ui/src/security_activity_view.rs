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

        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh roles", self.theme).clicked() {
                self.request_security_users();
            }
        });
        if let Some(error) = &self.security.security_error {
            ui.colored_label(self.theme.warning, error);
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "ROLES / USERS", self.theme);
        ui.add_space(SPACE_SM);
        if self.security.security_users.is_empty() {
            ui.label(
                RichText::new("No roles loaded yet — click Refresh.")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        for user in self.security.security_users.clone() {
            let selected = self.security.security_selected_role.as_deref() == Some(user.name.as_str());
            ui.horizontal(|ui| {
                if ui.selectable_label(selected, &user.name).clicked() {
                    self.security.security_selected_role = Some(user.name.clone());
                    self.request_security_role_details(&user.name);
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
                    self.security.security_drop_confirm = Some(user.name.clone());
                }
            });
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "CREATE ROLE", self.theme);
        ui.add_space(SPACE_SM);
        input_full_width(ui, &mut self.security.security_new_role, "role name", self.theme);
        ui.checkbox(&mut self.security.security_new_role_login, "LOGIN");
        if primary_button_with_icon(ui, Icon::Plus, "Create role", self.theme).clicked()
            && !self.security.security_new_role.trim().is_empty()
        {
            if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(self.security.create_role_command(
                    request_id,
                    connection_id,
                    self.security.security_new_role.trim().to_owned(),
                ));
                self.security.security_new_role.clear();
            }
        }

        if let Some(role) = self.security.security_selected_role.clone() {
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
                egui::TextEdit::singleline(&mut self.security.security_password)
                    .password(true)
                    .hint_text("new password")
                    .desired_width(f32::INFINITY),
            );
            if primary_button_with_icon(ui, Icon::Key, "Update password", self.theme).clicked()
                && !self.security.security_password.is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let password = std::mem::take(&mut self.security.security_password);
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.security.update_password_command(
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
            if self.security.security_memberships.is_empty() {
                ui.label(
                    RichText::new("No role memberships.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for membership in self.security.security_memberships.clone() {
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
                                self.dispatch_command(self.security.revoke_membership_command(
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
                &mut self.security.security_membership_role,
                "grant role name",
                self.theme,
            );
            if secondary_button_with_icon(ui, Icon::Plus, "Grant membership", self.theme).clicked()
                && !self.security.security_membership_role.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.security.grant_membership_command(
                        request_id,
                        connection_id,
                        self.security.security_membership_role.trim().to_owned(),
                        role.clone(),
                    ));
                    self.security.security_membership_role.clear();
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, format!("PRIVILEGES · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            if self.security.security_privileges.is_empty() {
                ui.label(
                    RichText::new("No privileges listed for this role.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for privs in self.security.security_privileges.clone() {
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
                                self.dispatch_command(self.security.revoke_privilege_command(
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
                        .selectable_label(self.security.security_grant_kind == kind, label)
                        .clicked()
                    {
                        self.security.security_grant_kind = kind;
                    }
                }
            });
            if !matches!(
                self.security.security_grant_kind,
                db_pro_core::domain::user::PrivilegeObjectKind::Database
                    | db_pro_core::domain::user::PrivilegeObjectKind::Schema
            ) {
                input_full_width(ui, &mut self.security.security_grant_schema, "schema", self.theme);
            }
            let object_hint = match self.security.security_grant_kind {
                db_pro_core::domain::user::PrivilegeObjectKind::Table => "table",
                db_pro_core::domain::user::PrivilegeObjectKind::Schema => "schema",
                db_pro_core::domain::user::PrivilegeObjectKind::Database => "database",
                db_pro_core::domain::user::PrivilegeObjectKind::Sequence => "sequence",
            };
            input_full_width(ui, &mut self.security.security_grant_object, object_hint, self.theme);
            input_full_width(
                ui,
                &mut self.security.security_grant_privilege,
                "privilege (SELECT/USAGE/CONNECT/…)",
                self.theme,
            );
            if primary_button_with_icon(ui, Icon::Plus, "Grant privilege", self.theme).clicked()
                && !self.security.security_grant_object.trim().is_empty()
                && !self.security.security_grant_privilege.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.security.grant_privilege_command(request_id, connection_id, role));
                }
            }
        }

        if let Some(name) = self.security.security_drop_confirm.clone() {
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
                                self.dispatch_command(self.security.drop_role_command(request_id, connection_id, name));
                            }
                            self.security.security_drop_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.security.security_drop_confirm = None;
                        }
                    });
                });
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "ROW-LEVEL SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Policy changes are administrative — preview SQL, then confirm apply.")
                .small()
                .color(self.theme.text_muted),
        );
        input_full_width(ui, &mut self.security.security_rls_schema, "schema", self.theme);
        input_full_width(ui, &mut self.security.security_rls_table, "table", self.theme);
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Inspect RLS", self.theme).clicked() {
                self.request_security_rls();
            }
            if secondary_button_with_icon(ui, Icon::Shield, "Enable RLS", self.theme).clicked() {
                self.preview_table_rls(false, true);
            }
            if secondary_button_with_icon(ui, Icon::ShieldOff, "Disable RLS", self.theme).clicked() {
                self.preview_table_rls(false, false);
            }
            if secondary_button_with_icon(ui, Icon::Lock, "Force RLS", self.theme).clicked() {
                self.preview_table_rls(true, true);
            }
            if secondary_button_with_icon(ui, Icon::Unlock, "No Force", self.theme).clicked() {
                self.preview_table_rls(true, false);
            }
        });
        if let Some(state) = self.security.security_rls_state.clone() {
            ui.label(
                RichText::new(format!(
                    "{}.{} · enabled={} · forced={} · {} policy(ies)",
                    state.schema,
                    state.table,
                    state.rls_enabled,
                    state.rls_forced,
                    state.policies.len()
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
            for policy in state.policies {
                let roles = if policy.roles.is_empty() {
                    "PUBLIC".to_owned()
                } else {
                    policy.roles.join(", ")
                };
                ui.label(
                    RichText::new(format!(
                        "{} · {} · {} · roles[{}] · USING({}) · CHECK({})",
                        policy.name,
                        policy.command,
                        if policy.permissive { "PERMISSIVE" } else { "RESTRICTIVE" },
                        roles,
                        policy.using_expr.as_deref().unwrap_or("—"),
                        policy.with_check_expr.as_deref().unwrap_or("—"),
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
                );
                ui.horizontal(|ui| {
                    if danger_button(ui, "Drop policy", self.theme).clicked() {
                        self.preview_drop_rls_policy(&policy.name);
                    }
                    if secondary_button_with_icon(ui, Icon::Pencil, "Load for edit", self.theme).clicked() {
                        self.security.security_rls_policy_name = policy.name.clone();
                        self.security.security_rls_command = policy.command.clone();
                        self.security.security_rls_roles = policy.roles.join(", ");
                        self.security.security_rls_using = policy.using_expr.clone().unwrap_or_default();
                        self.security.security_rls_with_check = policy.with_check_expr.clone().unwrap_or_default();
                    }
                });
            }
        }
        ui.add_space(SPACE_SM);
        section_label(ui, "CREATE / ALTER POLICY", self.theme);
        input_full_width(
            ui,
            &mut self.security.security_rls_policy_name,
            "policy name",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_command,
            "command (ALL/SELECT/INSERT/UPDATE/DELETE)",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_roles,
            "roles (comma; empty=PUBLIC)",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_using,
            "USING expression",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_with_check,
            "WITH CHECK expression",
            self.theme,
        );
        ui.horizontal(|ui| {
            if primary_button_with_icon(ui, Icon::Eye, "Preview CREATE", self.theme).clicked() {
                self.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Create);
            }
            if secondary_button_with_icon(ui, Icon::Pencil, "Preview ALTER", self.theme).clicked() {
                self.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Alter);
            }
        });
        if !self.security.security_rls_preview_sql.is_empty() {
            ui.label(
                RichText::new(&self.security.security_rls_preview_sql)
                    .small()
                    .monospace()
                    .color(self.theme.text_primary),
            );
            ui.checkbox(
                &mut self.security.security_rls_confirm_apply,
                "I understand this changes data visibility immediately",
            );
            if primary_button_with_icon(ui, Icon::Play, "Apply preview SQL", self.theme).clicked() {
                if !self.security.security_rls_confirm_apply {
                    self.feedback.runtime_message = "Confirm RLS apply checkbox first".into();
                } else {
                    self.apply_security_rls_preview();
                }
            }
        }
    }

    pub(crate) fn request_security_users(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.security.list_users_command(request_id, connection_id));
    }

    pub(crate) fn request_security_role_details(&mut self, role_name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.security.list_privileges_command(
            request_id,
            connection_id.clone(),
            role_name.to_owned(),
        ));
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.security
                .list_memberships_command(request_id, connection_id, role_name.to_owned()),
        );
    }

    pub(crate) fn request_security_rls(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        match self.security.list_table_rls_command(request_id, connection_id) {
            Ok(command) => {
                self.dispatch_command(command);
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn preview_table_rls(&mut self, force: bool, enable: bool) {
        match security_rls::plan_table_rls(security_rls::TableRlsPreviewRequest {
            schema: &self.security.security_rls_schema,
            table: &self.security.security_rls_table,
            force,
            enable,
        }) {
            Ok(sql) => {
                self.security.security_rls_preview_sql = sql;
                self.security.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    fn preview_rls_policy(&mut self, action: db_pro_core::domain::object_mutation::ObjectAction) {
        match security_rls::plan_policy(security_rls::PolicyPreviewRequest {
            action,
            schema: &self.security.security_rls_schema,
            table: &self.security.security_rls_table,
            name: &self.security.security_rls_policy_name,
            command: &self.security.security_rls_command,
            roles_csv: &self.security.security_rls_roles,
            using_expr: &self.security.security_rls_using,
            with_check_expr: &self.security.security_rls_with_check,
        }) {
            Ok(sql) => {
                self.security.security_rls_preview_sql = sql;
                self.security.security_rls_confirm_apply = false;
            }
            Err(error) => {
                self.security.security_rls_preview_sql.clear();
                self.feedback.runtime_message = error;
            }
        }
    }

    fn preview_drop_rls_policy(&mut self, policy_name: &str) {
        match security_rls::plan_drop_policy(
            &self.security.security_rls_schema,
            &self.security.security_rls_table,
            policy_name,
        ) {
            Ok(sql) => {
                self.security.security_rls_preview_sql = sql;
                self.security.security_rls_confirm_apply = false;
            }
            Err(error) => self.feedback.runtime_message = error,
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
        if let Some(command) = self.security.apply_rls_preview_command(request_id, connection_id) {
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
        self.dispatch_command(
            self.security
                .alter_role_command(request_id, connection_id, name.to_owned(), attributes),
        );
    }
}
