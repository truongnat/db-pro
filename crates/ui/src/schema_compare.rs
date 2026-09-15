//! Local schema snapshot / compare for Phase F (#199).

use crate::runtime::{UiSchemaSummary, UiTableSummary};

#[derive(Debug, Clone, Default)]
pub(crate) struct UiSchemaSnapshot {
    pub label: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{UiSchemaColumn, UiTableSummary};

    #[test]
    fn detects_added_table_and_column_type_change() {
        let source = UiSchemaSnapshot {
            label: "a".into(),
            tables: vec![UiTableSummary {
                schema: "public".into(),
                name: "t".into(),
                row_count: None,
                columns: vec![UiSchemaColumn {
                    name: "id".into(),
                    data_type: "integer".into(),
                    nullable: false,
                    is_primary_key: true,
                }],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };
        let target = UiSchemaSnapshot {
            label: "b".into(),
            tables: vec![
                UiTableSummary {
                    schema: "public".into(),
                    name: "t".into(),
                    row_count: None,
                    columns: vec![UiSchemaColumn {
                        name: "id".into(),
                        data_type: "bigint".into(),
                        nullable: false,
                        is_primary_key: true,
                    }],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".into(),
                    name: "u".into(),
                    row_count: None,
                    columns: vec![],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let diff = diff_snapshots(&source, &target);
        assert!(diff.tables_only_in_target.iter().any(|t| t == "public.u"));
        assert!(diff.column_changes.iter().any(|c| c.contains("integer → bigint")));
    }
}
