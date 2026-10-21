//! Tests for `../runtime_query_types.rs`.
use super::*;

fn truncated_text() -> String {
    let mut value = "x".repeat(MAX_CELL_TEXT_BYTES);
    value.push('…');
    value
}

// #537: the runtime truncation marker must be detectable so the grid can refuse
// to write a display-only prefix back over the full value.
#[test]
fn truncated_text_and_json_are_flagged() {
    let text = UiCell::Text(truncated_text());
    assert!(text.is_truncated());
    assert!(UiCell::Json(truncated_text()).is_truncated());
}

#[test]
fn text_at_the_cap_without_a_marker_is_not_flagged() {
    assert!(!UiCell::Text("x".repeat(MAX_CELL_TEXT_BYTES)).is_truncated());
    assert!(!UiCell::Text("x".repeat(MAX_CELL_TEXT_BYTES - 1)).is_truncated());
    assert!(!UiCell::Text("short …".to_owned()).is_truncated());
}

#[test]
fn bytes_truncation_marker_is_flagged_but_a_full_hex_render_is_not() {
    let mut truncated = format!("\\x{}", "ab".repeat(MAX_CELL_TEXT_BYTES / 2));
    truncated.push('…');
    assert!(UiCell::Bytes(truncated).is_truncated());

    let full = format!("\\x{}", "ab".repeat(MAX_CELL_TEXT_BYTES / 2));
    assert!(!UiCell::Bytes(full).is_truncated());
}

#[test]
fn scalar_cells_are_never_truncated() {
    assert!(!UiCell::Null.is_truncated());
    assert!(!UiCell::Boolean(true).is_truncated());
    assert!(!UiCell::Number("1".repeat(MAX_CELL_TEXT_BYTES + 1)).is_truncated());
}
