//! Dependency-aware migration planning from schema diffs (#200).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::domain::cross_connection::{SchemaDiff, TableColumnDiff};
use crate::domain::migration::{MigrationOpKind, MigrationOperation, MigrationPlan, MigrationRisk};
use crate::domain::schema::{Column, Index, IntrospectResult};
use crate::ports::SqlDialect;

pub struct MigrationPlanner;

impl MigrationPlanner {
    /// Build an ordered migration plan that moves `target` toward `source`.
    ///
    /// Destructive operations are included and flagged; callers must never auto-apply them.
    pub fn plan_from_schema_diff(diff: &SchemaDiff, driver: &str) -> MigrationPlan {
        MigrationPlanBuilder::new(driver).build(diff, None)
    }

    /// Move `target` toward `source` using the source introspection for column and index bodies.
    pub fn plan_from_introspection(
        source: &IntrospectResult,
        target: &IntrospectResult,
        driver: &str,
    ) -> MigrationPlan {
        let diff = crate::application::schema_diff::compare_schemas(source, target);
        MigrationPlanBuilder::new(driver).build(&diff, Some(source))
    }

    /// SQL preview for non-unsupported operations, joined with semicolons.
    pub fn preview_sql(plan: &MigrationPlan, include_unsupported: bool) -> String {
        plan.operations
            .iter()
            .filter(|op| include_unsupported || op.provider_supported)
            .map(|op| {
                if op.provider_supported {
                    format!("{};", op.sql)
                } else {
                    format!(
                        "-- unsupported: {} ({})",
                        op.sql,
                        op.unsupported_reason.as_deref().unwrap_or("n/a")
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Operations safe to apply without destructive confirmation.
    pub fn non_destructive_sql(plan: &MigrationPlan) -> String {
        plan.operations
            .iter()
            .filter(|op| op.provider_supported && op.risk != MigrationRisk::Destructive)
            .map(|op| format!("{};", op.sql))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn verify_fingerprint(plan: &MigrationPlan, expected: &str) -> bool {
        plan.fingerprint == expected
    }
}

struct MigrationPlanBuilder {
    driver: String,
    is_sqlite: bool,
    operations: Vec<MigrationOperation>,
    warnings: Vec<String>,
    next_sequence: u32,
}

struct PendingMigrationOperation {
    kind: MigrationOpKind,
    schema: String,
    object: String,
    sql: String,
    risk: MigrationRisk,
    dependencies: Vec<String>,
    provider_supported: bool,
    unsupported_reason: Option<String>,
}

impl MigrationPlanBuilder {
    fn new(driver: &str) -> Self {
        Self {
            driver: driver.to_owned(),
            is_sqlite: driver.to_ascii_lowercase().contains("sqlite"),
            operations: Vec::new(),
            warnings: Vec::new(),
            next_sequence: 0,
        }
    }

    fn build(mut self, diff: &SchemaDiff, source: Option<&IntrospectResult>) -> MigrationPlan {
        self.add_source_tables(diff, source);
        self.add_source_columns(diff, source);
        self.add_type_mismatches(diff);
        self.add_source_indexes(diff, source);
        self.add_target_indexes_to_drop(diff);
        self.add_target_columns_to_drop(diff);
        self.add_target_tables_to_drop(diff);

        let has_destructive = self
            .operations
            .iter()
            .any(|operation| operation.risk == MigrationRisk::Destructive);
        if has_destructive {
            self.warnings
                .push("Plan contains destructive operations — never auto-apply; require explicit confirmation".into());
        }

        let fingerprint = fingerprint_ops(&self.operations);
        MigrationPlan {
            driver: self.driver,
            operations: self.operations,
            fingerprint,
            warnings: self.warnings,
            has_destructive,
        }
    }

    fn add_source_tables(&mut self, diff: &SchemaDiff, source: Option<&IntrospectResult>) {
        for qualified in &diff.tables_only_in_source {
            let (schema, table) = split_qualified(qualified);
            let columns = source
                .map(|introspect| columns_for(introspect, &schema, &table))
                .unwrap_or_default();
            let (sql, provider_supported, unsupported_reason) = match source {
                Some(introspect) if !columns.is_empty() => {
                    let primary_key = primary_key_columns(introspect, &schema, &table, &columns);
                    (
                        create_table_sql(&self.driver, &schema, &table, &columns, &primary_key),
                        true,
                        None,
                    )
                }
                _ => (
                    String::new(),
                    false,
                    Some("CREATE TABLE needs columns from source introspection before apply".into()),
                ),
            };
            let id = self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::CreateTable,
                schema: schema.clone(),
                object: table.clone(),
                sql,
                risk: MigrationRisk::Mutating,
                dependencies: Vec::new(),
                provider_supported,
                unsupported_reason,
            });
            if !provider_supported {
                self.warnings.push(format!(
                    "{id}: CREATE TABLE for {qualified} needs columns from source introspection before apply"
                ));
            }
        }
    }

    fn add_source_columns(&mut self, diff: &SchemaDiff, source: Option<&IntrospectResult>) {
        for column_diff in &diff.column_diffs {
            for column in &column_diff.columns_only_in_source {
                let table_id = self.created_table_id(column_diff);
                let defined = source
                    .and_then(|introspect| column_for(introspect, &column_diff.schema, &column_diff.table, column));
                let (sql, provider_supported, unsupported_reason) = match defined {
                    Some(defined) if !defined.data_type.is_empty() => (
                        format!(
                            "ALTER TABLE {} ADD COLUMN {}",
                            qualify(&self.driver, &column_diff.schema, &column_diff.table),
                            column_clause(&self.driver, defined)
                        ),
                        true,
                        None,
                    ),
                    _ => (
                        String::new(),
                        false,
                        Some("ADD COLUMN needs the source column type from introspection before apply".into()),
                    ),
                };
                self.add_operation(PendingMigrationOperation {
                    kind: MigrationOpKind::AddColumn,
                    schema: column_diff.schema.clone(),
                    object: format!("{}.{}", column_diff.table, column),
                    sql,
                    risk: MigrationRisk::Mutating,
                    dependencies: table_id.into_iter().collect(),
                    provider_supported,
                    unsupported_reason,
                });
            }
        }
    }

    fn created_table_id(&self, column_diff: &TableColumnDiff) -> Option<String> {
        self.operations
            .iter()
            .find(|operation| {
                operation.kind == MigrationOpKind::CreateTable
                    && operation.schema == column_diff.schema
                    && operation.object == column_diff.table
            })
            .map(|operation| operation.id.clone())
    }

    fn add_type_mismatches(&mut self, diff: &SchemaDiff) {
        for column_diff in &diff.column_diffs {
            for mismatch in &column_diff.type_mismatches {
                let (provider_supported, unsupported_reason) = if self.is_sqlite {
                    (
                        false,
                        Some("SQLite cannot ALTER COLUMN type in-place; requires table rebuild strategy".into()),
                    )
                } else {
                    (true, None)
                };
                self.add_operation(PendingMigrationOperation {
                    kind: MigrationOpKind::AlterColumnType,
                    schema: column_diff.schema.clone(),
                    object: format!("{}.{}", column_diff.table, mismatch.column),
                    sql: format!(
                        "ALTER TABLE {} ALTER COLUMN {} TYPE {}",
                        qualify(&self.driver, &column_diff.schema, &column_diff.table),
                        quote_ident(&self.driver, &mismatch.column),
                        mismatch.source_type
                    ),
                    risk: MigrationRisk::Mutating,
                    dependencies: Vec::new(),
                    provider_supported,
                    unsupported_reason,
                });
            }
        }
    }

    fn add_source_indexes(&mut self, diff: &SchemaDiff, source: Option<&IntrospectResult>) {
        for qualified in &diff.indexes_only_in_source {
            let (schema, name) = split_qualified(qualified);
            let defined = source.and_then(|introspect| index_for(introspect, &schema, &name));
            let (sql, provider_supported, unsupported_reason) =
                match defined.and_then(|index| create_index_sql(&self.driver, index)) {
                    Some(sql) => (sql, true, None),
                    None => (
                        String::new(),
                        false,
                        Some("CREATE INDEX needs columns from source introspection before apply".into()),
                    ),
                };
            let id = self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::CreateIndex,
                schema,
                object: name,
                sql,
                risk: MigrationRisk::Mutating,
                dependencies: Vec::new(),
                provider_supported,
                unsupported_reason,
            });
            if !provider_supported {
                self.warnings.push(format!(
                    "{id}: CREATE INDEX for {qualified} needs columns from source introspection before apply"
                ));
            }
        }
    }

    fn add_target_indexes_to_drop(&mut self, diff: &SchemaDiff) {
        for qualified in &diff.indexes_only_in_target {
            let (schema, name) = split_qualified(qualified);
            let sql = if self.is_sqlite {
                format!("DROP INDEX IF EXISTS {}", quote_ident(&self.driver, &name))
            } else {
                format!("DROP INDEX IF EXISTS {}", qualify(&self.driver, &schema, &name))
            };
            self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::DropIndex,
                schema,
                object: name,
                sql,
                risk: MigrationRisk::Destructive,
                dependencies: Vec::new(),
                provider_supported: true,
                unsupported_reason: None,
            });
        }
    }

