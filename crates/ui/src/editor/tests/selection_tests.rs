//! Tests for `../selection.rs` (moved out of the source file).
use super::*;

#[test]
fn test_select_word_at() {
    let text = "SELECT user_id, user_name FROM users;";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    sel.select_word_at(&buf, 9); // inside "user_id"
    assert_eq!(sel.normalized(), (7, 14));
    assert_eq!(buf.slice(sel.anchor, sel.active), "user_id");

    sel.select_word_at(&buf, 0); // "SELECT"
    assert_eq!(sel.normalized(), (0, 6));
    assert_eq!(buf.slice(sel.anchor, sel.active), "SELECT");
}

#[test]
fn test_select_line_at() {
    let text = "SELECT 1;\nSELECT 2;\nSELECT 3;";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    sel.select_line_at(&buf, 1);
    assert_eq!(buf.slice(sel.anchor, sel.active), "SELECT 2;\n");
}

#[test]
fn select_word_at_includes_combining_marks_in_the_word() {
    let text = "cafe\u{301} other";
    let buffer = TextBuffer::from_string(text);
    let mut selection = SelectionRange::default();

    selection.select_word_at(&buffer, 4);

    assert_eq!(buffer.slice(selection.anchor, selection.active), "cafe\u{301}");
}

#[test]
fn select_word_at_keeps_multiple_vietnamese_marks_with_the_base_letter() {
    let text = "a\u{302}\u{301} next";
    let buffer = TextBuffer::from_string(text);
    let mut selection = SelectionRange::default();

    selection.select_word_at(&buffer, 2);

    assert_eq!(buffer.slice(selection.anchor, selection.active), "a\u{302}\u{301}");
}
