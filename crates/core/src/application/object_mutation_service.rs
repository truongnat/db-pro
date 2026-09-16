//! Object mutation plan/apply service (#183 / Phase A).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::application::ddl_builder::{self, ColumnDef};
use crate::domain::object_mutation::*;
use crate::ports::SqlDialect;

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

        let statements = build_statements(request, dialect)?;
        let safety = classify_safety(request.action);
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

fn build_statements(request: &ObjectMutationRequest, dialect: &dyn SqlDialect) -> Result<Vec<String>, MutationError> {
    match &request.definition {
        ObjectDefinition::Table(_) | ObjectDefinition::Column(_) => build_table_column_statements(request, dialect),
        ObjectDefinition::View(_) | ObjectDefinition::MaterializedView(_) => build_view_statements(request, dialect),
        ObjectDefinition::Index(_) => build_index_statements(request, dialect),
        ObjectDefinition::PrimaryKey(_)
        | ObjectDefinition::UniqueConstraint(_)
        | ObjectDefinition::CheckConstraint(_)
        | ObjectDefinition::ForeignKey(_) => build_constraint_statements(request, dialect),
        ObjectDefinition::Trigger(_) => build_trigger_statements(request, dialect),
        ObjectDefinition::Sequence(_) => build_sequence_statements(request, dialect),
        ObjectDefinition::EnumType(_) => build_enum_statements(request, dialect),
        ObjectDefinition::Schema(_) | ObjectDefinition::Database(_) => build_namespace_statements(request, dialect),
        ObjectDefinition::Extension(_) => build_extension_statements(request, dialect),
        ObjectDefinition::Comment(_) => build_comment_statements(request, dialect),
        ObjectDefinition::Partition(_) => build_partition_statements(request, dialect),
        ObjectDefinition::DomainType(_) | ObjectDefinition::Empty => Err(unsupported(request)),
    }
}

fn unsupported(request: &ObjectMutationRequest) -> MutationError {
    MutationError::Unsupported {
        capability: format!("{:?}", request.action),
        reason: format!("no DDL builder for {:?} + {:?}", request.definition, request.action),
    }
}

fn build_table_column_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Table(def), ObjectAction::Create) => {
            let cols: Vec<ColumnDef> = def
                .columns
                .iter()
                .map(|c| ColumnDef {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                    default: c.default.clone(),
                    is_pk: c.is_pk,
                })
                .collect();
            Ok(vec![ddl_builder::build_create_table(dialect, &def.schema, &def.name, &cols)
                .map_err(MutationError::Build)?])
        }
        (ObjectDefinition::Table(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_table(dialect, &def.schema, &def.name)])
        }
        (ObjectDefinition::Column(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_alter_table_add_column(
            dialect,
            &def.schema,
            &def.table,
            &ColumnDef {
                name: def.name.clone(),
                data_type: def.data_type.clone(),
                nullable: def.nullable,
                default: def.default.clone(),
                is_pk: false,
            },
        )
        .map_err(MutationError::Build)?]),
        (ObjectDefinition::Column(def), ObjectAction::Drop) => Ok(vec![ddl_builder::build_alter_table_drop_column(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
        )]),
        (ObjectDefinition::Column(def), ObjectAction::Rename) => {
            let new_name = def.new_name.as_deref().unwrap_or(&def.name);
            Ok(vec![ddl_builder::build_alter_table_rename_column(
                dialect,
                &def.schema,
                &def.table,
                &def.name,
                new_name,
            )])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_view_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::View(def), ObjectAction::Create)
        | (ObjectDefinition::MaterializedView(def), ObjectAction::Create)
            if !def.materialized =>
        {
            Ok(vec![ddl_builder::build_create_view(dialect, &def.schema, &def.name, &def.select_sql)
                .map_err(MutationError::Build)?])
        }
        (ObjectDefinition::MaterializedView(def), ObjectAction::Create)
        | (ObjectDefinition::View(def), ObjectAction::Create)
            if def.materialized =>
        {
            Ok(vec![ddl_builder::build_create_materialized_view(
                dialect,
                &def.schema,
                &def.name,
                &def.select_sql,
            )
            .map_err(MutationError::Build)?])
        }
        (ObjectDefinition::View(def), ObjectAction::Drop)
        | (ObjectDefinition::MaterializedView(def), ObjectAction::Drop)
            if !def.materialized =>
        {
            Ok(vec![ddl_builder::build_drop_view(dialect, &def.schema, &def.name)])
        }
        (ObjectDefinition::MaterializedView(def), ObjectAction::Drop)
        | (ObjectDefinition::View(def), ObjectAction::Drop)
            if def.materialized =>
        {
            Ok(vec![ddl_builder::build_drop_materialized_view(dialect, &def.schema, &def.name)])
        }
        (ObjectDefinition::MaterializedView(def), ObjectAction::Refresh) => {
            Ok(vec![ddl_builder::build_refresh_materialized_view(dialect, &def.schema, &def.name)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_index_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Index(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_index(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.columns,
            def.unique,
        )]),
        (ObjectDefinition::Index(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_index(dialect, &def.schema, &def.name)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_constraint_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::PrimaryKey(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_add_primary_key(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.columns,
        )]),
        (ObjectDefinition::UniqueConstraint(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_add_unique(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.columns,
        )]),
        (ObjectDefinition::CheckConstraint(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_add_check(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.expression,
        )
        .map_err(MutationError::Build)?]),
        (ObjectDefinition::ForeignKey(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_add_foreign_key(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.columns,
            &def.ref_schema,
            &def.ref_table,
            &def.ref_columns,
            def.on_delete.as_deref(),
            def.on_update.as_deref(),
        )
        .map_err(MutationError::Build)?]),
        (ObjectDefinition::ForeignKey(def), ObjectAction::Drop) => Ok(vec![ddl_builder::build_drop_constraint(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
        )]),
        (ObjectDefinition::PrimaryKey(def), ObjectAction::Drop)
        | (ObjectDefinition::UniqueConstraint(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_constraint(
                dialect,
                &def.schema,
                &def.table,
                &def.name,
            )])
        }
        (ObjectDefinition::CheckConstraint(def), ObjectAction::Drop) => Ok(vec![ddl_builder::build_drop_constraint(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
        )]),
        _ => Err(unsupported(request)),
    }
}

