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

/// Borrow the string representation of a cell without heap allocation.
pub fn cell_text_as_str(cell: &UiCell) -> &str {
    match cell {
        UiCell::Null => "NULL",
        UiCell::Boolean(true) => "true",
        UiCell::Boolean(false) => "false",
        UiCell::Number(value) | UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => value.as_str(),
    }
}

pub fn cell_text(cell: &UiCell) -> String {
    cell_text_as_str(cell).to_owned()
}

fn looks_like_iso_temporal(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 10 && bytes[0].is_ascii_digit() && bytes[4] == b'-'
}

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
            if looks_like_iso_temporal(l) && looks_like_iso_temporal(r) {
                // Attempt temporal parse if both look like ISO timestamps / dates
                if let (Ok(l_dt), Ok(r_dt)) = (
                    chrono::DateTime::parse_from_rfc3339(l),
                    chrono::DateTime::parse_from_rfc3339(r),
                ) {
                    return l_dt.cmp(&r_dt);
                } else if let (Ok(l_dt), Ok(r_dt)) = (
                    chrono::NaiveDateTime::parse_from_str(l, "%Y-%m-%d %H:%M:%S"),
                    chrono::NaiveDateTime::parse_from_str(r, "%Y-%m-%d %H:%M:%S"),
                ) {
                    return l_dt.cmp(&r_dt);
                } else if let (Ok(l_d), Ok(r_d)) = (
                    chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"),
                    chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
                ) {
                    return l_d.cmp(&r_d);
                }
            }
            l.cmp(r)
        }
        (Some(l), Some(r)) => cell_text_as_str(l).cmp(cell_text_as_str(r)),
    }
}

fn cell_contains_filter(cell: &UiCell, filter_lower: &str) -> bool {
    let s = cell_text_as_str(cell);
    if filter_lower.is_ascii() && s.is_ascii() {
        let needle = filter_lower.as_bytes();
        if needle.is_empty() {
            return true;
        }
        s.as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle))
    } else {
        s.to_lowercase().contains(filter_lower)
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
    let filter_lower = filter.to_lowercase();
    let mut indexes: Vec<usize> = result
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| filter_lower.is_empty() || row.iter().any(|cell| cell_contains_filter(cell, &filter_lower)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_text_as_str_returns_borrowed_slices() {
        assert_eq!(cell_text_as_str(&UiCell::Null), "NULL");
        assert_eq!(cell_text_as_str(&UiCell::Boolean(true)), "true");
        assert_eq!(cell_text_as_str(&UiCell::Boolean(false)), "false");
        assert_eq!(cell_text_as_str(&UiCell::Number("42".to_owned())), "42");
        assert_eq!(cell_text_as_str(&UiCell::Text("hello".to_owned())), "hello");
        assert_eq!(cell_text_as_str(&UiCell::Json("{}".to_owned())), "{}");
        assert_eq!(cell_text_as_str(&UiCell::Bytes("\\x00".to_owned())), "\\x00");
    }

    #[test]
    fn iso_temporal_heuristic_filters_non_date_strings() {
        assert!(looks_like_iso_temporal("2026-03-15"));
        assert!(looks_like_iso_temporal("2026-03-15 10:20:30"));
        assert!(looks_like_iso_temporal("2026-03-15T10:20:30Z"));
        assert!(!looks_like_iso_temporal("customer-100"));
        assert!(!looks_like_iso_temporal("Alice"));
        assert!(!looks_like_iso_temporal("2026"));
        assert!(!looks_like_iso_temporal("abc-def-ghi"));
    }

    #[test]
    fn cell_contains_filter_handles_ascii_and_unicode() {
        let text_ascii = UiCell::Text("Customer Alpha".to_owned());
        assert!(cell_contains_filter(&text_ascii, "alpha"));
        assert!(cell_contains_filter(&text_ascii, "ALPHA"));
        assert!(!cell_contains_filter(&text_ascii, "beta"));

        let text_unicode = UiCell::Text("người_dùng".to_owned());
        assert!(cell_contains_filter(&text_unicode, "người"));
        assert!(!cell_contains_filter(&text_unicode, "nguoi"));
    }
}
