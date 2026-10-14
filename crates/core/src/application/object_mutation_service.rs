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
#[path = "tests/object_mutation_service_tests.rs"]
mod tests;