    fn add_target_columns_to_drop(&mut self, diff: &SchemaDiff) {
        for column_diff in &diff.column_diffs {
            for column in &column_diff.columns_only_in_target {
                let (provider_supported, unsupported_reason) = if self.is_sqlite {
                    (
                        false,
                        Some("SQLite DROP COLUMN support varies; verify runtime capability".into()),
                    )
                } else {
                    (true, None)
                };
                self.add_operation(PendingMigrationOperation {
                    kind: MigrationOpKind::DropColumn,
                    schema: column_diff.schema.clone(),
                    object: format!("{}.{}", column_diff.table, column),
                    sql: format!(
                        "ALTER TABLE {} DROP COLUMN {}",
                        qualify(&self.driver, &column_diff.schema, &column_diff.table),
                        quote_ident(&self.driver, column)
                    ),
                    risk: MigrationRisk::Destructive,
                    dependencies: Vec::new(),
                    provider_supported,
                    unsupported_reason,
                });
            }
        }
    }

    fn add_target_tables_to_drop(&mut self, diff: &SchemaDiff) {
        for qualified in &diff.tables_only_in_target {
            let (schema, table) = split_qualified(qualified);
            self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::DropTable,
                schema: schema.clone(),
                object: table.clone(),
                sql: format!("DROP TABLE IF EXISTS {}", qualify(&self.driver, &schema, &table)),
                risk: MigrationRisk::Destructive,
                dependencies: Vec::new(),
                provider_supported: true,
                unsupported_reason: None,
            });
        }
    }

    fn add_operation(&mut self, pending: PendingMigrationOperation) -> String {
        self.next_sequence += 1;
        let id = format!("op-{:04}", self.next_sequence);
        self.operations.push(MigrationOperation {
            id: id.clone(),
            kind: pending.kind,
            schema: pending.schema,
            object: pending.object,
            sql: pending.sql,
            risk: pending.risk,
            dependencies: pending.dependencies,
            provider_supported: pending.provider_supported,
            unsupported_reason: pending.unsupported_reason,
        });
        id
    }
}

