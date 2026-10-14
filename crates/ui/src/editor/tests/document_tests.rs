//! Tests for `../document.rs` (moved out of the source file).
use super::*;

#[test]
fn test_statement_analysis_with_semicolon_inside_string() {
    let sql = "SELECT 'hello; world' AS greeting;\nSELECT id FROM users WHERE status = 'active';";
    let buf = TextBuffer::from_string(sql);
    let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
    assert_eq!(analysis.statements.len(), 2);
    assert_eq!(analysis.statements[0].text, "SELECT 'hello; world' AS greeting;");
    assert_eq!(
        analysis.statements[1].text,
        "SELECT id FROM users WHERE status = 'active';"
    );

    let stmt = analysis.current_statement_at(10).unwrap();
    assert_eq!(stmt.text, "SELECT 'hello; world' AS greeting;");

    let stmt2 = analysis.current_statement_at(40).unwrap();
    assert_eq!(stmt2.text, "SELECT id FROM users WHERE status = 'active';");
}

#[test]
fn caret_in_the_gap_after_a_semicolon_belongs_to_that_statement() {
    let sql = "SELECT 1;\n\nSELECT 2;";
    let buf = TextBuffer::from_string(sql);
    let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);

    let gap = analysis.statement_for_caret(10).unwrap();
    assert_eq!(gap.text, "SELECT 1;");

    let before = analysis.statement_for_caret(0).unwrap();
    assert_eq!(before.text, "SELECT 1;");

    let second = analysis.statement_for_caret(sql.find("SELECT 2").unwrap()).unwrap();
    assert_eq!(second.text, "SELECT 2;");

    let after_last = analysis.statement_for_caret(sql.len()).unwrap();
    assert_eq!(after_last.text, "SELECT 2;");
}

#[test]
fn caret_before_the_first_statement_belongs_to_it() {
    let sql = "\n\nSELECT 1;";
    let buf = TextBuffer::from_string(sql);
    let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
    let statement = analysis.statement_for_caret(0).unwrap();
    assert_eq!(statement.text, "SELECT 1;");
}

#[test]
fn test_statement_analysis_with_dollar_quote() {
    let sql = "CREATE OR REPLACE FUNCTION test() RETURNS void AS $$ BEGIN SELECT 1; END; $$ LANGUAGE plpgsql;";
    let buf = TextBuffer::from_string(sql);
    let analysis = SqlDocumentAnalysis::analyze(&buf, SqlDialect::Postgres);
    assert_eq!(analysis.statements.len(), 1);
    assert_eq!(analysis.statements[0].text, sql);
}
