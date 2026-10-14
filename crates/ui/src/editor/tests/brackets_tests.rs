//! Tests for `../brackets.rs` (moved out of the source file).
use super::*;

#[test]
fn matches_nested_parentheses_and_brackets() {
    let sql = "SELECT fn(items[1], (value))";
    let opening = sql.find('(').expect("opening parenthesis");
    let matching = matching_delimiter(sql, opening).expect("matching parenthesis");
    assert_eq!(&sql[matching..matching + 1], ")");
    let bracket = sql.find('[').expect("opening bracket");
    assert_eq!(matching_delimiter(sql, bracket), sql.find(']'));
}

#[test]
fn ignores_literals_comments_and_dollar_quotes() {
    let sql = "SELECT '(', $$ [not a bracket] $$, /* ( */ (value)";
    let opening = sql.rfind('(').expect("structural opening");
    assert_eq!(matching_delimiter(sql, opening), sql.rfind(')'));
    assert!(matching_delimiter(sql, sql.find('(').expect("literal opening")).is_none());
}

#[test]
fn cursor_matching_prefers_delimiter_before_caret() {
    let sql = "SELECT (value)";
    let cursor = sql.find(')').expect("closing delimiter");
    let result = delimiter_near_cursor(sql, cursor).expect("delimiter near cursor");
    assert_eq!(result.delimiter, cursor);
    assert_eq!(result.matching, sql.find('('));
}

#[test]
fn reports_mixed_delimiter_mismatch() {
    let sql = "SELECT ([)]";
    let mismatch = structural_delimiter_issues(sql)
        .into_iter()
        .find(|issue| issue.character == ')')
        .expect("mismatch diagnostic");
    assert_eq!(mismatch.expected, Some(']'));
    assert!(matching_delimiter(sql, sql.find('(').expect("opening parenthesis")).is_none());
}

#[test]
fn handles_escaped_e_strings_and_quoted_identifiers() {
    let sql = r#"SELECT E'it\'s (safe)', "name" FROM users"#;
    let string_start = sql.find('\'').expect("string quote");
    let string_end = sql.rfind('\'').expect("string end quote");
    assert_eq!(matching_delimiter(sql, string_start), Some(string_end));
    let identifier_start = sql.find('"').expect("identifier quote");
    let identifier_end = sql.rfind('"').expect("identifier end quote");
    assert_eq!(matching_delimiter(sql, identifier_start), Some(identifier_end));
}

#[test]
fn detects_the_range_to_delete_for_an_auto_pair() {
    let sql = "SELECT ()";
    let cursor = sql.find(')').expect("caret between pair");
    assert_eq!(auto_pair_range(sql, cursor), Some((7, 9)));
}
