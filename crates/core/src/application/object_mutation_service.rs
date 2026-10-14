//! Object mutation plan/apply service (#183 / Phase A).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::domain::object_mutation::*;
use crate::ports::SqlDialect;

#[path = "object_mutation_builders.rs"]
mod object_mutation_builders;

pub struct ObjectMutationService;

impl ObjectMutationService {
    pub fn plan(
        request: &ObjectMutationRequest,
        dialect: &dyn SqlDialect,
    ) -> Result<ObjectMutationPreview, MutationError> {
        if let Some(reason) = unsupported_reason(request) {
            return Ok(ObjectMutationPreview {
                statements: Vec::new(),
                safety: "blocked".into(),
                long_running: false,
                effects: vec![reason.clone()],
                fingerprint: fingerprint(&[]),
                unsupported_reason: Some(reason),
            });
        }

        let statements = object_mutation_builders::build_statements(request, dialect)?;
        let safety = classify_safety(request);
        let long_running = matches!(
            request.definition,
            ObjectDefinition::Table(_) | ObjectDefinition::Index(_) | ObjectDefinition::Partition(_)
        ) && matches!(
            request.action,
            ObjectAction::Create | ObjectAction::Alter | ObjectAction::Drop
        ) || matches!(
            (&request.definition, request.action),
            (ObjectDefinition::Column(_), ObjectAction::Alter)
        );
        let effects = describe_effects(request);
        let fingerprint = fingerprint(&statements);
        Ok(ObjectMutationPreview {
            statements,
            safety,
            long_running,
            effects,
            fingerprint,
            unsupported_reason: None,
        })
    }

    /// Plan using the dialect of `request.driver`, not a fixed quote style.
    pub fn plan_for_driver(request: &ObjectMutationRequest) -> Result<ObjectMutationPreview, MutationError> {
        let dialect = crate::application::driver_dialect::DriverDialect::from_name(&request.driver);
        Self::plan(request, &dialect)
    }
}

fn unsupported_reason(request: &ObjectMutationRequest) -> Option<String> {
    let driver = request.driver.to_ascii_lowercase();
    if matches!(
        (&request.definition, request.action),
        (
            ObjectDefinition::Trigger(_),
            ObjectAction::Enable | ObjectAction::Disable
        )
    ) && !(driver.contains("postgres") || driver == "pg")
    {
        let reason = if driver.contains("sqlite") {
            "SQLite cannot enable or disable individual triggers"
        } else if driver.contains("mysql") {
            "MySQL does not support enabling or disabling individual triggers"
        } else {
            "trigger enable/disable is not enabled for this provider"
        };
        return Some(reason.into());
    }
    if driver.contains("sqlite") {
        return sqlite_unsupported_reason(request);
    }
    if driver.contains("postgres") || driver == "pg" {
        return None;
    }
    if matches!(
        request.definition,
        ObjectDefinition::RlsPolicy(_) | ObjectDefinition::TableRls(_)
    ) {
        return Some("row-level security policies require PostgreSQL".into());
    }
    None
}

/// SQLite has no catalog objects beyond tables/views/triggers — every row here
/// is a documented "unsupported" answer for the workbench.
fn sqlite_unsupported_reason(request: &ObjectMutationRequest) -> Option<String> {
    match (&request.definition, request.action) {
        (ObjectDefinition::MaterializedView(_), _) => Some("SQLite does not support materialized views".into()),
        (ObjectDefinition::Sequence(_), _) => Some("SQLite does not support CREATE SEQUENCE".into()),
        (ObjectDefinition::EnumType(_), _) => Some("SQLite does not support CREATE TYPE ... AS ENUM".into()),
        (ObjectDefinition::DomainType(_), _) => Some("SQLite does not support CREATE DOMAIN".into()),
        (ObjectDefinition::Extension(_), _) => Some("SQLite does not support CREATE EXTENSION".into()),
        (ObjectDefinition::Partition(_), _) => Some("SQLite does not support table partitions".into()),
        (ObjectDefinition::Routine(_), _) => Some("SQLite does not support stored functions/procedures".into()),
        (ObjectDefinition::RlsPolicy(_) | ObjectDefinition::TableRls(_), _) => {
            Some("SQLite does not support row-level security policies".into())
        }
        (ObjectDefinition::Schema(_), ObjectAction::Create | ObjectAction::Drop) => {
            Some("SQLite has no CREATE/DROP SCHEMA".into())
        }
        (ObjectDefinition::Database(_), _) => {
            Some("SQLite CREATE/DROP DATABASE is not supported in this workbench".into())
        }
        (ObjectDefinition::Comment(_), _) => Some("SQLite COMMENT ON is not supported".into()),
        (ObjectDefinition::Column(_), ObjectAction::Alter) => {
            Some("SQLite cannot ALTER COLUMN type, nullability, or default without a table rebuild".into())
        }
        _ => None,
    }
}

