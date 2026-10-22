use crate::domain::error::DbError;
use crate::domain::schema::{DependencyDirection, DependencyKind, IntrospectResult, TableDependency, TableInfo};

pub(super) fn build_table_info(introspect: IntrospectResult, schema: &str, table: &str) -> Result<TableInfo, DbError> {
    let parts = select_table_parts(&introspect, schema, table)?;
    let mut dependencies = Vec::new();
    add_outgoing_foreign_key_dependencies(&mut dependencies, &parts.foreign_keys);
    add_incoming_foreign_key_dependencies(&mut dependencies, &introspect, schema, table);
    add_view_dependencies(&mut dependencies, &introspect, schema, table);
    add_trigger_dependencies(&mut dependencies, &introspect, schema, table);
    add_function_dependencies(&mut dependencies, &introspect, schema, table);
    add_sequence_dependencies(&mut dependencies, &parts.columns, schema);

    Ok(parts.into_table_info(deduplicate_dependencies(dependencies)))
}

struct TableInfoParts {
    table: crate::domain::schema::Table,
    columns: Vec<crate::domain::schema::Column>,
    primary_key: Option<crate::domain::schema::PrimaryKey>,
    indexes: Vec<crate::domain::schema::Index>,
    foreign_keys: Vec<crate::domain::schema::ForeignKey>,
    check_constraints: Vec<crate::domain::schema::CheckConstraint>,
}

impl TableInfoParts {
    fn into_table_info(self, dependencies: Vec<TableDependency>) -> TableInfo {
        TableInfo {
            table: self.table,
            columns: self.columns,
            primary_key: self.primary_key,
            indexes: self.indexes,
            foreign_keys: self.foreign_keys,
            check_constraints: self.check_constraints,
            dependencies,
        }
    }
}

fn select_table_parts(introspect: &IntrospectResult, schema: &str, table: &str) -> Result<TableInfoParts, DbError> {
    let table_definition = introspect
        .tables
        .iter()
        .find(|candidate| candidate.schema == schema && candidate.name == table)
        .ok_or_else(|| DbError::NotFound(format!("table {schema}.{table}")))?
        .clone();
    let columns = introspect
        .columns
        .iter()
        .filter(|column| column.schema == schema && column.table_name == table)
        .cloned()
        .collect();
    let primary_key = introspect
        .primary_keys
        .iter()
        .find(|key| key.schema == schema && key.table_name == table)
        .cloned();
    let indexes = introspect
        .indexes
        .iter()
        .filter(|index| index.schema == schema && index.table_name == table)
        .cloned()
        .collect();
    let foreign_keys = introspect
        .foreign_keys
        .iter()
        .filter(|foreign_key| foreign_key.from_table == table && foreign_key.schema == schema)
        .cloned()
        .collect();
    let check_constraints = introspect
        .check_constraints
        .iter()
        .filter(|constraint| constraint.schema == schema && constraint.table_name == table)
        .cloned()
        .collect();

    Ok(TableInfoParts {
        table: table_definition,
        columns,
        primary_key,
        indexes,
        foreign_keys,
        check_constraints,
    })
}

fn add_outgoing_foreign_key_dependencies(
    dependencies: &mut Vec<TableDependency>,
    foreign_keys: &[crate::domain::schema::ForeignKey],
) {
    for foreign_key in foreign_keys {
        dependencies.push(TableDependency {
            name: foreign_key.to_table.clone(),
            schema: foreign_key.to_schema.clone(),
            kind: DependencyKind::Table,
            direction: DependencyDirection::DependsOn,
            details: format!(
                "Foreign key {} ({}) → {}.{}({})",
                foreign_key.name,
                foreign_key.from_columns.join(", "),
                foreign_key.to_schema,
                foreign_key.to_table,
                foreign_key.to_columns.join(", ")
            ),
        });
    }
}

