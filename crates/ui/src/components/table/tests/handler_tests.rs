//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

fn columns() -> [TableColumn<'static>; 3] {
    [
        TableColumn::fixed("ID", 100.0),
        TableColumn::new("Name"),
        TableColumn::new("Status"),
    ]
}

#[test]
fn layout_preserves_checkbox_space_and_column_offsets() {
    let layout = build_column_layout(&columns(), 500.0, true);

    assert_eq!(layout.checkbox_width, CHECKBOX_COLUMN_WIDTH);
    assert_eq!(layout.offsets, vec![0.0, 100.0, 278.0]);
    assert_eq!(layout.widths, vec![100.0, 178.0, 178.0]);
    assert_eq!(layout.total_width, 498.0);
}

#[test]
fn geometry_keeps_empty_and_populated_table_heights_distinct() {
    assert_eq!(TableGeometry::new(0, 44.0).rows_height, EMPTY_BODY_HEIGHT);
    assert_eq!(TableGeometry::new(3, 44.0).rows_height, 132.0);
    assert_eq!(TableGeometry::new(3, 44.0).total_height, 169.0);
}

#[test]
fn select_all_state_prioritizes_checked_over_indeterminate() {
    assert_eq!(select_all_state(false, false), SelectAllState::Unchecked);
    assert_eq!(select_all_state(false, true), SelectAllState::Indeterminate);
    assert_eq!(select_all_state(true, true), SelectAllState::Checked);
}
