//! Pure PostgreSQL RLS mutation preview planning for the security surface.

use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::{
    MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationRequest, RlsPolicyDefinition, TableRlsDefinition,
};
use db_pro_core::ports::SqlDialect;

struct QuoteDialect;

impl SqlDialect for QuoteDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

pub(super) fn plan_table_rls(request: TableRlsPreviewRequest<'_>) -> Result<String, String> {
    let TableRlsPreviewRequest {
        schema,
        table,
        force,
        enable,
    } = request;
    let schema = schema.trim();
    let table = table.trim();
    if schema.is_empty() || table.is_empty() {
        return Err("Schema and table are required".to_owned());
    }
    plan(ObjectMutationRequest {
        action: if enable {
            ObjectAction::Enable
        } else {
            ObjectAction::Disable
        },
        target: None,
        definition: ObjectDefinition::TableRls(TableRlsDefinition {
            schema: schema.to_owned(),
            table: table.to_owned(),
            force,
        }),
        options: MutationOptions::default(),
        driver: "postgresql".into(),
    })
}

pub(super) struct TableRlsPreviewRequest<'a> {
    pub(super) schema: &'a str,
    pub(super) table: &'a str,
    pub(super) force: bool,
    pub(super) enable: bool,
}

pub(super) fn plan_policy(request: PolicyPreviewRequest<'_>) -> Result<String, String> {
    let PolicyPreviewRequest {
        action,
        schema,
        table,
        name,
        command,
        roles_csv,
        using_expr,
        with_check_expr,
    } = request;
    let schema = schema.trim();
    let table = table.trim();
    let name = name.trim();
    if schema.is_empty() || table.is_empty() || name.is_empty() {
        return Err("Schema, table, and policy name are required".to_owned());
    }
    let roles = roles_csv
        .split(',')
        .map(str::trim)
        .filter(|role| !role.is_empty())
        .map(str::to_owned)
        .collect();
    plan(ObjectMutationRequest {
        action,
        target: None,
        definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
            schema: schema.to_owned(),
            table: table.to_owned(),
            name: name.to_owned(),
            permissive: true,
            command: command.to_owned(),
            roles,
            using_expr: Some(using_expr.to_owned()).filter(|value| !value.trim().is_empty()),
            with_check_expr: Some(with_check_expr.to_owned()).filter(|value| !value.trim().is_empty()),
            new_name: None,
        }),
        options: MutationOptions::default(),
        driver: "postgresql".into(),
    })
}

pub(super) struct PolicyPreviewRequest<'a> {
    pub(super) action: ObjectAction,
    pub(super) schema: &'a str,
    pub(super) table: &'a str,
    pub(super) name: &'a str,
    pub(super) command: &'a str,
    pub(super) roles_csv: &'a str,
    pub(super) using_expr: &'a str,
    pub(super) with_check_expr: &'a str,
}

pub(super) fn plan_drop_policy(schema: &str, table: &str, name: &str) -> Result<String, String> {
    plan(ObjectMutationRequest {
        action: ObjectAction::Drop,
        target: None,
        definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
            schema: schema.trim().to_owned(),
            table: table.trim().to_owned(),
            name: name.to_owned(),
            permissive: true,
            command: "ALL".into(),
            roles: Vec::new(),
            using_expr: None,
            with_check_expr: None,
            new_name: None,
        }),
        options: MutationOptions {
            if_exists: true,
            ..MutationOptions::default()
        },
        driver: "postgresql".into(),
    })
}

fn plan(request: ObjectMutationRequest) -> Result<String, String> {
    let preview = ObjectMutationService::plan(&request, &QuoteDialect).map_err(|error| error.to_string())?;
    if let Some(reason) = preview.unsupported_reason {
        return Err(reason);
    }
    let mut sql = preview.statements.join(";\n");
    if !sql.is_empty() {
        sql.push(';');
    }
    Ok(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_rls_preview_rejects_missing_identity() {
        assert_eq!(
            plan_table_rls(TableRlsPreviewRequest {
                schema: "public",
                table: "",
                force: false,
                enable: true,
            }),
            Err("Schema and table are required".into())
        );
    }

    #[test]
    fn policy_preview_quotes_identifiers_and_keeps_explicit_roles() {
        let sql = plan_policy(PolicyPreviewRequest {
            action: ObjectAction::Create,
            schema: "public",
            table: "orders",
            name: "orders_read",
            command: "SELECT",
            roles_csv: "app_user, reporting",
            using_expr: "tenant_id = current_setting('app.tenant')::int",
            with_check_expr: "",
        })
        .expect("preview");

        assert!(sql.contains("CREATE POLICY"));
        assert!(sql.contains("app_user"));
        assert!(sql.ends_with(';'));
    }
}
