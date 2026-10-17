//! Foreign-key lookup for the data grid. A key does not make the cell read-only.

use super::{UiQueryResult, UiTableDataFilter, UiTableFilterOperator, UiTableInfo};
use crate::UiTableForeignKey;
use db_pro_core::ports::SqlDialect;

pub(crate) fn foreign_key_for_column<'a>(info: &'a UiTableInfo, column: &str) -> Option<&'a UiTableForeignKey> {
    info.foreign_keys
        .iter()
        .find(|fk| fk.from_columns.iter().any(|from| from == column))
}

/// Bound on the destination values fetched for an FK cell dropdown (#497). Without it the
/// lookup pulled the whole referenced table on every cell edit. `foreign_key_lookup_sql`
/// reports the cap through this constant so the truncation note stays in sync.
pub(crate) const FK_LOOKUP_LIMIT: usize = 500;

/// A per-cell value dropdown only makes sense for a 1:1 foreign key (#496): `options_from_result`
/// joins every selected column into one label, so a composite key would offer strings like
/// "7, 42" that cannot be written back into a single column. Composite-FK cells stay plain
/// text edits; `open_referenced_row` still navigates them via `referenced_row_filters`.
pub(crate) fn dropdown_applies(fk: &UiTableForeignKey) -> bool {
    fk.from_columns.len() == 1 && fk.to_columns.len() == 1
}

pub(crate) fn lookup_sql(dialect: &dyn SqlDialect, fk: &UiTableForeignKey) -> String {
    let columns = fk
        .to_columns
        .iter()
        .map(|column| dialect.quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let schema = dialect.quote_identifier(&fk.to_schema);
    let table = dialect.quote_identifier(&fk.to_table);
    // Ordering by the leading referenced column is cheap and gives a deterministic prefix;
    // the LIMIT is what keeps a large lookup table out of the dropdown fetch (#497).
    let order_by = fk
        .to_columns
        .first()
        .map(|column| format!(" ORDER BY {}", dialect.quote_identifier(column)))
        .unwrap_or_default();
    format!("SELECT {columns} FROM {schema}.{table}{order_by} LIMIT {FK_LOOKUP_LIMIT}")
}

pub(crate) fn options_from_result(result: &UiQueryResult) -> Vec<String> {
    result
        .rows
        .iter()
        .filter_map(|row| {
            let label = row
                .iter()
                .map(crate::cell_text)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            (!label.is_empty()).then_some(label)
        })
        .collect()
}

pub(crate) fn referenced_row_filters(
    info: &UiTableInfo,
    fk: &UiTableForeignKey,
    result: &UiQueryResult,
    row: &[super::UiCell],
) -> Option<Vec<UiTableDataFilter>> {
    if fk.from_columns.len() != fk.to_columns.len() || fk.from_columns.is_empty() {
        return None;
    }
    let mut filters = Vec::with_capacity(fk.from_columns.len());
    for (from_column, to_column) in fk.from_columns.iter().zip(&fk.to_columns) {
        let index = result.columns.iter().position(|column| column.name == *from_column)?;
        let cell = row.get(index)?;
        if matches!(cell, super::UiCell::Null) {
            return None;
        }
        let data_type = info
            .columns
            .iter()
            .find(|column| column.name == *from_column)
            .map(|column| column.data_type.clone())
            .unwrap_or_else(|| "text".to_owned());
        filters.push(UiTableDataFilter {
            column: to_column.clone(),
            data_type,
            operator: UiTableFilterOperator::Equals,
            value: crate::cell_text(cell),
        });
    }
    Some(filters)
}
