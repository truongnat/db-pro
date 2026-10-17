//! Tests for `../sql_policy.rs` (moved out of the source file).
use super::*;

#[test]
fn reject_multi_statement_basic() {
    assert!(reject_multi_statement("SELECT 1", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("SELECT 1; SELECT 2", DriverType::Postgres).is_err());
    assert!(reject_multi_statement("  ", DriverType::Postgres).is_err());
    assert!(reject_multi_statement("SELECT 1;", DriverType::Postgres).is_ok());
}

#[test]
fn reject_multi_statement_ignores_literals_identifiers_and_comments() {
    assert!(reject_multi_statement("SELECT ';'", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("SELECT \"semi;colon\"", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("SELECT 1 -- ;\n", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("SELECT 1 /* ; */", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("DO $body$ BEGIN PERFORM 1; END $body$", DriverType::Postgres).is_ok());
}

// #477: with `standard_conforming_strings=on` (the PostgreSQL default) a
// trailing `\` inside `'…'` is plain text — `'x\'` closes at the quote, so the
// tail is a second, real statement that must be rejected.
#[test]
fn reject_multi_statement_catches_pg_backslash_before_quote() {
    assert!(reject_multi_statement("SELECT 'x\\'; DROP TABLE important; --", DriverType::Postgres).is_err());
    assert!(reject_multi_statement("SELECT 'x\\'; SELECT 2", DriverType::Postgres).is_err());
    // An explicit `E'…'` literal still escapes `\'` on PostgreSQL.
    assert!(reject_multi_statement("SELECT E'x\\''", DriverType::Postgres).is_ok());
    assert!(reject_multi_statement("SELECT E'x\\''; SELECT 2", DriverType::Postgres).is_err());
}

// MySQL's default sql_mode keeps `\'` as an escape, so the same text is a
// single statement there.
#[test]
fn reject_multi_statement_keeps_mysql_backslash_escape() {
    assert!(reject_multi_statement("SELECT 'x\\'; SELECT 2", DriverType::Mysql).is_ok());
    assert!(reject_multi_statement("SELECT 'it\\'s ok'", DriverType::Mysql).is_ok());
}
