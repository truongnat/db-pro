use crate::{UiCell, UiQueryResult};

pub fn displayed_row_number(offset: u64, row_index: usize) -> u64 {
    offset.saturating_add(row_index as u64).saturating_add(1)
}

/// Resolve a keyboard navigation event against the filtered/sorted row projection.
///
/// The returned row is always an original result index, so filtering and sorting do not
/// change the identity used by editing and copy actions.
pub fn grid_keyboard_selection(
    selected: Option<(usize, usize)>,
    visible_rows: &[usize],
    column_count: usize,
    key: eframe::egui::Key,
) -> Option<(usize, usize)> {
    if visible_rows.is_empty() || column_count == 0 {
        return None;
    }

    let Some((selected_row, selected_column)) = selected else {
        return Some((visible_rows[0], 0));
    };
    let row_position = visible_rows.iter().position(|row| *row == selected_row).unwrap_or(0);
    let column = selected_column.min(column_count - 1);

    match key {
        eframe::egui::Key::ArrowUp => visible_rows
            .get(row_position.saturating_sub(1))
            .copied()
            .map(|row| (row, column)),
        eframe::egui::Key::ArrowDown => visible_rows
            .get((row_position + 1).min(visible_rows.len() - 1))
            .copied()
            .map(|row| (row, column)),
        eframe::egui::Key::ArrowLeft => Some((visible_rows[row_position], column.saturating_sub(1))),
        eframe::egui::Key::ArrowRight => Some((visible_rows[row_position], (column + 1).min(column_count - 1))),
        eframe::egui::Key::Home => Some((visible_rows[row_position], 0)),
        eframe::egui::Key::End => Some((visible_rows[row_position], column_count - 1)),
        _ => None,
    }
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
