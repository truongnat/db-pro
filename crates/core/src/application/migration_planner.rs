//! Dependency-aware migration planning from schema diffs (#200).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::domain::cross_connection::{SchemaDiff, TableColumnDiff};
use crate::domain::migration::{MigrationOpKind, MigrationOperation, MigrationPlan, MigrationRisk};

pub struct MigrationPlanner;

impl MigrationPlanner {
    /// Build an ordered migration plan that moves `target` toward `source`.
    ///
    /// Destructive operations are included and flagged; callers must never auto-apply them.
    pub fn plan_from_schema_diff(diff: &SchemaDiff, driver: &str) -> MigrationPlan {
        MigrationPlanBuilder::new(driver).build(diff)
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

    fn build(mut self, diff: &SchemaDiff) -> MigrationPlan {
        self.add_source_tables(diff);
        self.add_source_columns(diff);
        self.add_type_mismatches(diff);
        self.add_source_indexes(diff);
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

    fn add_source_tables(&mut self, diff: &SchemaDiff) {
        for qualified in &diff.tables_only_in_source {
            let (schema, table) = split_qualified(qualified);
            let id = self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::CreateTable,
                schema: schema.clone(),
                object: table.clone(),
                sql: format!(
                    "CREATE TABLE {} (id INTEGER PRIMARY KEY /* placeholder — expand from introspection */)",
                    qualify(&schema, &table)
                ),
                risk: MigrationRisk::Mutating,
                dependencies: Vec::new(),
                provider_supported: true,
                unsupported_reason: None,
            });
            self.warnings.push(format!(
                "{id}: CREATE TABLE for {qualified} uses a placeholder body; expand columns from source introspection before apply"
            ));
        }
    }

    fn add_source_columns(&mut self, diff: &SchemaDiff) {
        for column_diff in &diff.column_diffs {
            for column in &column_diff.columns_only_in_source {
                let table_id = self.created_table_id(column_diff);
                self.add_operation(PendingMigrationOperation {
                    kind: MigrationOpKind::AddColumn,
                    schema: column_diff.schema.clone(),
                    object: format!("{}.{}", column_diff.table, column),
                    sql: format!(
                        "ALTER TABLE {} ADD COLUMN {} TEXT /* type from source */",
                        qualify(&column_diff.schema, &column_diff.table),
                        quote_ident(column)
                    ),
                    risk: MigrationRisk::Mutating,
                    dependencies: table_id.into_iter().collect(),
                    provider_supported: true,
                    unsupported_reason: None,
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
                        qualify(&column_diff.schema, &column_diff.table),
                        quote_ident(&mismatch.column),
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

    fn add_source_indexes(&mut self, diff: &SchemaDiff) {
        for qualified in &diff.indexes_only_in_source {
            let (schema, name) = split_qualified(qualified);
            let id = self.add_operation(PendingMigrationOperation {
                kind: MigrationOpKind::CreateIndex,
                schema: schema.clone(),
                object: name.clone(),
                sql: format!(
                    "CREATE INDEX {} ON {} (/* columns from source */)",
                    qualify(&schema, &name),
                    qualify(&schema, &name)
                ),
                risk: MigrationRisk::Mutating,
                dependencies: Vec::new(),
                provider_supported: true,
                unsupported_reason: None,
            });
            self.warnings.push(format!(
                "{id}: CREATE INDEX for {qualified} needs column list from source introspection"
            ));
        }
    }

    fn add_target_indexes_to_drop(&mut self, diff: &SchemaDiff) {
        for qualified in &diff.indexes_only_in_target {
            let (schema, name) = split_qualified(qualified);
            let sql = if self.is_sqlite {
                format!("DROP INDEX IF EXISTS {}", quote_ident(&name))
            } else {
                format!("DROP INDEX IF EXISTS {}", qualify(&schema, &name))
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
                        qualify(&column_diff.schema, &column_diff.table),
                        quote_ident(column)
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
                sql: format!("DROP TABLE IF EXISTS {}", qualify(&schema, &table)),
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

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn qualify(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        quote_ident(name)
    } else {
        format!("{}.{}", quote_ident(schema), quote_ident(name))
    }
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
}