fn build_trigger_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Trigger(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_trigger(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
            &def.timing,
            &def.event,
            &def.body,
        )
        .map_err(MutationError::Build)?]),
        (ObjectDefinition::Trigger(def), ObjectAction::Drop) => Ok(vec![ddl_builder::build_drop_trigger(
            dialect,
            &def.schema,
            &def.table,
            &def.name,
        )]),
        _ => Err(unsupported(request)),
    }
}

fn build_sequence_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Sequence(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_sequence(
            dialect,
            &def.schema,
            &def.name,
            def.start,
            def.increment,
            def.cycle,
        )]),
        (ObjectDefinition::Sequence(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_sequence(dialect, &def.schema, &def.name)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_enum_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::EnumType(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_enum(
            dialect,
            &def.schema,
            &def.name,
            &def.values,
        )
        .map_err(MutationError::Build)?]),
        (ObjectDefinition::EnumType(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_type(dialect, &def.schema, &def.name)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_namespace_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Schema(def), ObjectAction::Create) => {
            Ok(vec![ddl_builder::build_create_schema(dialect, &def.name)])
        }
        (ObjectDefinition::Schema(def), ObjectAction::Drop) => Ok(vec![ddl_builder::build_drop_schema(
            dialect,
            &def.name,
            request.options.cascade,
        )]),
        (ObjectDefinition::Database(def), ObjectAction::Create) => {
            Ok(vec![ddl_builder::build_create_database(dialect, &def.name)])
        }
        (ObjectDefinition::Database(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_database(dialect, &def.name)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_extension_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Extension(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_extension(
            dialect,
            &def.name,
            def.schema.as_deref(),
        )]),
        (ObjectDefinition::Extension(def), ObjectAction::Drop) => {
            Ok(vec![ddl_builder::build_drop_extension(dialect, &def.name, def.cascade)])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_comment_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Comment(def), ObjectAction::Comment | ObjectAction::Create) => {
            let kind = object_kind_sql(def.object.kind);
            Ok(vec![ddl_builder::build_comment_on(
                dialect,
                kind,
                def.object.schema.as_deref(),
                &def.object.name,
                def.object.parent.as_deref(),
                def.comment.as_deref(),
            )
            .map_err(MutationError::Build)?])
        }
        _ => Err(unsupported(request)),
    }
}

fn build_partition_statements(
    request: &ObjectMutationRequest,
    dialect: &dyn SqlDialect,
) -> Result<Vec<String>, MutationError> {
    match (&request.definition, request.action) {
        (ObjectDefinition::Partition(def), ObjectAction::Create) => Ok(vec![ddl_builder::build_create_partition(
            dialect,
            &def.schema,
            &def.parent_table,
            &def.name,
            &def.bound_expression,
        )
        .map_err(MutationError::Build)?]),
        _ => Err(unsupported(request)),
    }
}


fn object_kind_sql(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Table => "TABLE",
        ObjectKind::Column => "COLUMN",
        ObjectKind::View | ObjectKind::MaterializedView => "VIEW",
        ObjectKind::Index => "INDEX",
        ObjectKind::Trigger => "TRIGGER",
        ObjectKind::Sequence => "SEQUENCE",
        ObjectKind::Schema => "SCHEMA",
        ObjectKind::Database => "DATABASE",
        ObjectKind::EnumType | ObjectKind::DomainType | ObjectKind::CompositeType => "TYPE",
        _ => "TABLE",
    }
}

fn classify_safety(action: ObjectAction) -> String {
    match action {
        ObjectAction::Drop => "destructive".into(),
        ObjectAction::Alter | ObjectAction::Rename => "mutating".into(),
        ObjectAction::Create | ObjectAction::Refresh | ObjectAction::Comment => "mutating".into(),
        ObjectAction::Enable | ObjectAction::Disable | ObjectAction::GenerateDdl => "safe".into(),
    }
}

fn describe_effects(request: &ObjectMutationRequest) -> Vec<String> {
    vec![format!(
        "{:?} on {:?}",
        request.action,
        request.target.as_ref().map(|t| t.name.as_str()).unwrap_or("<new>")
    )]
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
}
