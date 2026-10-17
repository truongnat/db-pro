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
    // `None` means the selection survived a filter/sort change that dropped its row from the
    // projection (#505). Arrow keys then re-enter the visible range instead of assuming
    // position 0 (which skipped the first row on ArrowDown and made ArrowUp a no-op).
    let row_position = visible_rows.iter().position(|row| *row == selected_row);
    let column = selected_column.min(column_count - 1);

    match key {
        eframe::egui::Key::ArrowUp => match row_position {
            Some(position) => visible_rows.get(position.saturating_sub(1)).copied(),
            None => visible_rows.last().copied(),
        }
        .map(|row| (row, column)),
        eframe::egui::Key::ArrowDown => match row_position {
            Some(position) => visible_rows.get((position + 1).min(visible_rows.len() - 1)).copied(),
            None => visible_rows.first().copied(),
        }
        .map(|row| (row, column)),
        eframe::egui::Key::ArrowLeft => Some((visible_rows[row_position.unwrap_or(0)], column.saturating_sub(1))),
        eframe::egui::Key::ArrowRight => Some((
            visible_rows[row_position.unwrap_or(0)],
            (column + 1).min(column_count - 1),
        )),
        eframe::egui::Key::Home => Some((visible_rows[row_position.unwrap_or(0)], 0)),
        eframe::egui::Key::End => Some((visible_rows[row_position.unwrap_or(0)], column_count - 1)),
        _ => None,
    }
}

use std::cmp::Ordering;

use bigdecimal::BigDecimal;

/// Parsed once per sorted cell. The comparator then compares these keys and
/// only falls back to the original text when the two cells are not the same type.
enum SortKey {
    Missing,
    Bool(bool),
    Number(Option<BigDecimal>),
    Rfc(chrono::DateTime<chrono::FixedOffset>),
    Naive(chrono::NaiveDateTime),
    Date(chrono::NaiveDate),
    Text,
}

fn sort_key(cell: Option<&UiCell>) -> SortKey {
    match cell {
        None | Some(UiCell::Null) => SortKey::Missing,
        Some(UiCell::Boolean(value)) => SortKey::Bool(*value),
        Some(UiCell::Number(value)) => SortKey::Number(value.parse().ok()),
        Some(UiCell::Text(value)) if looks_like_iso_temporal(value) => {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(value) {
                SortKey::Rfc(parsed)
            } else if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
                SortKey::Naive(parsed)
            } else if let Ok(parsed) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                SortKey::Date(parsed)
            } else {
                SortKey::Text
            }
        }
        Some(_) => SortKey::Text,
    }
}

fn cmp_sort_keys(left: &SortKey, right: &SortKey, left_cell: Option<&UiCell>, right_cell: Option<&UiCell>) -> Ordering {
    match (left, right) {
        (SortKey::Missing, SortKey::Missing) => Ordering::Equal,
        (SortKey::Missing, _) => Ordering::Greater,
        (_, SortKey::Missing) => Ordering::Less,
        (SortKey::Bool(left), SortKey::Bool(right)) => left.cmp(right),
        (SortKey::Number(Some(left)), SortKey::Number(Some(right))) => left.cmp(right),
        (SortKey::Number(_), SortKey::Number(_)) => cell_text_cmp(left_cell, right_cell),
        (SortKey::Rfc(left), SortKey::Rfc(right)) => left.cmp(right),
        (SortKey::Naive(left), SortKey::Naive(right)) => left.cmp(right),
        (SortKey::Date(left), SortKey::Date(right)) => left.cmp(right),
        _ => cell_text_cmp(left_cell, right_cell),
    }
}

fn cell_text_cmp(left: Option<&UiCell>, right: Option<&UiCell>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => cell_text_as_str(left).cmp(cell_text_as_str(right)),
        _ => Ordering::Equal,
    }
}

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
        // Parse each cell once. Comparing the keys avoids parsing decimals and
        // timestamps again inside every O(n log n) comparison.
        let keys: Vec<SortKey> = indexes
            .iter()
            .map(|&index| sort_key(result.rows[index].get(column)))
            .collect();
        let mut order: Vec<usize> = (0..indexes.len()).collect();
        order.sort_by(|&left, &right| {
            let left_cell = result.rows[indexes[left]].get(column);
            let right_cell = result.rows[indexes[right]].get(column);
            let ordering = cmp_sort_keys(&keys[left], &keys[right], left_cell, right_cell);
            if sort_desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
        indexes = order.into_iter().map(|position| indexes[position]).collect();
    }

    indexes
}

