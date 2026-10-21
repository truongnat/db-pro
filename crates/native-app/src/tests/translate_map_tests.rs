//! Tests for `../translate_map.rs` (moved out of the source file).
use super::*;
use db_pro_core::domain::query::CellValue;

// #537: a value the grid cannot hold in full must be recognisable as truncated;
// otherwise the editor stages the display prefix back over the real value.
#[test]
fn oversized_text_is_marked_as_truncated() {
    let mapped = map_cell(CellValue::Text("x".repeat(70_000)));

    let UiCell::Text(value) = &mapped else {
        panic!("text maps to UiCell::Text");
    };
    assert_eq!(value.len(), MAX_CELL_TEXT_BYTES + '…'.len_utf8());
    assert!(value.ends_with('…'));
    assert!(mapped.is_truncated());
}

#[test]
fn text_at_the_cap_is_not_marked_truncated() {
    let mapped = map_cell(CellValue::Text("x".repeat(MAX_CELL_TEXT_BYTES)));

    assert!(matches!(&mapped, UiCell::Text(value) if value.len() == MAX_CELL_TEXT_BYTES));
    assert!(!mapped.is_truncated());
}

#[test]
fn oversized_json_is_marked_as_truncated() {
    let json = format!("\"{}\"", "x".repeat(70_000));
    let mapped = map_cell(CellValue::Json(serde_json::from_str(&json).expect("valid JSON")));

    assert!(matches!(mapped, UiCell::Json(_)));
    assert!(mapped.is_truncated());
}

#[test]
fn oversized_bytes_are_marked_as_truncated() {
    let mapped = map_cell(CellValue::Bytes(vec![0x41; MAX_CELL_TEXT_BYTES / 2 + 1]));
    assert!(matches!(mapped, UiCell::Bytes(_)));
    assert!(mapped.is_truncated());

    let full = map_cell(CellValue::Bytes(vec![0x41; MAX_CELL_TEXT_BYTES / 2]));
    assert!(!full.is_truncated(), "a bytea value at the cap is not truncated");
}