fn classify_safety(request: &ObjectMutationRequest) -> String {
    match &request.definition {
        ObjectDefinition::RlsPolicy(_) | ObjectDefinition::TableRls(_) => "administrative".into(),
        _ => match request.action {
            ObjectAction::Drop => "destructive".into(),
            ObjectAction::Alter | ObjectAction::Rename => "mutating".into(),
            ObjectAction::Create | ObjectAction::Refresh | ObjectAction::Comment => "mutating".into(),
            ObjectAction::Enable | ObjectAction::Disable | ObjectAction::GenerateDdl => "safe".into(),
        },
    }
}

fn describe_effects(request: &ObjectMutationRequest) -> Vec<String> {
    match &request.definition {
        ObjectDefinition::RlsPolicy(def) => vec![format!(
            "{:?} RLS policy {}.{} · {}",
            request.action, def.schema, def.table, def.name
        )],
        ObjectDefinition::TableRls(def) => vec![format!(
            "{:?} row-level security on {}.{} (force={})",
            request.action, def.schema, def.table, def.force
        )],
        _ => vec![format!(
            "{:?} on {:?}",
            request.action,
            request.target.as_ref().map(|t| t.name.as_str()).unwrap_or("<new>")
        )],
    }
}

fn fingerprint(statements: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    for stmt in statements {
        stmt.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::SqlDialect;

    struct PgDialect;
    impl SqlDialect for PgDialect {
        fn placeholder(&self, index: usize) -> String {
            format!("${index}")
        }
        fn quote_identifier(&self, name: &str) -> String {
            format!("\"{}\"", name.replace('"', "\"\""))
        }
    }
    fn trigger_request(driver: &str, action: ObjectAction) -> ObjectMutationRequest {
        ObjectMutationRequest {
            action,
            target: None,
            definition: ObjectDefinition::Trigger(TriggerDefinition {
                schema: "odd schema".into(),
                table: "order\"items".into(),
                name: "trig\"ger".into(),
                timing: String::new(),
                event: String::new(),
                body: String::new(),
            }),
            options: MutationOptions::default(),
            driver: driver.into(),
        }
    }

    #[test]
    fn plans_postgres_trigger_enable_and_disable_with_quoted_identifiers() {
        for (action, keyword) in [(ObjectAction::Enable, "ENABLE"), (ObjectAction::Disable, "DISABLE")] {
            let preview = ObjectMutationService::plan(&trigger_request("postgresql", action), &PgDialect).unwrap();
            assert_eq!(
                preview.statements,
                vec![format!(
                    "ALTER TABLE \"odd schema\".\"order\"\"items\" {keyword} TRIGGER \"trig\"\"ger\""
                )]
            );
            assert!(preview.unsupported_reason.is_none());
        }
    }

    #[test]
    fn sqlite_and_mysql_trigger_toggle_plans_emit_no_sql() {
        for (driver, reason) in [
            ("sqlite", "SQLite cannot enable or disable individual triggers"),
            (
                "mysql",
                "MySQL does not support enabling or disabling individual triggers",
            ),
        ] {
            let preview =
                ObjectMutationService::plan(&trigger_request(driver, ObjectAction::Disable), &PgDialect).unwrap();
            assert!(preview.statements.is_empty());
            assert_eq!(preview.unsupported_reason.as_deref(), Some(reason));
        }
    }

    #[test]
    fn plans_create_table() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: None,
            definition: ObjectDefinition::Table(TableDefinition {
                schema: "public".into(),
                name: "t".into(),
                columns: vec![ColumnDefinition {
                    schema: "public".into(),
                    table: "t".into(),
                    name: "id".into(),
                    data_type: "INTEGER".into(),
                    nullable: false,
                    default: None,
                    is_pk: true,
                    new_name: None,
                    origin_name: None,
                }],
                baseline: Vec::new(),
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_none());
        assert!(preview.statements[0].contains("CREATE TABLE"));
        assert!(!preview.statements[0].to_ascii_uppercase().contains("IF NOT EXISTS"));
    }

    #[test]
    fn editing_an_existing_table_emits_alter_not_create() {
        let baseline = vec![
            ColumnDefinition {
                schema: "public".into(),
                table: "orders".into(),
                name: "id".into(),
                data_type: "integer".into(),
                nullable: false,
                default: None,
                is_pk: true,
                new_name: None,
                origin_name: None,
            },
            ColumnDefinition {
                schema: "public".into(),
                table: "orders".into(),
                name: "label".into(),
                data_type: "text".into(),
                nullable: true,
                default: None,
                is_pk: false,
                new_name: None,
                origin_name: None,
            },
        ];
        let req = ObjectMutationRequest {
            action: ObjectAction::Alter,
            target: None,
            definition: ObjectDefinition::Table(TableDefinition {
                schema: "public".into(),
                name: "orders".into(),
                columns: vec![
                    ColumnDefinition {
                        schema: "public".into(),
                        table: "orders".into(),
                        name: "id".into(),
                        data_type: "bigint".into(),
                        nullable: false,
                        default: None,
                        is_pk: true,
                        new_name: None,
                        origin_name: Some("id".into()),
                    },
                    ColumnDefinition {
                        schema: "public".into(),
                        table: "orders".into(),
                        name: "title".into(),
                        data_type: "text".into(),
                        nullable: false,
                        default: Some("'x'".into()),
                        is_pk: false,
                        new_name: None,
                        origin_name: Some("label".into()),
                    },
                ],
                baseline,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        let sql = preview.statements.join("\n");
        let upper = sql.to_ascii_uppercase();
        assert!(!upper.contains("CREATE TABLE"), "{sql}");
        assert!(!upper.contains("IF NOT EXISTS"), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"id\" TYPE bigint"), "{sql}");
        assert!(sql.contains("RENAME COLUMN \"label\" TO \"title\""), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"title\" SET NOT NULL"), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"title\" SET DEFAULT 'x'"), "{sql}");
    }

    #[test]
    fn plan_for_driver_quotes_mysql_identifiers_with_backticks() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: None,
            definition: ObjectDefinition::Table(TableDefinition {
                schema: "app".into(),
                name: "widgets".into(),
                columns: vec![ColumnDefinition {
                    schema: "app".into(),
                    table: "widgets".into(),
                    name: "id".into(),
                    data_type: "int".into(),
                    nullable: false,
                    default: None,
                    is_pk: true,
                    new_name: None,
                    origin_name: None,
                }],
                baseline: Vec::new(),
            }),
            options: MutationOptions::default(),
            driver: "mysql".into(),
        };
        let preview = ObjectMutationService::plan_for_driver(&req).unwrap();
        let sql = preview.statements.join("\n");
        assert!(sql.contains("`app`.`widgets`"), "{sql}");
        assert!(sql.contains("`id`"), "{sql}");
        assert!(!sql.contains("\"widgets\""), "{sql}");
    }

    #[test]
    fn blocks_sequence_on_sqlite() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: None,
            definition: ObjectDefinition::Sequence(SequenceDefinition {
                schema: "main".into(),
                name: "s".into(),
                start: Some(1),
                increment: Some(1),
                min_value: None,
                max_value: None,
                cache: None,
                cycle: false,
            }),
            options: MutationOptions::default(),
            driver: "sqlite".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_some());
    }

    #[test]
    fn plans_create_or_replace_and_drop_routine() {
        let create = ObjectMutationRequest {
            action: ObjectAction::Alter,
            target: None,
            definition: ObjectDefinition::Routine(RoutineDefinition {
                schema: "public".into(),
                name: "calc".into(),
                routine_type: "FUNCTION".into(),
                identity_arguments: "x integer".into(),
                definition_sql:
                    "CREATE FUNCTION public.calc(x integer) RETURNS integer LANGUAGE sql AS $$ SELECT x; $$;".into(),
                replace: true,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&create, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_none());
        assert!(preview.statements[0].contains("CREATE OR REPLACE FUNCTION"));

        let drop = ObjectMutationRequest {
            action: ObjectAction::Drop,
            target: None,
            definition: ObjectDefinition::Routine(RoutineDefinition {
                schema: "public".into(),
                name: "calc".into(),
                routine_type: "FUNCTION".into(),
                identity_arguments: "x integer".into(),
                definition_sql: String::new(),
                replace: false,
            }),
            options: MutationOptions {
                if_exists: true,
                ..MutationOptions::default()
            },
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&drop, &PgDialect).unwrap();
        assert!(preview.statements[0].contains("DROP FUNCTION IF EXISTS"));
        assert!(preview.statements[0].contains("\"calc\"(x integer)"));
    }

    #[test]
    fn blocks_routine_on_sqlite() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Drop,
            target: None,
            definition: ObjectDefinition::Routine(RoutineDefinition {
                schema: "main".into(),
                name: "f".into(),
                routine_type: "FUNCTION".into(),
                identity_arguments: String::new(),
                definition_sql: String::new(),
                replace: false,
            }),
            options: MutationOptions::default(),
            driver: "sqlite".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_some());
    }

    #[test]
    fn plans_rls_policy_create_preserving_expressions() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: None,
            definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
                schema: "public".into(),
                table: "orders".into(),
                name: "tenant_select".into(),
                permissive: true,
                command: "SELECT".into(),
                roles: vec!["app_reader".into()],
                using_expr: Some("tenant_id = current_setting('app.tenant')::uuid".into()),
                with_check_expr: None,
                new_name: None,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert_eq!(preview.safety, "administrative");
        assert!(preview.statements[0].contains("CREATE POLICY \"tenant_select\""));
        assert!(preview.statements[0].contains("USING (tenant_id = current_setting('app.tenant')::uuid)"));
        assert!(!preview.statements[0].contains("::UUID")); // no rewrite
    }

    #[test]
    fn plans_table_rls_enable_and_force() {
        let enable = ObjectMutationRequest {
            action: ObjectAction::Enable,
            target: None,
            definition: ObjectDefinition::TableRls(TableRlsDefinition {
                schema: "public".into(),
                table: "orders".into(),
                force: false,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&enable, &PgDialect).unwrap();
        assert_eq!(
            preview.statements[0],
            "ALTER TABLE \"public\".\"orders\" ENABLE ROW LEVEL SECURITY"
        );

        let force = ObjectMutationRequest {
            action: ObjectAction::Enable,
            target: None,
            definition: ObjectDefinition::TableRls(TableRlsDefinition {
                schema: "public".into(),
                table: "orders".into(),
                force: true,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&force, &PgDialect).unwrap();
        assert!(preview.statements[0].contains("FORCE ROW LEVEL SECURITY"));
    }

    #[test]
    fn blocks_rls_on_sqlite() {
        let req = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: None,
            definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
                schema: "main".into(),
                table: "t".into(),
                name: "p".into(),
                permissive: true,
                command: "ALL".into(),
                roles: vec![],
                using_expr: Some("true".into()),
                with_check_expr: None,
                new_name: None,
            }),
            options: MutationOptions::default(),
            driver: "sqlite".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.unwrap().contains("SQLite"));
    }

    fn column_alter_request(driver: &str) -> ObjectMutationRequest {
        ObjectMutationRequest {
            action: ObjectAction::Alter,
            target: None,
            definition: ObjectDefinition::Column(ColumnDefinition {
                schema: "public".into(),
                table: "users".into(),
                name: "email".into(),
                data_type: "VARCHAR(320)".into(),
                nullable: false,
                default: Some("'x'".into()),
                is_pk: false,
                new_name: None,
            }),
            options: MutationOptions::default(),
            driver: driver.to_owned(),
        }
    }

    #[test]
    fn plans_column_alter_postgres_combined_clauses() {
        let preview = ObjectMutationService::plan(&column_alter_request("postgresql"), &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_none());
        assert!(preview.long_running);
        assert_eq!(preview.safety, "mutating");
        let sql = &preview.statements[0];
        assert!(sql.contains("ALTER TABLE \"public\".\"users\""));
        assert!(sql.contains("ALTER COLUMN \"email\" TYPE VARCHAR(320)"));
        assert!(sql.contains("ALTER COLUMN \"email\" SET NOT NULL"));
        assert!(sql.contains("ALTER COLUMN \"email\" SET DEFAULT 'x'"));
    }

    #[test]
    fn plans_column_alter_mysql_modify_column() {
        let preview = ObjectMutationService::plan(&column_alter_request("mysql"), &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_none());
        assert!(preview.statements[0].contains("MODIFY COLUMN \"email\" VARCHAR(320) NOT NULL DEFAULT 'x'"));
    }

    #[test]
    fn blocks_column_alter_on_sqlite_with_reason() {
        let preview = ObjectMutationService::plan(&column_alter_request("sqlite"), &PgDialect).unwrap();
        let reason = preview.unsupported_reason.unwrap();
        assert!(reason.contains("ALTER COLUMN"));
        assert!(preview.statements.is_empty());
    }

    #[test]
    fn column_alter_sqlserver_rejects_default_change() {
        let err = ObjectMutationService::plan(&column_alter_request("sqlserver"), &PgDialect).unwrap_err();
        assert!(matches!(err, MutationError::Unsupported { .. }));
    }

    #[test]
    fn column_alter_requires_type_and_names() {
        let mut request = column_alter_request("postgresql");
        if let ObjectDefinition::Column(def) = &mut request.definition {
            def.data_type = "  ".into();
        }
        let err = ObjectMutationService::plan(&request, &PgDialect).unwrap_err();
        assert!(matches!(err, MutationError::Validation { .. }));
    }
}
