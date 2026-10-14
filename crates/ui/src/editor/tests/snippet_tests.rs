//! Tests for `../snippet.rs` (moved out of the source file).
use super::{SnippetMove, SnippetSession};
use crate::editor::buffer::TextBuffer;

#[test]
fn sql_parameters_are_not_tab_stops() {
    let (text, session) = SnippetSession::expand("INSERT INTO ${1:table_name} VALUES ($1, $2);");
    assert_eq!(text, "INSERT INTO table_name VALUES ($1, $2);");
    let session = session.expect("one stop");
    assert_eq!(session.order, vec![1]);
    assert_eq!(session.active_range(), Some((12, 22)));
}

#[test]
fn mirrors_share_an_index_and_tab_leaves_the_session() {
    let (text, session) = SnippetSession::expand("ON ${1:table_name} (${2:column}) FROM ${1:table_name}");
    assert!(text.contains("table_name"));
    let mut session = session.unwrap();
    assert!(matches!(session.move_by(1), SnippetMove::Select(_, _)));
    assert!(matches!(session.move_by(1), SnippetMove::Finished));
}

#[test]
fn editing_one_mirror_rewrites_the_other() {
    let (text, session) = SnippetSession::expand("IDX ${1:table_name} ON ${1:table_name}");
    let mut buffer = TextBuffer::from_string(text);
    let mut session = session.unwrap();
    let (start, end) = session.active_range().unwrap();
    buffer.replace(start, end, "orders");
    assert!(session.note_edit(&mut buffer));
    assert_eq!(buffer.text(), "IDX orders ON orders");
}
