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
fn redacts_quoted_password_with_spaces() {
    let raw = "host=db password='top secret' dbname=app";
    let red = redact_conninfo(raw);
    assert_eq!(red, "host=db password=[redacted] dbname=app");
    assert!(!red.contains("secret"));
}

#[test]
fn redacts_quoted_password_with_escaped_quote() {
    let raw = "host=db password='it\\'s ok' dbname=app";
    let red = redact_conninfo(raw);
    assert_eq!(red, "host=db password=[redacted] dbname=app");
    assert!(!red.contains("ok"));
}

#[test]
fn redacts_quoted_passfile_with_spaces() {
    let raw = "host=db passfile='/home/u/my keys/pgpass' dbname=app";
    let red = redact_conninfo(raw);
    assert_eq!(red, "host=db passfile=[redacted] dbname=app");
    assert!(!red.contains("pgpass"));
}

#[test]
fn redacts_every_occurrence_of_each_key() {
    let raw = "password=one host=db PASSWORD=two sslkey=path";
    let red = redact_conninfo(raw);
    assert_eq!(red.matches("[redacted]").count(), 3);
    assert!(!red.contains("one"));
    assert!(!red.contains("two"));
    assert!(!red.contains("sslkey=path"));
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
