//! Tests for `../diagnostics.rs` (moved out of the source file).
use super::*;

#[test]
fn syntax_diagnostic_covers_only_the_statement_around_the_cursor() {
    let sql = "SELECT 1; SELECT 'a;b' FROM t; SELECT FROM; SELECT 2;";
    let bad_at = sql.find("SELECT FROM").expect("bad statement");
    let diagnostic = statement_syntax_diagnostic(sql, bad_at + 4, SqlDialect::Postgres).expect("syntax error");
    assert_eq!(diagnostic.code.as_deref(), Some("syntax"));
    assert!(diagnostic.range.0 >= bad_at);
    assert!(diagnostic.range.1 <= bad_at + "SELECT FROM;".len());
    assert!(statement_syntax_diagnostic(sql, 0, SqlDialect::Postgres).is_none());

    let mut diagnostics = vec![Diagnostic::error((0, 1), "old")];
    diagnostics[0].code = Some("syntax".to_owned());
    sync_statement_syntax_diagnostic(&mut diagnostics, "SELECT 1;", 0, SqlDialect::Postgres);
    assert!(diagnostics.is_empty());
}