fn split_qualified(qualified: &str) -> (String, String) {
    if let Some((schema, name)) = qualified.split_once('.') {
        (unquote(schema), unquote(name))
    } else {
        (String::new(), unquote(qualified))
    }
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').replace("\"\"", "\"")
}

fn quote_ident(driver: &str, name: &str) -> String {
    crate::application::driver_dialect::DriverDialect::from_name(driver).quote_identifier(name)
}

fn qualify(driver: &str, schema: &str, name: &str) -> String {
    if schema.is_empty() {
        quote_ident(driver, name)
    } else {
        format!("{}.{}", quote_ident(driver, schema), quote_ident(driver, name))
    }
}

fn columns_for<'a>(source: &'a IntrospectResult, schema: &str, table: &str) -> Vec<&'a Column> {
    source
        .columns
        .iter()
        .filter(|column| column.schema == schema && column.table_name == table)
        .collect()
}

fn column_for<'a>(source: &'a IntrospectResult, schema: &str, table: &str, name: &str) -> Option<&'a Column> {
    source
        .columns
        .iter()
        .find(|column| column.schema == schema && column.table_name == table && column.name == name)
}

fn index_for<'a>(source: &'a IntrospectResult, schema: &str, name: &str) -> Option<&'a Index> {
    source
        .indexes
        .iter()
        .find(|index| index.schema == schema && index.name == name)
}

fn primary_key_columns(source: &IntrospectResult, schema: &str, table: &str, columns: &[&Column]) -> Vec<String> {
    if let Some(primary_key) = source
        .primary_keys
        .iter()
        .find(|primary_key| primary_key.schema == schema && primary_key.table_name == table)
    {
        if !primary_key.columns.is_empty() {
            return primary_key.columns.clone();
        }
    }
    columns
        .iter()
        .filter(|column| column.is_primary_key)
        .map(|column| column.name.clone())
        .collect()
}

