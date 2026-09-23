//! PostgreSQL row-level-security presentation and user intents.

use super::security_state::SecurityState;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SecurityRlsAction {
    Inspect,
    PreviewTable {
        force: bool,
        enable: bool,
    },
    DropPolicy(String),
    LoadPolicy {
        name: String,
        command: String,
        roles: String,
        using_expression: String,
        with_check: String,
    },
    PreviewPolicy(db_pro_core::domain::object_mutation::ObjectAction),
    ApplyPreview,
}

pub(super) struct SecurityRlsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut SecurityState,
}

impl SecurityRlsContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRlsAction> {
        let mut actions = Vec::new();
        self.draw_header(ui);
        actions.extend(self.draw_controls(ui));
        actions.extend(self.draw_current_state(ui));
        actions.extend(self.draw_policy_form(ui));
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        section_label(ui, "ROW-LEVEL SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Policy changes are administrative — preview SQL, then confirm apply.")
                .small()
                .color(self.theme.text_muted),
        );
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRlsAction> {
        let mut actions = Vec::new();
        input_full_width(ui, &mut self.state.security_rls_schema, "schema", self.theme);
        input_full_width(ui, &mut self.state.security_rls_table, "table", self.theme);
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Inspect RLS", self.theme).clicked() {
                actions.push(SecurityRlsAction::Inspect);
            }
            if secondary_button_with_icon(ui, Icon::Shield, "Enable RLS", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewTable {
                    force: false,
                    enable: true,
                });
            }
            if secondary_button_with_icon(ui, Icon::ShieldOff, "Disable RLS", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewTable {
                    force: false,
                    enable: false,
                });
            }
            if secondary_button_with_icon(ui, Icon::Lock, "Force RLS", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewTable {
                    force: true,
                    enable: true,
                });
            }
            if secondary_button_with_icon(ui, Icon::Unlock, "No Force", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewTable {
                    force: true,
                    enable: false,
                });
            }
        });
        actions
    }

    fn draw_current_state(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRlsAction> {
        let mut actions = Vec::new();
        let Some(state) = self.state.security_rls_state.clone() else {
            return actions;
        };
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
            actions.extend(self.draw_policy(ui, policy));
        }
        actions
    }

    fn draw_policy(&self, ui: &mut egui::Ui, policy: db_pro_core::domain::rls::RlsPolicy) -> Vec<SecurityRlsAction> {
        let mut actions = Vec::new();
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
                actions.push(SecurityRlsAction::DropPolicy(policy.name.clone()));
            }
            if secondary_button_with_icon(ui, Icon::Pencil, "Load for edit", self.theme).clicked() {
                actions.push(SecurityRlsAction::LoadPolicy {
                    name: policy.name.clone(),
                    command: policy.command.clone(),
                    roles: policy.roles.join(", "),
                    using_expression: policy.using_expr.clone().unwrap_or_default(),
                    with_check: policy.with_check_expr.clone().unwrap_or_default(),
                });
            }
        });
        actions
    }

    fn draw_policy_form(&mut self, ui: &mut egui::Ui) -> Vec<SecurityRlsAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_SM);
        section_label(ui, "CREATE / ALTER POLICY", self.theme);
        input_full_width(ui, &mut self.state.security_rls_policy_name, "policy name", self.theme);
        input_full_width(
            ui,
            &mut self.state.security_rls_command,
            "command (ALL/SELECT/INSERT/UPDATE/DELETE)",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.state.security_rls_roles,
            "roles (comma; empty=PUBLIC)",
            self.theme,
        );
        input_full_width(ui, &mut self.state.security_rls_using, "USING expression", self.theme);
        input_full_width(
            ui,
            &mut self.state.security_rls_with_check,
            "WITH CHECK expression",
            self.theme,
        );
        ui.horizontal(|ui| {
            if primary_button_with_icon(ui, Icon::Eye, "Preview CREATE", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewPolicy(
                    db_pro_core::domain::object_mutation::ObjectAction::Create,
                ));
            }
            if secondary_button_with_icon(ui, Icon::Pencil, "Preview ALTER", self.theme).clicked() {
                actions.push(SecurityRlsAction::PreviewPolicy(
                    db_pro_core::domain::object_mutation::ObjectAction::Alter,
                ));
            }
        });
        if !self.state.security_rls_preview_sql.is_empty() {
            ui.label(
                RichText::new(&self.state.security_rls_preview_sql)
                    .small()
                    .monospace()
                    .color(self.theme.text_primary),
            );
            ui.checkbox(
                &mut self.state.security_rls_confirm_apply,
                "I understand this changes data visibility immediately",
            );
            if primary_button_with_icon(ui, Icon::Play, "Apply preview SQL", self.theme).clicked() {
                actions.push(SecurityRlsAction::ApplyPreview);
            }
        }
        actions
    }
}
