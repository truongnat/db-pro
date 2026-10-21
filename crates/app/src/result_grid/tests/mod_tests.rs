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
    assert!(FilterNeedle::new("alpha").matches(&text_ascii));
    assert!(FilterNeedle::new("ALPHA").matches(&text_ascii));
    assert!(!FilterNeedle::new("beta").matches(&text_ascii));

    let text_unicode = UiCell::Text("người_dùng".to_owned());
    assert!(FilterNeedle::new("người").matches(&text_unicode));
    assert!(!FilterNeedle::new("nguoi").matches(&text_unicode));
}

/// #543 — the needle fast paths and the exact edge cases the streaming matcher must
/// keep from the old `cell.to_lowercase().contains(filter.to_lowercase())`.
#[test]
fn filter_needle_keeps_unicode_lowercase_edge_semantics() {
    // Turkish İ expands to two lowercase chars (i + combining dot above).
    let istanbul = UiCell::Text("İstanbul".to_owned());
    assert!(FilterNeedle::new("i").matches(&istanbul));
    // İ expands to i + U+0307, so the *plain ASCII* needle "istanbul" does NOT match
    // (exactly the old `to_lowercase().contains` behaviour) — while the decomposed
    // needle does.
    assert!(!FilterNeedle::new("istanbul").matches(&istanbul));
    assert!(FilterNeedle::new("i\u{307}stanbul").matches(&istanbul));
    assert!(FilterNeedle::new("stanbul").matches(&istanbul));
    assert!(FilterNeedle::new("İ".to_owned().to_lowercase().as_str()).matches(&istanbul));
    // Dotless ı is a different letter; an "i" needle must not match a cell whose
    // only i-like letters are dotless.
    assert!(!FilterNeedle::new("i").matches(&UiCell::Text("Kırkpınar".to_owned())));
    assert!(FilterNeedle::new("ı").matches(&UiCell::Text("Kırkpınar".to_owned())));
    assert!(FilterNeedle::new("i").matches(&UiCell::Text("diyarbakır".to_owned()))); // real ASCII i present
                                                                                     // ASCII cell + non-ASCII needle: rejected outright.
    assert!(!FilterNeedle::new("người").matches(&UiCell::Text("Customer Alpha".to_owned())));
    // Vietnamese round-trips through the streaming scan.
    assert!(FilterNeedle::new("NGUYỄN").matches(&UiCell::Text("Nguyễn Văn Toàn".to_owned())));
    assert!(FilterNeedle::new("toàn").matches(&UiCell::Text("Nguyễn Văn Toàn".to_owned())));
    // Kelvin sign (U+212A) lowercases to ASCII 'k' — bridge char mid-needle, the
    // case the raw byte scan alone cannot see.
    assert!(FilterNeedle::new("aki").matches(&UiCell::Text("a\u{212A}i".to_owned())));
    assert!(FilterNeedle::new("kil").matches(&UiCell::Text("\u{212A}ilo".to_owned())));
    // Greek final sigma keeps the allocating fallback (Σ→ς only at word end).
    assert!(FilterNeedle::new("οδυσσεας").matches(&UiCell::Text("ΟΔΥΣΣΕΑΣ".to_owned())));
    assert!(!FilterNeedle::new("οδυσσεας").matches(&UiCell::Text("ΟΔΥΣΣΕΙΑ".to_owned())));
    // Empty filter matches everything (guarded by the caller; matches() agrees).
    assert!(FilterNeedle::new("").matches(&UiCell::Null));
}