fn column_clause(driver: &str, column: &Column) -> String {
    let mut sql = format!("{} {}", quote_ident(driver, &column.name), column.data_type);
    if !column.nullable {
        sql.push_str(" NOT NULL");
    }
    if let Some(default) = column.default.as_deref().filter(|default| !default.is_empty()) {
        sql.push_str(" DEFAULT ");
        sql.push_str(default);
    }
    sql
}

fn create_table_sql(driver: &str, schema: &str, table: &str, columns: &[&Column], primary_key: &[String]) -> String {
    let mut parts: Vec<String> = columns
        .iter()
        .copied()
        .map(|column| column_clause(driver, column))
        .collect();
    if !primary_key.is_empty() {
        let names = primary_key
            .iter()
            .map(|name| quote_ident(driver, name))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("PRIMARY KEY ({names})"));
    }
    format!("CREATE TABLE {} ({})", qualify(driver, schema, table), parts.join(", "))
}

fn create_index_sql(driver: &str, index: &Index) -> Option<String> {
    let definition = index.definition.trim().trim_end_matches(';').trim();
    if !definition.is_empty() {
        return Some(definition.to_owned());
    }
    if index.columns.is_empty() {
        return None;
    }
    let unique = if index.unique { "UNIQUE " } else { "" };
    let columns = index
        .columns
        .iter()
        .map(|name| quote_ident(driver, name))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "CREATE {unique}INDEX {} ON {} ({columns})",
        qualify(driver, &index.schema, &index.name),
        qualify(driver, &index.schema, &index.table_name)
    ))
}

