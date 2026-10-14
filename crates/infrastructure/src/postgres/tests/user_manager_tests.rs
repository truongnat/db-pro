//! Tests for `../user_manager.rs` (moved out of the source file).
use super::*;

#[test]
fn quote_identifier_escapes_embedded_quotes() {
    assert_eq!(quote_identifier(r#"role"name"#, "role").unwrap(), r#""role""name""#);
}

#[test]
fn quote_identifier_rejects_empty_and_null_values() {
    assert!(matches!(quote_identifier(" ", "role"), Err(DbError::Validation(_))));
    assert!(matches!(
        quote_identifier("role\0name", "role"),
        Err(DbError::Validation(_))
    ));
}

#[test]
fn quote_privilege_allows_known_table_privileges_only() {
    assert_eq!(quote_table_privilege(" select ").unwrap(), "SELECT");
    assert!(matches!(
        quote_table_privilege("SELECT; DROP TABLE users"),
        Err(DbError::Validation(_))
    ));
}

#[test]
fn unsupported_sequence_privilege_is_explicit() {
    assert!(matches!(
        quote_sequence_privilege("DELETE"),
        Err(DbError::Unsupported(_))
    ));
}

#[test]
fn dollar_quote_password_avoids_single_quoted_literal() {
    let quoted = dollar_quote_password("s3cret").expect("encode");
    assert!(quoted.starts_with("$pwd"));
    assert!(quoted.ends_with('$'));
    assert!(!quoted.contains("'s3cret'"));
    assert!(quoted.contains("s3cret"));
}
