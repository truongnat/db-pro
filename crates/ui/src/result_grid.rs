use crate::{UiCell, UiQueryResult};
use std::collections::HashMap;

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
    let row_positions: HashMap<usize, usize> = visible_rows
        .iter()
        .enumerate()
        .map(|(position, row)| (*row, position))
        .collect();
    let row_position = row_positions.get(&selected_row).copied().unwrap_or(0);
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

use bigdecimal::BigDecimal;
use std::cmp::Ordering;

/// Compare two cells using database-appropriate typed semantics.
/// NULL values are placed at the end when sorting ascending.
pub fn compare_ui_cells(left: Option<&UiCell>, right: Option<&UiCell>) -> Ordering {
    match (left, right) {
        (None | Some(UiCell::Null), None | Some(UiCell::Null)) => Ordering::Equal,
        (None | Some(UiCell::Null), Some(_)) => Ordering::Greater,
        (Some(_), None | Some(UiCell::Null)) => Ordering::Less,
        (Some(UiCell::Boolean(l)), Some(UiCell::Boolean(r))) => l.cmp(r),
        (Some(UiCell::Number(l)), Some(UiCell::Number(r))) => {
            if let (Ok(l_dec), Ok(r_dec)) = (l.parse::<BigDecimal>(), r.parse::<BigDecimal>()) {
                l_dec.cmp(&r_dec)
            } else {
                l.cmp(r)
            }
        }
        (Some(UiCell::Text(l)), Some(UiCell::Text(r))) => {
            // Attempt temporal parse if both look like ISO timestamps / dates
            if let (Ok(l_dt), Ok(r_dt)) = (
                chrono::DateTime::parse_from_rfc3339(l),
                chrono::DateTime::parse_from_rfc3339(r),
            ) {
                l_dt.cmp(&r_dt)
            } else if let (Ok(l_dt), Ok(r_dt)) = (
                chrono::NaiveDateTime::parse_from_str(l, "%Y-%m-%d %H:%M:%S"),
                chrono::NaiveDateTime::parse_from_str(r, "%Y-%m-%d %H:%M:%S"),
            ) {
                l_dt.cmp(&r_dt)
            } else if let (Ok(l_d), Ok(r_d)) = (
                chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"),
                chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
            ) {
                l_d.cmp(&r_d)
            } else {
                l.cmp(r)
            }
        }
        (Some(l), Some(r)) => cell_text(l).cmp(&cell_text(r)),
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
            let left_cell = result.rows[*left].get(column);
            let right_cell = result.rows[*right].get(column);
            let ordering = compare_ui_cells(left_cell, right_cell);
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
