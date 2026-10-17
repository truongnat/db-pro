//! Tests for `../mod.rs` (moved out of the source file).
use super::*;

fn pt(x: f64, label: &str) -> ChartPoint {
    ChartPoint {
        x,
        y: 1.0,
        label: label.to_owned(),
        series: String::new(),
    }
}

#[test]
fn x_axis_labels_keep_first_appearance_order_and_data_index() {
    // Issue #495: HashSet made label order nondeterministic per frame, and
    // positions used the unique-list index instead of the data index.
    let points = [pt(0.0, "b"), pt(1.0, "a"), pt(2.0, "b"), pt(3.0, "c")];
    assert_eq!(
        x_axis_labels(&points),
        vec![("b".to_owned(), 0), ("a".to_owned(), 1), ("c".to_owned(), 3)]
    );
}

#[test]
fn x_axis_labels_empty_points() {
    assert!(x_axis_labels(&[]).is_empty());
}

#[test]
fn evenly_spread_indices_include_first_and_last() {
    assert_eq!(evenly_spread_indices(10, 8), vec![0, 1, 2, 3, 5, 6, 7, 9]);
    assert_eq!(evenly_spread_indices(9, 8).len(), 8);
    assert_eq!(*evenly_spread_indices(9, 8).last().unwrap(), 8);
    // More ticks than labels: one index per label.
    assert_eq!(evenly_spread_indices(3, 8), vec![0, 1, 2]);
    // Edge cases: no labels or no ticks.
    assert!(evenly_spread_indices(0, 8).is_empty());
    assert!(evenly_spread_indices(8, 0).is_empty());
    assert_eq!(evenly_spread_indices(1, 8), vec![0]);
    // All indices stay in bounds.
    let idx = evenly_spread_indices(7, 8);
    assert!(idx.iter().all(|&i| i < 7));
}
