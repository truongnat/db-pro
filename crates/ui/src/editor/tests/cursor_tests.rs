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
fn horizontal_cursor_movement_skips_combining_sequences_and_zwj_emoji() {
    let buffer = TextBuffer::from_string("e\u{301}👩‍💻x");
    let mut cursor = CursorPosition::from_offset(&buffer, buffer.len_bytes());

    cursor.move_left(&buffer);
    assert_eq!(cursor.offset, 14, "one step should move before x");
    cursor.move_left(&buffer);
    assert_eq!(cursor.offset, 3, "one step should move before the whole ZWJ emoji");
    cursor.move_left(&buffer);
    assert_eq!(
        cursor.offset, 0,
        "one step should move before the whole combining grapheme"
    );

    cursor.move_right(&buffer);
    assert_eq!(cursor.offset, 3);
    cursor.move_right(&buffer);
    assert_eq!(cursor.offset, 14);
    cursor.move_right(&buffer);
    assert_eq!(cursor.offset, buffer.len_bytes());
}
