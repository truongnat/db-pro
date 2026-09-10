use std::collections::HashSet;

use crate::domain::connection::ConnectionId;
use crate::domain::cross_connection::{ColumnTypeMismatch, SchemaDiff, TableColumnDiff};
use crate::domain::error::DbError;
use crate::domain::schema::IntrospectResult;

use super::SchemaService;

impl SchemaService {
    pub async fn diff_schemas(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
    ) -> Result<SchemaDiff, DbError> {
        let source = self.introspect(source_id, false).await?;
        let target = self.introspect(target_id, false).await?;
        Ok(compare_introspect_results(&source, &target))
    }
}

fn qualify_key(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        // Keep the separator so split_qualified can round-trip dotted names.
        format!(".{name}")
    } else {
        format!("{schema}.{name}")
    }
}

fn compare_introspect_results(source: &IntrospectResult, target: &IntrospectResult) -> SchemaDiff {
    let source_tables: HashSet<String> = source.tables.iter().map(|t| qualify_key(&t.schema, &t.name)).collect();
    let target_tables: HashSet<String> = target.tables.iter().map(|t| qualify_key(&t.schema, &t.name)).collect();

    let tables_only_in_source: Vec<String> = source_tables.difference(&target_tables).cloned().collect();
    let tables_only_in_target: Vec<String> = target_tables.difference(&source_tables).cloned().collect();

    let common_tables: Vec<&String> = source_tables.intersection(&target_tables).collect();

    let mut column_diffs = Vec::new();
    for qualified in common_tables {
        let (schema, table) = split_qualified(qualified);

        let source_cols: HashSet<String> = source
            .columns
            .iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .map(|c| c.name.clone())
            .collect();
        let target_cols: HashSet<String> = target
            .columns
            .iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .map(|c| c.name.clone())
            .collect();

        let cols_only_source: Vec<String> = source_cols.difference(&target_cols).cloned().collect();
        let cols_only_target: Vec<String> = target_cols.difference(&source_cols).cloned().collect();

        let common_cols: Vec<&String> = source_cols.intersection(&target_cols).collect();
        let mut type_mismatches = Vec::new();
        for col_name in common_cols {
            let source_type = source
                .columns
                .iter()
                .find(|c| c.schema == schema && c.table_name == table && &c.name == col_name)
                .map(|c| c.data_type.clone());
            let target_type = target
                .columns
                .iter()
                .find(|c| c.schema == schema && c.table_name == table && &c.name == col_name)
                .map(|c| c.data_type.clone());

            if let (Some(st), Some(tt)) = (source_type, target_type) {
                if st != tt {
                    type_mismatches.push(ColumnTypeMismatch {
                        column: col_name.clone(),
                        source_type: st,
                        target_type: tt,
                    });
                }
            }
        }

        if !cols_only_source.is_empty() || !cols_only_target.is_empty() || !type_mismatches.is_empty() {
            column_diffs.push(TableColumnDiff {
                schema: schema.to_string(),
                table: table.to_string(),
                columns_only_in_source: cols_only_source,
                columns_only_in_target: cols_only_target,
                type_mismatches,
            });
        }
    }

    let source_indexes: HashSet<String> = source.indexes.iter().map(|i| qualify_key(&i.schema, &i.name)).collect();
    let target_indexes: HashSet<String> = target.indexes.iter().map(|i| qualify_key(&i.schema, &i.name)).collect();

    let indexes_only_in_source: Vec<String> = source_indexes.difference(&target_indexes).cloned().collect();
    let indexes_only_in_target: Vec<String> = target_indexes.difference(&source_indexes).cloned().collect();

    SchemaDiff {
        tables_only_in_source,
        tables_only_in_target,
        column_diffs,
        indexes_only_in_source,
        indexes_only_in_target,
    }
}

fn split_qualified(qualified: &str) -> (&str, &str) {
    match qualified.split_once('.') {
        Some((schema, table)) => (schema, table),
        None => ("", qualified),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::{Column, Table};

    #[test]
    fn empty_schema_dotted_table_name_round_trips() {
        let qualified = qualify_key("", "my.table");
        assert_eq!(split_qualified(&qualified), ("", "my.table"));
    }

    #[test]
    fn empty_schema_dotted_table_is_compared_by_its_literal_name() {
        let mut source = IntrospectResult::empty();
        source.tables.push(Table {
            name: "my.table".into(),
            schema: "".into(),
            row_count: None,
        });
        source.columns.push(Column {
            name: "id".into(),
            data_type: "INTEGER".into(),
            nullable: false,
            default: None,
            is_primary_key: true,
            table_name: "my.table".into(),
            schema: "".into(),
        });

        let mut target = source.clone();
        target.columns[0].data_type = "TEXT".into();

        let diff = compare_introspect_results(&source, &target);
        assert_eq!(diff.column_diffs.len(), 1);
        assert_eq!(diff.column_diffs[0].schema, "");
        assert_eq!(diff.column_diffs[0].table, "my.table");
        assert_eq!(diff.column_diffs[0].type_mismatches[0].column, "id");
    }
}