/// The inputs that decide one filtered/sorted row projection.
///
/// `epoch` is the caller's monotonic id for the row data behind the grid: replacing a result set, or
/// editing a displayed row in place, must advance it (`TableDataState::invalidate_grid_projection`). The
/// remaining fields are the grid's own filter/sort state plus a shape guard, so a projection is
/// reused across frames until one of these inputs really changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridProjectionKey {
    pub epoch: u64,
    pub filter: String,
    pub sort_column: Option<usize>,
    pub sort_desc: bool,
    pub row_count: u64,
    pub column_count: usize,
}

/// One-entry memo for [`filtered_sorted_indexes`].
///
/// Rebuilding the projection on every frame is what made a sorted large result unusable: the
/// comparator parses temporal text per comparison, so a 200k-row sort on a timestamp-shaped column
/// measured 10.5 s *per frame* in debug (`docs/release/evidence/v01-runtime/providers/53-...md`).
/// The cache is deliberately dumb — a single entry and no eviction — because a grid draws exactly
/// one result at a time, and the epoch in the key is what keeps it honest: a stale projection can
/// only survive if a caller changes row data without advancing the epoch.
#[derive(Default)]
pub struct GridProjectionCache {
    /// Key the cached projection was built for, or `None` while a frame holds the rows.
    key: Option<GridProjectionKey>,
    indexes: Vec<usize>,
    rebuilds: u64,
}

impl GridProjectionCache {
    /// Take the projection for `key`, rebuilding it only when the cache holds a different key.
    ///
    /// The rows are *moved out*: the draw path needs them as an owned local while it keeps `&mut
    /// self` available for the rest of the frame. [`Self::restore`] hands them back.
    pub fn take(&mut self, key: GridProjectionKey, result: &UiQueryResult) -> Vec<usize> {
        if self.key.as_ref() == Some(&key) {
            self.key = None;
            return std::mem::take(&mut self.indexes);
        }
        self.key = None;
        self.indexes.clear();
        self.rebuilds = self.rebuilds.saturating_add(1);
        filtered_sorted_indexes(result, &key.filter, key.sort_column, key.sort_desc)
    }

    /// Store a projection the caller got from [`Self::take`] so the next frame can reuse it.
    ///
    /// A projection cached meanwhile under the same-or-different key wins: dropping the frame's copy
    /// only costs one rebuild, whereas overwriting a newer entry could serve a stale order.
    pub fn restore(&mut self, key: GridProjectionKey, indexes: Vec<usize>) {
        if self.key.is_some() {
            return;
        }
        self.key = Some(key);
        self.indexes = indexes;
    }

    /// Number of projections built by this cache. Asserted by tests to pin the
    /// "no per-frame rebuild" property, which is invisible in the rendered frame.
    pub fn rebuilds(&self) -> u64 {
        self.rebuilds
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridColumnWindow {
    pub start: usize,
    pub end: usize,
    pub leading: f32,
    pub trailing: f32,
}

/// Columns that intersect the horizontal viewport, plus a small overscan.
///
/// `leading` and `trailing` are the widths of the columns skipped on each side
/// so the row still occupies the full scroll width.
pub fn grid_column_window(
    order: &[usize],
    widths: &[f32],
    gutter: f32,
    visible_left: f32,
    visible_right: f32,
) -> GridColumnWindow {
    let full = GridColumnWindow {
        start: 0,
        end: order.len(),
        leading: 0.0,
        trailing: 0.0,
    };
    if order.is_empty() || visible_right - visible_left < 1.0 {
        return full;
    }
    const OVERSCAN: f32 = 80.0;
    let left = visible_left - OVERSCAN;
    let right = visible_right + OVERSCAN;
    let width_of = |column: usize| widths.get(column).copied().unwrap_or(180.0);
    let mut x = gutter;
    let mut start = 0;
    let mut end = order.len();
    for (index, &column) in order.iter().enumerate() {
        let width = width_of(column);
        let next = x + width;
        if next < left {
            start = index + 1;
        }
        if x > right {
            end = index;
            break;
        }
        x = next;
    }
    if start > end {
        start = end;
    }
    GridColumnWindow {
        start,
        end,
        leading: order[..start].iter().map(|column| width_of(*column)).sum(),
        trailing: order[end..].iter().map(|column| width_of(*column)).sum(),
    }
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
