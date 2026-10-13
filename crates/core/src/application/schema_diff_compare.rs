use std::collections::BTreeSet;

use crate::domain::cross_connection::{ColumnTypeMismatch, SchemaDiff, TableColumnDiff, ViewBodyMismatch};
use crate::domain::schema::{ForeignKey, Index, IntrospectResult};

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
    let index_definition_mismatches = compare_index_definitions(source, target);
    let (foreign_keys_only_in_source, foreign_keys_only_in_target) =
        diff_qualified_names(&foreign_key_labels(source), &foreign_key_labels(target));
    let (views_only_in_source, views_only_in_target, view_body_mismatches) = compare_views(source, target);

    SchemaDiff {
        tables_only_in_source,
        tables_only_in_target,
        column_diffs,
        indexes_only_in_source,
        indexes_only_in_target,
        index_definition_mismatches,
        foreign_keys_only_in_source,
        foreign_keys_only_in_target,
        views_only_in_source,
        views_only_in_target,
        view_body_mismatches,
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

fn compare_index_definitions(source: &IntrospectResult, target: &IntrospectResult) -> Vec<String> {
    let mut mismatches = Vec::new();
    for source_index in &source.indexes {
        let Some(target_index) = target.indexes.iter().find(|index| same_index(source_index, index)) else {
            continue;
        };
        let source_body = index_body(source_index);
        let target_body = index_body(target_index);
        if source_body != target_body {
            mismatches.push(format!(
                "{}: {source_body} -> {target_body}",
                display_qualified_name(&qualify_key(&source_index.schema, &source_index.name))
            ));
        }
    }
    mismatches.sort();
    mismatches
}

fn same_index(left: &Index, right: &Index) -> bool {
    left.schema == right.schema && left.name == right.name
}

fn index_body(index: &Index) -> String {
    format!(
        "unique={} columns={} definition={}",
        index.unique,
        index.columns.join(","),
        index.definition.trim()
    )
}

fn foreign_key_labels(result: &IntrospectResult) -> BTreeSet<QualifiedName> {
    result
        .foreign_keys
        .iter()
        .map(|foreign_key| qualify_key(&foreign_key.schema, &foreign_key_label(foreign_key)))
        .collect()
}

fn foreign_key_label(foreign_key: &ForeignKey) -> String {
    format!(
        "{} ({}) -> {}.{} ({}) ON UPDATE {} ON DELETE {}",
        foreign_key.name,
        foreign_key.from_columns.join(","),
        foreign_key.to_schema,
        foreign_key.to_table,
        foreign_key.to_columns.join(","),
        foreign_key.on_update,
        foreign_key.on_delete,
    )
}

fn compare_views(
    source: &IntrospectResult,
    target: &IntrospectResult,
) -> (Vec<String>, Vec<String>, Vec<ViewBodyMismatch>) {
    let source_views = view_names(source);
    let target_views = view_names(target);
    let (views_only_in_source, views_only_in_target) = diff_qualified_names(&source_views, &target_views);
    let mut view_body_mismatches = Vec::new();
    for qualified in source_views.intersection(&target_views) {
        let (schema, name) = (qualified.0.as_str(), qualified.1.as_str());
        let source_body = view_body(source, schema, name);
        let target_body = view_body(target, schema, name);
        if source_body != target_body {
            view_body_mismatches.push(ViewBodyMismatch {
                schema: schema.to_owned(),
                name: name.to_owned(),
                source_body,
                target_body,
            });
        }
    }
    view_body_mismatches.sort_by(|left, right| (&left.schema, &left.name).cmp(&(&right.schema, &right.name)));
    (views_only_in_source, views_only_in_target, view_body_mismatches)
}

fn view_names(result: &IntrospectResult) -> BTreeSet<QualifiedName> {
    result
        .views
        .iter()
        .map(|view| qualify_key(&view.schema, &view.name))
        .collect()
}

fn view_body(result: &IntrospectResult, schema: &str, name: &str) -> String {
    result
        .views
        .iter()
        .find(|view| view.schema == schema && view.name == name)
        .map(|view| view.definition.trim().to_owned())
        .unwrap_or_default()
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
