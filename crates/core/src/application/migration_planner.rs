//! Dependency-aware migration planning from schema diffs (#200).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::domain::cross_connection::SchemaDiff;
use crate::domain::migration::{MigrationOpKind, MigrationOperation, MigrationPlan, MigrationRisk};

pub struct MigrationPlanner;

impl MigrationPlanner {
    /// Build an ordered migration plan that moves `target` toward `source`.
    ///
    /// Destructive operations are included and flagged; callers must never auto-apply them.
    pub fn plan_from_schema_diff(diff: &SchemaDiff, driver: &str) -> MigrationPlan {
        let is_sqlite = driver.to_ascii_lowercase().contains("sqlite");
        let mut operations = Vec::new();
        let mut warnings = Vec::new();
        let mut seq = 0u32;

        // 1. Create tables present only in source (desired).
        for qualified in &diff.tables_only_in_source {
            let (schema, table) = split_qualified(qualified);
            seq += 1;
            let id = format!("op-{seq:04}");
            operations.push(MigrationOperation {
                id: id.clone(),
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
            warnings.push(format!(
                "{id}: CREATE TABLE for {qualified} uses a placeholder body; expand columns from source introspection before apply"
            ));
        }

        // 2. Add columns present only in source.
        for col_diff in &diff.column_diffs {
            for column in &col_diff.columns_only_in_source {
                seq += 1;
                let id = format!("op-{seq:04}");
                let table_id = operations
                    .iter()
                    .find(|op| {
                        op.kind == MigrationOpKind::CreateTable
                            && op.schema == col_diff.schema
                            && op.object == col_diff.table
                    })
                    .map(|op| op.id.clone());
                operations.push(MigrationOperation {
                    id,
                    kind: MigrationOpKind::AddColumn,
                    schema: col_diff.schema.clone(),
                    object: format!("{}.{}", col_diff.table, column),
                    sql: format!(
                        "ALTER TABLE {} ADD COLUMN {} TEXT /* type from source */",
                        qualify(&col_diff.schema, &col_diff.table),
                        quote_ident(column)
                    ),
                    risk: MigrationRisk::Mutating,
                    dependencies: table_id.into_iter().collect(),
                    provider_supported: true,
                    unsupported_reason: None,
                });
            }
        }

        // 3. Type mismatches — mutating; SQLite often unsupported without rebuild.
        for col_diff in &diff.column_diffs {
            for mismatch in &col_diff.type_mismatches {
                seq += 1;
                let id = format!("op-{seq:04}");
                let (supported, reason) = if is_sqlite {
                    (
                        false,
                        Some("SQLite cannot ALTER COLUMN type in-place; requires table rebuild strategy".into()),
                    )
                } else {
                    (true, None)
                };
                operations.push(MigrationOperation {
                    id,
                    kind: MigrationOpKind::AlterColumnType,
                    schema: col_diff.schema.clone(),
                    object: format!("{}.{}", col_diff.table, mismatch.column),
                    sql: format!(
                        "ALTER TABLE {} ALTER COLUMN {} TYPE {}",
                        qualify(&col_diff.schema, &col_diff.table),
                        quote_ident(&mismatch.column),
                        mismatch.source_type
                    ),
                    risk: MigrationRisk::Mutating,
                    dependencies: Vec::new(),
                    provider_supported: supported,
                    unsupported_reason: reason,
                });
            }
        }

        // 4. Create indexes present only in source.
        for qualified in &diff.indexes_only_in_source {
            let (schema, name) = split_qualified(qualified);
            seq += 1;
            operations.push(MigrationOperation {
                id: format!("op-{seq:04}"),
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
            warnings.push(format!(
                "op-{seq:04}: CREATE INDEX for {qualified} needs column list from source introspection"
            ));
        }

        // 5. Drop indexes present only in target (before drop column/table).
        for qualified in &diff.indexes_only_in_target {
            let (schema, name) = split_qualified(qualified);
            seq += 1;
            operations.push(MigrationOperation {
                id: format!("op-{seq:04}"),
                kind: MigrationOpKind::DropIndex,
                schema: schema.clone(),
                object: name.clone(),
                sql: if is_sqlite {
                    format!("DROP INDEX IF EXISTS {}", quote_ident(&name))
                } else {
                    format!("DROP INDEX IF EXISTS {}", qualify(&schema, &name))
                },
                risk: MigrationRisk::Destructive,
                dependencies: Vec::new(),
                provider_supported: true,
                unsupported_reason: None,
            });
        }

        // 6. Drop columns present only in target.
        for col_diff in &diff.column_diffs {
            for column in &col_diff.columns_only_in_target {
                seq += 1;
                let (supported, reason) = if is_sqlite {
                    (
                        false,
                        Some("SQLite DROP COLUMN support varies; verify runtime capability".into()),
                    )
                } else {
                    (true, None)
                };
                operations.push(MigrationOperation {
                    id: format!("op-{seq:04}"),
                    kind: MigrationOpKind::DropColumn,
                    schema: col_diff.schema.clone(),
                    object: format!("{}.{}", col_diff.table, column),
                    sql: format!(
                        "ALTER TABLE {} DROP COLUMN {}",
                        qualify(&col_diff.schema, &col_diff.table),
                        quote_ident(column)
                    ),
                    risk: MigrationRisk::Destructive,
                    dependencies: Vec::new(),
                    provider_supported: supported,
                    unsupported_reason: reason,
                });
            }
        }

        // 7. Drop tables present only in target.
        for qualified in &diff.tables_only_in_target {
            let (schema, table) = split_qualified(qualified);
            seq += 1;
            operations.push(MigrationOperation {
                id: format!("op-{seq:04}"),
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

        let has_destructive = operations.iter().any(|op| op.risk == MigrationRisk::Destructive);
        if has_destructive {
            warnings
                .push("Plan contains destructive operations — never auto-apply; require explicit confirmation".into());
        }

        let fingerprint = fingerprint_ops(&operations);
        MigrationPlan {
            driver: driver.to_owned(),
            operations,
            fingerprint,
            warnings,
            has_destructive,
        }
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
    for op in operations {
        op.id.hash(&mut hasher);
        op.sql.hash(&mut hasher);
        op.risk.hash(&mut hasher);
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