fn fingerprint_ops(operations: &[MigrationOperation]) -> String {
    let mut hasher = DefaultHasher::new();
    for operation in operations {
        operation.id.hash(&mut hasher);
        operation.sql.hash(&mut hasher);
        operation.risk.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cross_connection::{ColumnTypeMismatch, SchemaDiff, TableColumnDiff};

    fn sample_diff() -> SchemaDiff {
        SchemaDiff {
            tables_only_in_source: vec!["public.new_table".into()],
            tables_only_in_target: vec!["public.old_table".into()],
            column_diffs: vec![TableColumnDiff {
                schema: "public".into(),
                table: "orders".into(),
                columns_only_in_source: vec!["tenant_id".into()],
                columns_only_in_target: vec!["legacy_flag".into()],
                type_mismatches: vec![ColumnTypeMismatch {
                    column: "amount".into(),
                    source_type: "numeric".into(),
                    target_type: "integer".into(),
                }],
            }],
            indexes_only_in_source: vec!["public.idx_orders_tenant".into()],
            indexes_only_in_target: vec!["public.idx_orders_legacy".into()],
            index_definition_mismatches: Vec::new(),
            foreign_keys_only_in_source: Vec::new(),
            foreign_keys_only_in_target: Vec::new(),
            views_only_in_source: Vec::new(),
            views_only_in_target: Vec::new(),
            view_body_mismatches: Vec::new(),
        }
    }

    #[test]
    fn plans_dependency_order_create_before_drop() {
        let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
        let kinds: Vec<_> = plan.operations.iter().map(|op| op.kind).collect();
        let create_pos = kinds.iter().position(|k| *k == MigrationOpKind::CreateTable).unwrap();
        let drop_pos = kinds.iter().position(|k| *k == MigrationOpKind::DropTable).unwrap();
        assert!(create_pos < drop_pos);
        assert!(plan.has_destructive);
        assert!(!plan.fingerprint.is_empty());
        assert!(MigrationPlanner::verify_fingerprint(&plan, &plan.fingerprint));
    }

    #[test]
    fn sqlite_marks_type_alter_unsupported() {
        let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "sqlite");
        let alter = plan
            .operations
            .iter()
            .find(|op| op.kind == MigrationOpKind::AlterColumnType)
            .expect("alter");
        assert!(!alter.provider_supported);
    }

    #[test]
    fn non_destructive_sql_excludes_drops() {
        let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
        let sql = MigrationPlanner::non_destructive_sql(&plan);
        assert!(!sql.to_ascii_uppercase().contains("DROP TABLE"));
        assert!(!sql.to_ascii_uppercase().contains("DROP COLUMN"));
    }

    #[test]
    fn migration_sql_uses_introspected_columns() {
        use crate::domain::schema::{Column, Index, IntrospectResult, Table};

        let mut source = IntrospectResult::empty();
        source.tables.push(Table {
            name: "widgets".into(),
            schema: "public".into(),
            row_count: None,
        });
        source.tables.push(Table {
            name: "orders".into(),
            schema: "public".into(),
            row_count: None,
        });
        source.columns.push(Column {
            name: "id".into(),
            data_type: "integer".into(),
            nullable: false,
            is_primary_key: true,
            table_name: "widgets".into(),
            schema: "public".into(),
            ..Default::default()
        });
        source.columns.push(Column {
            name: "label".into(),
            data_type: "text".into(),
            nullable: true,
            default: Some("'new'".into()),
            table_name: "widgets".into(),
            schema: "public".into(),
            ..Default::default()
        });
        source.columns.push(Column {
            name: "id".into(),
            data_type: "integer".into(),
            nullable: false,
            is_primary_key: true,
            table_name: "orders".into(),
            schema: "public".into(),
            ..Default::default()
        });
        source.columns.push(Column {
            name: "note".into(),
            data_type: "text".into(),
            nullable: false,
            table_name: "orders".into(),
            schema: "public".into(),
            ..Default::default()
        });
        source.indexes.push(Index {
            name: "widgets_label_idx".into(),
            columns: vec!["label".into()],
            table_name: "widgets".into(),
            schema: "public".into(),
            ..Default::default()
        });

        let mut target = IntrospectResult::empty();
        target.tables.push(Table {
            name: "orders".into(),
            schema: "public".into(),
            row_count: None,
        });
        target.columns.push(Column {
            name: "id".into(),
            data_type: "integer".into(),
            nullable: false,
            is_primary_key: true,
            table_name: "orders".into(),
            schema: "public".into(),
            ..Default::default()
        });

        let plan = MigrationPlanner::plan_from_introspection(&source, &target, "postgresql");
        let sql = MigrationPlanner::non_destructive_sql(&plan);
        assert!(sql.contains("CREATE TABLE \"public\".\"widgets\""));
        assert!(sql.contains("\"id\" integer NOT NULL"));
        assert!(sql.contains("\"label\" text DEFAULT 'new'"));
        assert!(sql.contains("PRIMARY KEY (\"id\")"));
        assert!(sql.contains("ALTER TABLE \"public\".\"orders\" ADD COLUMN \"note\" text NOT NULL"));
        assert!(sql.contains("CREATE INDEX \"public\".\"widgets_label_idx\" ON \"public\".\"widgets\" (\"label\")"));
        assert!(!crate::application::placeholder_remains(&sql));
    }

    #[test]
    fn migration_quotes_identifiers_for_the_driver() {
        use crate::domain::schema::{Column, IntrospectResult, Table};

        let mut source = IntrospectResult::empty();
        source.tables.push(Table {
            name: "widgets".into(),
            schema: "app".into(),
            row_count: None,
        });
        source.columns.push(Column {
            name: "id".into(),
            data_type: "int".into(),
            nullable: false,
            is_primary_key: true,
            table_name: "widgets".into(),
            schema: "app".into(),
            ..Default::default()
        });
        let mysql = MigrationPlanner::non_destructive_sql(&MigrationPlanner::plan_from_introspection(
            &source,
            &IntrospectResult::empty(),
            "mysql",
        ));
        assert!(mysql.contains("CREATE TABLE `app`.`widgets`"), "{mysql}");
        assert!(mysql.contains("`id` int"), "{mysql}");
        let sqlserver = MigrationPlanner::non_destructive_sql(&MigrationPlanner::plan_from_introspection(
            &source,
            &IntrospectResult::empty(),
            "sqlserver",
        ));
        assert!(sqlserver.contains("CREATE TABLE [app].[widgets]"), "{sqlserver}");
        assert!(sqlserver.contains("[id] int"), "{sqlserver}");
    }

    #[test]
    fn missing_introspection_is_not_applicable_sql() {
        let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
        let create = plan
            .operations
            .iter()
            .find(|op| op.kind == MigrationOpKind::CreateTable)
            .expect("create");
        assert!(!create.provider_supported);
        let sql = MigrationPlanner::non_destructive_sql(&plan);
        assert!(!crate::application::placeholder_remains(&sql));
        assert!(!sql.contains("CREATE TABLE"));
    }
}
