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
fn select_word_keeps_combining_marks_inside_the_word() {
    // Issue #506: per-char is_alphanumeric split words at combining marks.
    // "café" written decomposed: 'e' + U+0301 combining acute.
    let text = "SELECT cafe\u{301} FROM t";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    // Click inside the combining mark itself — still the same word.
    let mark_offset = text.find('\u{301}').unwrap();
    sel.select_word_at(&buf, mark_offset);
    assert_eq!(buf.slice(sel.anchor, sel.active), "cafe\u{301}");

    // Click on the base 'e'.
    sel.select_word_at(&buf, mark_offset - 1);
    assert_eq!(buf.slice(sel.anchor, sel.active), "cafe\u{301}");
}

#[test]
fn select_word_handles_decomposed_vietnamese_tone() {
    // "ấ" decomposed is 'a' + U+0302 + U+0301 — three code points, one grapheme.
    let text = "WHERE a\u{302}\u{301} = 1";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    sel.select_word_at(&buf, text.find('a').unwrap() + 1);
    assert_eq!(buf.slice(sel.anchor, sel.active), "a\u{302}\u{301}");
}

#[test]
fn select_word_at_end_of_buffer_selects_last_word() {
    let text = "SELECT id";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    sel.select_word_at(&buf, buf.len_bytes());
    assert_eq!(buf.slice(sel.anchor, sel.active), "id");
}

#[test]
fn test_select_line_at() {
    let text = "SELECT 1;\nSELECT 2;\nSELECT 3;";
    let buf = TextBuffer::from_string(text);

    let mut sel = SelectionRange::default();
    sel.select_line_at(&buf, 1);
    assert_eq!(buf.slice(sel.anchor, sel.active), "SELECT 2;\n");
}
