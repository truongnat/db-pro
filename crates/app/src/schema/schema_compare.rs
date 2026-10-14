// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Local schema snapshot / compare for Phase F (#199).

use crate::runtime::{UiSchemaSummary, UiTableSummary};

#[derive(Debug, Clone, Default)]
pub(crate) struct UiSchemaSnapshot {
    pub label: String,
    // allow: field is captured from schema introspection snapshot; not yet rendered in current
    // compare view but preserved in payload so diff maintains schema context.
    #[allow(dead_code)]
    pub schemas: Vec<String>,
    pub tables: Vec<UiTableSummary>,
    pub views: Vec<(String, String)>,
    pub functions: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UiSchemaDiffResult {
    pub tables_only_in_source: Vec<String>,
    pub tables_only_in_target: Vec<String>,
    pub views_only_in_source: Vec<String>,
    pub views_only_in_target: Vec<String>,
    pub functions_only_in_source: Vec<String>,
    pub functions_only_in_target: Vec<String>,
    pub column_changes: Vec<String>,
}

impl UiSchemaSnapshot {
    pub fn from_summary(label: impl Into<String>, summary: &UiSchemaSummary) -> Self {
        Self {
            label: label.into(),
            schemas: summary.schemas.clone(),
            tables: summary.table_details.clone(),
            views: summary
                .views
                .iter()
                .map(|v| (v.schema.clone(), v.name.clone()))
                .collect(),
            functions: summary
                .functions
                .iter()
                .map(|f| (f.schema.clone(), f.name.clone(), f.routine_type.clone()))
                .collect(),
        }
    }
}

pub(crate) fn diff_snapshots(source: &UiSchemaSnapshot, target: &UiSchemaSnapshot) -> UiSchemaDiffResult {
    let source_tables: std::collections::BTreeSet<_> = source
        .tables
        .iter()
        .map(|t| format!("{}.{}", t.schema, t.name))
        .collect();
    let target_tables: std::collections::BTreeSet<_> = target
        .tables
        .iter()
        .map(|t| format!("{}.{}", t.schema, t.name))
        .collect();

    let mut column_changes = Vec::new();
    for table in &source.tables {
        let key = format!("{}.{}", table.schema, table.name);
        if let Some(other) = target
            .tables
            .iter()
            .find(|t| t.schema == table.schema && t.name == table.name)
        {
            let src_cols: std::collections::BTreeSet<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
            let tgt_cols: std::collections::BTreeSet<_> = other.columns.iter().map(|c| c.name.as_str()).collect();
            for col in src_cols.difference(&tgt_cols) {
                column_changes.push(format!("{key}: column `{col}` only in source"));
            }
            for col in tgt_cols.difference(&src_cols) {
                column_changes.push(format!("{key}: column `{col}` only in target"));
            }
            for col in &table.columns {
                if let Some(other_col) = other.columns.iter().find(|c| c.name == col.name) {
                    if col.data_type != other_col.data_type {
                        column_changes.push(format!(
                            "{key}.{}: type {} → {}",
                            col.name, col.data_type, other_col.data_type
                        ));
                    }
                    if col.nullable != other_col.nullable {
                        column_changes.push(format!(
                            "{key}.{}: nullable {} → {}",
                            col.name, col.nullable, other_col.nullable
                        ));
                    }
                }
            }
        }
    }

    let source_views: std::collections::BTreeSet<_> = source.views.iter().map(|(s, n)| format!("{s}.{n}")).collect();
    let target_views: std::collections::BTreeSet<_> = target.views.iter().map(|(s, n)| format!("{s}.{n}")).collect();

    let source_fns: std::collections::BTreeSet<_> = source
        .functions
        .iter()
        .map(|(s, n, k)| format!("{k} {s}.{n}"))
        .collect();
    let target_fns: std::collections::BTreeSet<_> = target
        .functions
        .iter()
        .map(|(s, n, k)| format!("{k} {s}.{n}"))
        .collect();

    UiSchemaDiffResult {
        tables_only_in_source: source_tables.difference(&target_tables).cloned().collect(),
        tables_only_in_target: target_tables.difference(&source_tables).cloned().collect(),
        views_only_in_source: source_views.difference(&target_views).cloned().collect(),
        views_only_in_target: target_views.difference(&source_views).cloned().collect(),
        functions_only_in_source: source_fns.difference(&target_fns).cloned().collect(),
        functions_only_in_target: target_fns.difference(&source_fns).cloned().collect(),
        column_changes,
    }
}

pub(crate) fn snapshot_to_introspect(snapshot: &UiSchemaSnapshot) -> db_pro_core::domain::schema::IntrospectResult {
    use db_pro_core::domain::schema::{Column, IntrospectResult, Table};

    let mut result = IntrospectResult::empty();
    for table in &snapshot.tables {
        result.tables.push(Table {
            name: table.name.clone(),
            schema: table.schema.clone(),
            row_count: table.row_count,
        });
        for column in &table.columns {
            result.columns.push(Column {
                name: column.name.clone(),
                data_type: column.data_type.clone(),
                nullable: column.nullable,
                is_primary_key: column.is_primary_key,
                table_name: table.name.clone(),
                schema: table.schema.clone(),
                ..Default::default()
            });
        }
    }
    result
}

#[cfg(test)]
#[path = "tests/schema_compare_tests.rs"]
mod tests;