fn add_incoming_foreign_key_dependencies(
    dependencies: &mut Vec<TableDependency>,
    introspect: &IntrospectResult,
    schema: &str,
    table: &str,
) {
    for foreign_key in &introspect.foreign_keys {
        if foreign_key.to_schema == schema
            && foreign_key.to_table == table
            && !(foreign_key.schema == schema && foreign_key.from_table == table)
        {
            dependencies.push(TableDependency {
                name: foreign_key.from_table.clone(),
                schema: foreign_key.schema.clone(),
                kind: DependencyKind::Table,
                direction: DependencyDirection::DependedBy,
                details: format!(
                    "Referenced by foreign key {} ({}.{}) → ({})",
                    foreign_key.name,
                    foreign_key.schema,
                    foreign_key.from_table,
                    foreign_key.to_columns.join(", ")
                ),
            });
        }
    }
}

fn add_view_dependencies(
    dependencies: &mut Vec<TableDependency>,
    introspect: &IntrospectResult,
    schema: &str,
    table: &str,
) {
    let table_lower = table.to_lowercase();
    let qualified_lower = format!("{}.{}", schema.to_lowercase(), table_lower);
    for view in &introspect.views {
        let definition_lower = view.definition.to_lowercase();
        if definition_lower.contains(&qualified_lower) || definition_lower.contains(&table_lower) {
            dependencies.push(TableDependency {
                name: view.name.clone(),
                schema: view.schema.clone(),
                kind: DependencyKind::View,
                direction: DependencyDirection::DependedBy,
                details: format!(
                    "View {}.{} references table in query definition",
                    view.schema, view.name
                ),
            });
        }
    }
}

fn add_trigger_dependencies(
    dependencies: &mut Vec<TableDependency>,
    introspect: &IntrospectResult,
    schema: &str,
    table: &str,
) {
    for trigger in &introspect.triggers {
        if trigger.schema == schema && trigger.table_name == table {
            dependencies.push(TableDependency {
                name: trigger.name.clone(),
                schema: trigger.schema.clone(),
                kind: DependencyKind::Trigger,
                direction: DependencyDirection::DependedBy,
                details: format!(
                    "Trigger {} ({} {}) attached to table",
                    trigger.name, trigger.timing, trigger.event
                ),
            });
        }
    }
}

fn add_function_dependencies(
    dependencies: &mut Vec<TableDependency>,
    introspect: &IntrospectResult,
    schema: &str,
    table: &str,
) {
    let table_lower = table.to_lowercase();
    let qualified_lower = format!("{schema}.{table}").to_lowercase();
    for function in &introspect.functions {
        let definition_lower = function.definition.to_lowercase();
        if !function.definition.is_empty()
            && (definition_lower.contains(&qualified_lower) || definition_lower.contains(&table_lower))
        {
            dependencies.push(TableDependency {
                name: function.name.clone(),
                schema: function.schema.clone(),
                kind: DependencyKind::Function,
                direction: DependencyDirection::DependedBy,
                details: format!(
                    "Routine {}.{} ({}) references this table",
                    function.schema, function.name, function.routine_type
                ),
            });
        }
    }
}

fn add_sequence_dependencies(
    dependencies: &mut Vec<TableDependency>,
    columns: &[crate::domain::schema::Column],
    schema: &str,
) {
    for column in columns {
        if let Some(definition) = column.default.as_deref() {
            let definition_lower = definition.to_lowercase();
            if definition_lower.contains("nextval") || definition_lower.contains("_seq") {
                dependencies.push(TableDependency {
                    name: column.name.clone(),
                    schema: schema.to_owned(),
                    kind: DependencyKind::Sequence,
                    direction: DependencyDirection::DependsOn,
                    details: format!("Column {} default sequence expression: {}", column.name, definition),
                });
            }
        }
    }
}

fn deduplicate_dependencies(dependencies: Vec<TableDependency>) -> Vec<TableDependency> {
    let mut seen = std::collections::HashSet::new();
    dependencies
        .into_iter()
        .filter(|dependency| {
            seen.insert((
                dependency.kind,
                dependency.direction,
                dependency.schema.clone(),
                dependency.name.clone(),
            ))
        })
        .collect()
}
