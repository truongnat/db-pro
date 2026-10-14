//! Tests for `../sql_policy.rs` (moved out of the source file).
use super::*;

#[test]
fn reject_multi_statement_basic() {
    assert!(reject_multi_statement("SELECT 1").is_ok());
    assert!(reject_multi_statement("SELECT 1; SELECT 2").is_err());
    assert!(reject_multi_statement("  ").is_err());
    assert!(reject_multi_statement("SELECT 1;").is_ok());
}

#[test]
fn reject_multi_statement_ignores_literals_identifiers_and_comments() {
    assert!(reject_multi_statement("SELECT ';'").is_ok());
    assert!(reject_multi_statement("SELECT \"semi;colon\"").is_ok());
    assert!(reject_multi_statement("SELECT 1 -- ;\n").is_ok());
    assert!(reject_multi_statement("SELECT 1 /* ; */").is_ok());
    assert!(reject_multi_statement("DO $body$ BEGIN PERFORM 1; END $body$").is_ok());
}
