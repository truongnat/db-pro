use crate::{UiCell, UiQueryResult};

pub fn displayed_row_number(offset: u64, row_index: usize) -> u64 {
    offset.saturating_add(row_index as u64).saturating_add(1)
}

/// Build the stable row-index projection consumed by the virtualized result grid.
///
/// The result payload stays immutable; filtering and sorting only rearrange
/// indexes so the renderer can materialize the visible window on demand.
pub fn filtered_sorted_indexes(
    result: &UiQueryResult,
    filter: &str,
    sort_column: Option<usize>,
    sort_desc: bool,
) -> Vec<usize> {
    let filter = filter.to_lowercase();
    let mut indexes: Vec<usize> = result
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| filter.is_empty() || row.iter().any(|cell| cell_text(cell).to_lowercase().contains(&filter)))
        .map(|(index, _)| index)
        .collect();

    if let Some(column) = sort_column {
        indexes.sort_by(|left, right| {
            let left_value = result.rows[*left].get(column).map(cell_text).unwrap_or_default();
            let right_value = result.rows[*right].get(column).map(cell_text).unwrap_or_default();
            let ordering = left_value.cmp(&right_value);
            if sort_desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    indexes
}

pub fn cell_text(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => "NULL".to_owned(),
        UiCell::Boolean(value) => value.to_string(),
        UiCell::Number(value) | UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => value.clone(),
    }
}
