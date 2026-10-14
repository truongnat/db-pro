//! Tests for `../config.rs` (moved out of the source file).
use super::*;

#[test]
fn table_metrics_preserve_the_existing_coordinate_contract() {
    assert_eq!(CHECKBOX_COLUMN_WIDTH, 42.0);
    assert_eq!(HEADER_HEIGHT, TABLE_HEADER_HEIGHT);
    assert_eq!(ROW_HEIGHT_DEFAULT, TABLE_ROW_HEIGHT_DEFAULT);
    assert_eq!(CHECKBOX_CORNER_RADIUS, RADIUS_XS);
    assert_eq!(HEADER_INNER_RADIUS, RADIUS_SM);
    assert_eq!(INNER_PADDING_X, 12.0);
}
