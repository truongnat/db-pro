//! Tests for `../buffer.rs` (moved out of the source file).
use super::*;

#[test]
fn test_text_buffer_insert_delete_and_lines() {
    let mut buf = TextBuffer::from_string("SELECT * FROM users;\nSELECT * FROM orders;");
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.line_at(0), Some("SELECT * FROM users;"));
    assert_eq!(buf.line_at(1), Some("SELECT * FROM orders;"));

    buf.insert(0, "-- Comment\n");
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.line_at(0), Some("-- Comment"));
    assert_eq!(buf.line_at(1), Some("SELECT * FROM users;"));

    let (line, col) = buf.offset_to_line_col(11);
    assert_eq!(line, 1);
    assert_eq!(col, 0);

    let off = buf.line_col_to_offset(1, 7);
    assert_eq!(off, 18);
    assert_eq!(buf.slice(off, off + 6), "* FROM");
}

#[test]
fn test_text_buffer_undo_redo_snapshots() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "SELECT 1;");
    assert_eq!(buf.text(), "SELECT 1;");

    buf.insert(9, "\nSELECT 2;");
    assert_eq!(buf.text(), "SELECT 1;\nSELECT 2;");

    let undo_res = buf.undo();
    assert_eq!(undo_res, Some((9, 9)));
    assert_eq!(buf.text(), "SELECT 1;");

    let redo_res = buf.redo();
    assert_eq!(redo_res, Some((19, 19)));
    assert_eq!(buf.text(), "SELECT 1;\nSELECT 2;");
}

#[test]
fn replacement_is_one_undo_step_and_restores_original_text() {
    let mut buf = TextBuffer::from_string("SELECT cust");
    buf.replace(7, 11, "customer");
    assert_eq!(buf.text(), "SELECT customer");
    assert_eq!(buf.undo(), Some((11, 7)));
    assert_eq!(buf.text(), "SELECT cust");
}

#[test]
fn test_text_buffer_typing_undo_grouping() {
    let mut buf = TextBuffer::new();
    // Type characters 'S', 'E', 'L' consecutively
    buf.type_text(
        0,
        "S",
        EditorSnapshot {
            cursor_offset: 0,
            anchor_offset: 0,
        },
        EditorSnapshot {
            cursor_offset: 1,
            anchor_offset: 1,
        },
    );
    buf.type_text(
        1,
        "E",
        EditorSnapshot {
            cursor_offset: 1,
            anchor_offset: 1,
        },
        EditorSnapshot {
            cursor_offset: 2,
            anchor_offset: 2,
        },
    );
    buf.type_text(
        2,
        "L",
        EditorSnapshot {
            cursor_offset: 2,
            anchor_offset: 2,
        },
        EditorSnapshot {
            cursor_offset: 3,
            anchor_offset: 3,
        },
    );

    assert_eq!(buf.text(), "SEL");
    // A single undo should undo the grouped typing word back to empty!
    let undo_res = buf.undo();
    assert_eq!(undo_res, Some((0, 0)));
    assert_eq!(buf.text(), "");
}

#[test]
fn test_text_buffer_utf8_boundaries() {
    let buf = TextBuffer::from_string("SELECT 'Xin chào thế giới';");
    // "SELECT 'Xin ch" is 14 bytes (0..14). 'à' is 2 bytes at offset 14..16.
    let offset = 15;
    let floor = buf.floor_char_boundary(offset);
    assert_eq!(floor, 14);
    let ceil = buf.ceil_char_boundary(offset);
    assert_eq!(ceil, 16);
    assert_eq!(buf.char_at(floor), Some('à'));
    assert_eq!(buf.prev_char(ceil), Some('à'));
}
