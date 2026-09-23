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
}

fn unsupported_reason(request: &ObjectMutationRequest) -> Option<String> {
    let driver = request.driver.to_ascii_lowercase();
    let is_sqlite = driver.contains("sqlite");
    let is_pg = driver.contains("postgres") || driver == "pg";

    match (&request.definition, request.action) {
        (ObjectDefinition::MaterializedView(_), _) if is_sqlite => {
            Some("SQLite does not support materialized views".into())
        }
        (ObjectDefinition::Sequence(_), _) if is_sqlite => Some("SQLite does not support CREATE SEQUENCE".into()),
        (ObjectDefinition::EnumType(_), _) if is_sqlite => {
            Some("SQLite does not support CREATE TYPE ... AS ENUM".into())
        }
        (ObjectDefinition::DomainType(_), _) if is_sqlite => Some("SQLite does not support CREATE DOMAIN".into()),
        (ObjectDefinition::Extension(_), _) if is_sqlite => Some("SQLite does not support CREATE EXTENSION".into()),
        (ObjectDefinition::Partition(_), _) if is_sqlite => Some("SQLite does not support table partitions".into()),
        (ObjectDefinition::Routine(_), _) if is_sqlite => {
            Some("SQLite does not support stored functions/procedures".into())
        }
        (ObjectDefinition::RlsPolicy(_) | ObjectDefinition::TableRls(_), _) if is_sqlite => {
            Some("SQLite does not support row-level security policies".into())
        }
        (ObjectDefinition::RlsPolicy(_) | ObjectDefinition::TableRls(_), _) if !is_pg => {
            Some("row-level security policies require PostgreSQL".into())
        }
        (ObjectDefinition::Schema(_), ObjectAction::Create | ObjectAction::Drop) if is_sqlite => {
            Some("SQLite has no CREATE/DROP SCHEMA".into())
        }
        (ObjectDefinition::Database(_), _) if is_sqlite => {
            Some("SQLite CREATE/DROP DATABASE is not supported in this workbench".into())
        }
        (ObjectDefinition::Comment(_), _) if is_sqlite => Some("SQLite COMMENT ON is not supported".into()),
        (ObjectDefinition::Trigger(_), ObjectAction::Create) if is_pg || is_sqlite => None,
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
            format!("\"{name}\"")
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
                }],
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        let preview = ObjectMutationService::plan(&req, &PgDialect).unwrap();
        assert!(preview.unsupported_reason.is_none());
        assert!(preview.statements[0].contains("CREATE TABLE"));
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
}
