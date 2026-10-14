//! Tests for `../rls_manager.rs` (moved out of the source file).
use super::parse_roles;

#[test]
fn parse_roles_handles_pg_array_text() {
    assert!(parse_roles(None).is_empty());
    assert!(parse_roles(Some("{}".into())).is_empty());
    assert_eq!(
        parse_roles(Some("{app_reader,app_writer}".into())),
        vec!["app_reader".to_owned(), "app_writer".to_owned()]
    );
}
