//! Tests for `../replication.rs` (moved out of the source file).
use super::*;

#[test]
fn redacts_password_in_conninfo() {
    let raw = "host=db user=u password=s3cret dbname=app";
    let red = redact_conninfo(raw);
    assert!(red.contains("password=[redacted]"));
    assert!(!red.contains("s3cret"));
}

#[test]
fn preview_publication_sql() {
    assert_eq!(
        preview_create_publication_all("pub1").unwrap(),
        "CREATE PUBLICATION \"pub1\" FOR ALL TABLES;"
    );
}

#[test]
fn preview_drop_sql() {
    assert_eq!(
        preview_drop_publication("pub1").unwrap(),
        "DROP PUBLICATION IF EXISTS \"pub1\";"
    );
    assert_eq!(
        preview_drop_subscription("sub1").unwrap(),
        "DROP SUBSCRIPTION IF EXISTS \"sub1\";"
    );
}

#[test]
fn rejects_unsafe_ident() {
    assert!(validate_repl_ident("bad-name").is_err());
    assert!(quote_ident(";drop").is_err());
}
