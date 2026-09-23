use std::collections::BTreeSet;

use crate::domain::cross_connection::{ColumnTypeMismatch, SchemaDiff, TableColumnDiff};
use crate::domain::schema::IntrospectResult;

type QualifiedName = (String, String);

pub(super) fn qualify_key(schema: &str, name: &str) -> QualifiedName {
    (schema.to_owned(), name.to_owned())
}

pub(super) fn display_qualified_name((schema, name): &QualifiedName) -> String {
    if schema.is_empty() {
        if needs_qualified_name_quoting(name) {
            format!(".{}", quote_qualified_name_part(name))
        } else {
            format!(".{name}")
        }
    } else if needs_qualified_name_quoting(schema) || needs_qualified_name_quoting(name) {
        format!(
            "{}.{}",
            display_qualified_name_part(schema),
            display_qualified_name_part(name)
        )
    } else {
        format!("{schema}.{name}")
    }
}

fn needs_qualified_name_quoting(name: &str) -> bool {
    name.contains('.') || name.contains('"')
}

fn quote_qualified_name_part(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn display_qualified_name_part(name: &str) -> String {
    if needs_qualified_name_quoting(name) {
        quote_qualified_name_part(name)
    } else {
        name.to_owned()
    }
}

pub(super) fn compare_introspect_results(source: &IntrospectResult, target: &IntrospectResult) -> SchemaDiff {
    let source_tables = qualified_table_names(source);
    let target_tables = qualified_table_names(target);
    let (tables_only_in_source, tables_only_in_target) = diff_qualified_names(&source_tables, &target_tables);
    let column_diffs = compare_common_columns(source, target, &source_tables, &target_tables);
    let source_indexes = qualified_index_names(source);
    let target_indexes = qualified_index_names(target);
    let (indexes_only_in_source, indexes_only_in_target) = diff_qualified_names(&source_indexes, &target_indexes);

    SchemaDiff {
        tables_only_in_source,
        tables_only_in_target,
        column_diffs,
        indexes_only_in_source,
        indexes_only_in_target,
    }
}

fn qualified_table_names(result: &IntrospectResult) -> BTreeSet<QualifiedName> {
    result
        .tables
        .iter()
        .map(|table| qualify_key(&table.schema, &table.name))
        .collect()
}

fn qualified_index_names(result: &IntrospectResult) -> BTreeSet<QualifiedName> {
    result
        .indexes
        .iter()
        .map(|index| qualify_key(&index.schema, &index.name))
        .collect()
}

fn diff_qualified_names(
    source: &BTreeSet<QualifiedName>,
    target: &BTreeSet<QualifiedName>,
) -> (Vec<String>, Vec<String>) {
    let only_in_source = source.difference(target).map(display_qualified_name).collect();
    let only_in_target = target.difference(source).map(display_qualified_name).collect();
    (only_in_source, only_in_target)
}

fn compare_common_columns(
    source: &IntrospectResult,
    target: &IntrospectResult,
    source_tables: &BTreeSet<QualifiedName>,
    target_tables: &BTreeSet<QualifiedName>,
) -> Vec<TableColumnDiff> {
    source_tables
        .intersection(target_tables)
        .filter_map(|qualified| compare_table_columns(source, target, qualified))
        .collect()
}

fn compare_table_columns(
    source: &IntrospectResult,
    target: &IntrospectResult,
    qualified: &QualifiedName,
) -> Option<TableColumnDiff> {
    let (schema, table) = (qualified.0.as_str(), qualified.1.as_str());
    let source_columns = column_names(source, schema, table);
    let target_columns = column_names(target, schema, table);
    let columns_only_in_source: Vec<String> = source_columns.difference(&target_columns).cloned().collect();
    let columns_only_in_target: Vec<String> = target_columns.difference(&source_columns).cloned().collect();
    let type_mismatches = compare_column_types(source, target, schema, table, &source_columns, &target_columns);

    if columns_only_in_source.is_empty() && columns_only_in_target.is_empty() && type_mismatches.is_empty() {
        return None;
    }
    Some(TableColumnDiff {
        schema: schema.to_owned(),
        table: table.to_owned(),
        columns_only_in_source,
        columns_only_in_target,
        type_mismatches,
    })
}

fn column_names(result: &IntrospectResult, schema: &str, table: &str) -> BTreeSet<String> {
    result
        .columns
        .iter()
        .filter(|column| column.schema == schema && column.table_name == table)
        .map(|column| column.name.clone())
        .collect()
}

fn compare_column_types(
    source: &IntrospectResult,
    target: &IntrospectResult,
    schema: &str,
    table: &str,
    source_columns: &BTreeSet<String>,
    target_columns: &BTreeSet<String>,
) -> Vec<ColumnTypeMismatch> {
    source_columns
        .intersection(target_columns)
        .filter_map(|column_name| {
            let location = ColumnLocation {
                schema,
                table,
                name: column_name,
            };
            let source_type = find_column_type(source, &location)?;
            let target_type = find_column_type(target, &location)?;
            (source_type != target_type).then_some(ColumnTypeMismatch {
                column: column_name.clone(),
                source_type,
                target_type,
            })
        })
        .collect()
}

struct ColumnLocation<'a> {
    schema: &'a str,
    table: &'a str,
    name: &'a str,
}

fn find_column_type(result: &IntrospectResult, location: &ColumnLocation<'_>) -> Option<String> {
    result
        .columns
        .iter()
        .find(|column| {
            column.schema == location.schema && column.table_name == location.table && column.name == location.name
        })
        .map(|column| column.data_type.clone())
}
