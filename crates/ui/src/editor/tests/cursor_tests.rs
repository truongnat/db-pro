//! Tests for `../cursor.rs` (moved out of the source file).
use super::*;

#[test]
fn test_cursor_vertical_movement_preserves_preferred_column() {
    let text = "SELECT very_long_column_name_here\nFROM tbl\nWHERE id = 100;";
    let buf = TextBuffer::from_string(text);

    let mut cursor = CursorPosition::from_line_col(&buf, 0, 20);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.col, 20);

    // Move down to short line (len 8 chars)
    cursor.move_down(&buf);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.col, 8); // clamped to end of "FROM tbl"
    assert_eq!(cursor.preferred_column, Some(20));

    // Move down to third line
    cursor.move_down(&buf);
    assert_eq!(cursor.line, 2);
    assert_eq!(cursor.col, 15); // "WHERE id = 100;" is 15 chars, clamped to 15
    assert_eq!(cursor.preferred_column, Some(20));

    // Move up back to first line
    cursor.move_up(&buf);
    cursor.move_up(&buf);
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.col, 20); // restored preferred column 20!
}

#[test]
fn test_cursor_utf8_boundary_safety() {
    let text = "SELECT 'Tiếng Việt';";
    let buf = TextBuffer::from_string(text);

    let mut cursor = CursorPosition::from_offset(&buf, 0);
    // Step through characters
    for _ in 0..20 {
        cursor.move_right(&buf);
        assert!(buf.text().is_char_boundary(cursor.offset));
    }
    for _ in 0..20 {
        cursor.move_left(&buf);
        assert!(buf.text().is_char_boundary(cursor.offset));
    }
}

#[test]
fn move_left_right_step_whole_grapheme_clusters() {
    // 👩‍💻 is one grapheme but five scalars (woman + ZWJ + laptop).
    let text = "a 👩\u{200d}💻 b";
    let buf = TextBuffer::from_string(text);
    let emoji_start = text.find('👩').unwrap();
    let emoji_end = emoji_start + "👩\u{200d}💻".len();

    let mut cursor = CursorPosition::from_offset(&buf, emoji_end);
    cursor.move_left(&buf);
    assert_eq!(cursor.offset, emoji_start, "left must not stop mid-cluster");
    cursor.move_right(&buf);
    assert_eq!(cursor.offset, emoji_end, "right must not stop mid-cluster");
}

#[test]
fn move_left_skips_combining_mark_with_base_letter() {
    // "é" decomposed: 'e' + U+0301 — one grapheme, two scalars.
    let text = "e\u{301}x";
    let buf = TextBuffer::from_string(text);
    let cluster_end = "e\u{301}".len();

    let mut cursor = CursorPosition::from_offset(&buf, cluster_end);
    cursor.move_left(&buf);
    assert_eq!(cursor.offset, 0, "combining mark must move with its base");
    cursor.move_right(&buf);
    assert_eq!(cursor.offset, cluster_end);
}
