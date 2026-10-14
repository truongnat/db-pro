//! Tests for `../settings.rs` (moved out of the source file).
use super::*;

#[test]
fn parse_enumvals_postgres_array() {
    assert_eq!(
        parse_enumvals("{on,off}".into()).as_deref(),
        Some(["on".to_owned(), "off".to_owned()].as_slice())
    );
}
