//! Tests for `../mod.rs` (moved out of the source file).
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
fn column_window_skips_columns_outside_the_viewport() {
    let order = [0, 1, 2, 3];
    let widths = [100.0, 100.0, 100.0, 100.0];
    let window = grid_column_window(&order, &widths, 48.0, 48.0, 180.0);
    assert_eq!(window.start, 0);
    assert!(window.end < order.len());
    assert!(window.trailing > 0.0);
    let wide = grid_column_window(&order, &widths, 48.0, 0.0, 10_000.0);
    assert_eq!((wide.start, wide.end), (0, 4));
}

#[test]
fn prepared_sort_keys_match_pairwise_cell_order() {
    let cells = [
        UiCell::Null,
        UiCell::Number("10".to_owned()),
        UiCell::Number("2".to_owned()),
        UiCell::Number("not-a-number".to_owned()),
        UiCell::Text("2024-01-02".to_owned()),
        UiCell::Text("2024-01-02T00:00:00Z".to_owned()),
        UiCell::Text("2024-03-01 10:00:00".to_owned()),
        UiCell::Text("alice".to_owned()),
        UiCell::Boolean(false),
        UiCell::Boolean(true),
        UiCell::Json("{\"b\":1}".to_owned()),
    ];
    let result = UiQueryResult {
        columns: vec![],
        rows: cells.iter().map(|cell| vec![cell.clone()]).collect(),
        row_count: cells.len() as u64,
        duration_ms: 0,
    };
    let indexes = filtered_sorted_indexes(&result, "", Some(0), false);
    for pair in indexes.windows(2) {
        let left = result.rows[pair[0]].first();
        let right = result.rows[pair[1]].first();
        assert_ne!(
            compare_ui_cells(left, right),
            std::cmp::Ordering::Greater,
            "row {} sorted after row {}",
            pair[0],
            pair[1]
        );
    }
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

fn cache_result() -> UiQueryResult {
    UiQueryResult {
        columns: (0..2)
            .map(|index| crate::UiColumn {
                name: format!("c{index}"),
                data_type: "text".to_owned(),
                nullable: true,
            })
            .collect(),
        rows: vec![
            vec![UiCell::Text("2".to_owned()), UiCell::Text("Beta".to_owned())],
            vec![UiCell::Text("1".to_owned()), UiCell::Text("Alpha".to_owned())],
            vec![UiCell::Text("3".to_owned()), UiCell::Text("Gamma".to_owned())],
        ],
        row_count: 3,
        duration_ms: 0,
    }
}

fn key(epoch: u64, filter: &str, sort_column: Option<usize>, sort_desc: bool) -> GridProjectionKey {
    let result = cache_result();
    GridProjectionKey {
        epoch,
        filter: filter.to_owned(),
        sort_column,
        sort_desc,
        row_count: result.row_count,
        column_count: result.columns.len(),
    }
}

#[test]
fn projection_cache_reuses_the_projection_across_frames() {
    let result = cache_result();
    let mut cache = GridProjectionCache::default();

    let first = cache.take(key(0, "", None, false), &result);
    assert_eq!(first, vec![0, 1, 2]);
    assert_eq!(cache.rebuilds(), 1);
    cache.restore(key(0, "", None, false), first);

    // Two more frames with unchanged inputs: the rows come back without a second rebuild.
    for _ in 0..2 {
        let rows = cache.take(key(0, "", None, false), &result);
        assert_eq!(rows, vec![0, 1, 2]);
        cache.restore(key(0, "", None, false), rows);
    }
    assert_eq!(cache.rebuilds(), 1);
}

#[test]
fn projection_cache_rebuilds_when_an_input_changes() {
    let result = cache_result();
    let mut cache = GridProjectionCache::default();
    let sorted = |cache: &mut GridProjectionCache, key: GridProjectionKey| {
        let rows = cache.take(key.clone(), &result);
        cache.restore(key, rows.clone());
        rows
    };

    assert_eq!(sorted(&mut cache, key(0, "", Some(0), false)), vec![1, 0, 2]);
    assert_eq!(cache.rebuilds(), 1);

    // Sorted descending: same epoch, different direction.
    assert_eq!(sorted(&mut cache, key(0, "", Some(0), true)), vec![2, 0, 1]);
    assert_eq!(cache.rebuilds(), 2);

    // Filtered.
    assert_eq!(sorted(&mut cache, key(0, "gamma", Some(0), true)), vec![2]);
    assert_eq!(cache.rebuilds(), 3);

    // A new result behind the same filter/sort: only the epoch differs.
    assert_eq!(sorted(&mut cache, key(1, "gamma", Some(0), true)), vec![2]);
    assert_eq!(cache.rebuilds(), 4);
}

#[test]
fn projection_cache_rebuilds_when_the_row_shape_changes() {
    let result = cache_result();
    let mut cache = GridProjectionCache::default();
    cache.restore(key(0, "", None, false), vec![0, 1, 2]);

    // Same epoch, but a result with a different shape cannot reuse the cached rows.
    let mut wider = key(0, "", None, false);
    wider.column_count = 4;
    assert_eq!(cache.take(wider, &result), vec![0, 1, 2]);
    assert_eq!(cache.rebuilds(), 1);
}

#[test]
fn projection_cache_keeps_a_newer_entry_over_the_frame_copy() {
    let result = cache_result();
    let mut cache = GridProjectionCache::default();
    let stale_key = key(0, "", None, false);
    let frame_rows = cache.take(stale_key.clone(), &result);

    // Something rebuilt during the same frame (a toolbar action changed the sort).
    let newer_key = key(0, "", Some(1), false);
    cache.restore(newer_key.clone(), vec![1, 2, 0]);

    // The frame's own copy must not overwrite the newer entry.
    cache.restore(stale_key, frame_rows);
    assert_eq!(cache.take(newer_key, &result), vec![1, 2, 0]);
    assert_eq!(cache.rebuilds(), 1);
}

// #505: a selection whose row left the filtered/sorted projection must re-enter the
// visible range instead of treating the row as position 0 (which skipped the first
// row on ArrowDown and dead-ended ArrowUp).
#[test]
fn keyboard_selection_reenters_projection_from_filtered_out_row() {
    let visible_rows = vec![4, 7, 9];

    assert_eq!(
        grid_keyboard_selection(Some((3, 1)), &visible_rows, 3, eframe::egui::Key::ArrowDown),
        Some((4, 1))
    );
    assert_eq!(
        grid_keyboard_selection(Some((3, 1)), &visible_rows, 3, eframe::egui::Key::ArrowUp),
        Some((9, 1))
    );
    // Home/End keep their row anchored on the first visible row (unchanged fallback).
    assert_eq!(
        grid_keyboard_selection(Some((3, 1)), &visible_rows, 3, eframe::egui::Key::Home),
        Some((4, 0))
    );
    assert_eq!(
        grid_keyboard_selection(Some((3, 1)), &visible_rows, 3, eframe::egui::Key::End),
        Some((4, 2))
    );
}

#[test]
fn keyboard_selection_moves_within_projection_when_row_is_visible() {
    let visible_rows = vec![4, 7, 9];

    assert_eq!(
        grid_keyboard_selection(Some((7, 0)), &visible_rows, 3, eframe::egui::Key::ArrowDown),
        Some((9, 0))
    );
    assert_eq!(
        grid_keyboard_selection(Some((7, 0)), &visible_rows, 3, eframe::egui::Key::ArrowUp),
        Some((4, 0))
    );
}