/// #543 soundness invariant: the byte fast paths assume that `İ` (U+0130) and the
/// KELVIN SIGN (U+212A) are the *only* code points whose lowercase expansion contains
/// an ASCII character. If that ever stops being true the fast paths are wrong, so pin
/// the enumeration itself.
#[test]
fn only_the_two_enumerated_bridge_codepoints_lowercase_to_ascii() {
    let bridges: Vec<char> = (0x80..0x110000u32)
        .filter_map(char::from_u32)
        .filter(|c| c.to_lowercase().any(|l| l.is_ascii()))
        .collect();
    assert_eq!(bridges, vec!['\u{130}', '\u{212A}']);
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

/// #543 — the grid filter used to `to_lowercase()` every cell on every keystroke
/// (the filter is part of `GridProjectionKey`): 123.7 ms/keystroke at 100k rows ×
/// 8 Vietnamese columns, 21.3 ms for the ASCII fixture. The matcher now normalizes
/// the query once and scans cells without allocating. These tests pin the keystroke
/// budget and, more importantly, row-set equality against the old semantics.
mod issue543_filter_tests {
    use super::*;

    fn fixture(rows: usize, cols: usize, cell: impl Fn(usize, usize) -> String) -> UiQueryResult {
        UiQueryResult {
            columns: (0..cols)
                .map(|index| crate::UiColumn {
                    name: format!("c{index}"),
                    data_type: "text".to_owned(),
                    nullable: true,
                })
                .collect(),
            rows: (0..rows)
                .map(|r| (0..cols).map(|c| UiCell::Text(cell(r, c))).collect())
                .collect(),
            row_count: rows as u64,
            duration_ms: 0,
        }
    }

    /// The exact semantics of the pre-fix path: lowercase the whole cell, then `contains`.
    fn legacy_row_set(result: &UiQueryResult, filter: &str) -> Vec<usize> {
        let filter_lower = filter.to_lowercase();
        result
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                filter_lower.is_empty()
                    || row
                        .iter()
                        .any(|cell| cell_text_as_str(cell).to_lowercase().contains(&filter_lower))
            })
            .map(|(index, _)| index)
            .collect()
    }

    #[test]
    fn filter_keeps_row_sets_of_the_legacy_lowercase_semantics() {
        let corpus: Vec<String> = vec![
            "Customer Alpha".to_owned(),
            "customer alpha".to_owned(),
            "CUSTOMER ALPHA".to_owned(),
            "Nguyễn Văn Toàn".to_owned(),
            "nguyễn văn toàn".to_owned(),
            "TRẦN THỊ BÔNG SEN".to_owned(),
            "hÀ nỘi thànH pHố".to_owned(),
            "İstanbul".to_owned(),   // İ → "i̇" two-char lowercase expansion
            "diyarbakır".to_owned(), // dotless ı must not match "i"
            "Kırkpınar".to_owned(),  // ASCII K + dotless ı, no ASCII i
            "İ".to_owned(),
            "i\u{307}".to_owned(),   // pre-decomposed form of İ's lowercase
            "a\u{212A}i".to_owned(), // KELVIN sign lowercases to ASCII k mid-needle
            "\u{212A}ilo".to_owned(),
            "ΟΔΥΣΣΕΙΑ".to_owned(), // capital sigma, one is word-final → ς
            "ΣΤΣ".to_owned(),
            "RESETΣ".to_owned(), // Latin word ending in a capital sigma
            "Straße".to_owned(),
            "STRASSE ẞ".to_owned(),
            "ǅungla".to_owned(), // titlecase letter lowercases to ǆ
            "emoji 🎉 filter".to_owned(),
            "a".repeat(65_000),
            "x".repeat(64_000) + "Nguyễn",
            "NULL-shaped".to_owned(),
            "2026-01-02T03:04:05Z".to_owned(),
        ];
        let mut result = fixture(corpus.len(), 2, |r, c| {
            if c == 0 {
                corpus[r].clone()
            } else {
                "row{r}".to_owned()
            }
        });
        result.rows.push(vec![UiCell::Null, UiCell::Number("42".to_owned())]);

        let filters = [
            "",
            " ",
            "alpha",
            "ALPHA",
            "AlPh",
            "nguyễn",
            "NGUYỄN VĂN TOÀN",
            "toàn",
            "toi",
            "bông",
            "istanbul",
            "I",
            "i",
            "İ",
            "ı",
            "aki",
            "kil",
            "kı",
            "kırkpınar",
            "οδυσσεια",
            "ΟΔΥΣΣΕΙΑ",
            "ς",
            "σ",
            "στς",
            "resetσ",
            "resetς",
            "strasse",
            "ß",
            "ẞ",
            "ǆ",
            "ǅ",
            "🎉",
            "NULL",
            "42",
            "row17",
            "zzz_nomatch",
            "Nguyễn zzz",
        ];
        for filter in filters {
            assert_eq!(
                filtered_sorted_indexes(&result, filter, None, false),
                legacy_row_set(&result, filter),
                "row set diverged from the legacy to_lowercase semantics for filter {filter:?}"
            );
        }
    }

    /// Release-mode acceptance from the issue: 100k × 8 Vietnamese cells, filter
    /// matching nothing (full scan). Before: 123.7 ms/keystroke (21.3 ms ASCII).
    #[test]
    fn keystroke_budget_at_100k_vietnamese_rows() {
        let vi = fixture(100_000, 8, |r, c| format!("Nguyễn Thị Hồng Nhung #{r}-{c}"));
        let ascii = fixture(100_000, 8, |r, c| format!("Nguyen Thi Hong Nhung #{r}-{c}"));
        let mut measured = Vec::new();
        for (name, result) in [("vietnamese", &vi), ("ascii", &ascii)] {
            let _ = filtered_sorted_indexes(result, "zzz_nomatch", None, false); // warm-up
            let mut best = std::time::Duration::MAX;
            for _ in 0..5 {
                let t = std::time::Instant::now();
                let n = filtered_sorted_indexes(result, "zzz_nomatch", None, false).len();
                let elapsed = t.elapsed();
                assert_eq!(n, 0, "fixture must not match the probe filter");
                best = best.min(elapsed);
            }
            measured.push((name, best));
        }
        for (name, best) in &measured {
            println!("#543 filter 100k×8 {name}: best {best:?} (issue before: VI 123.7ms / ASCII 21.3ms)");
        }
        if !cfg!(debug_assertions) {
            for (name, best) in &measured {
                assert!(
                    best.as_millis() <= 25,
                    "#543: {name} filter keystroke cost {best:?}, budget 25 ms at 100k×8"
                );
            }
        }
    }
}
