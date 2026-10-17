//! Tests for `../syntax.rs` (moved out of the source file).
use super::*;

#[test]
fn test_cached_tokens_invalidation() {
    let mut buf = TextBuffer::from_string("SELECT 1;");
    let mut cache = CachedSqlTokens::new();

    let tokens1 = cache.get_or_recompute(&buf, SqlDialect::Postgres);
    assert_eq!(tokens1.len(), 4); // SELECT, ' ', 1, ';'

    // Unmodified buffer returns cached tokens
    let len_cached = cache.tokens().len();
    assert_eq!(len_cached, 4);

    // Edit buffer
    buf.insert(9, " SELECT 2;");
    let tokens2 = cache.get_or_recompute(&buf, SqlDialect::Postgres);
    assert_eq!(tokens2.len(), 9);
}

#[test]
fn test_is_in_string_or_comment() {
    let buf = TextBuffer::from_string("SELECT 'hello world', -- comment\n1;");
    let mut cache = CachedSqlTokens::new();
    cache.get_or_recompute(&buf, SqlDialect::Postgres);

    assert!(cache.is_in_string_or_comment(10)); // inside 'hello world'
    assert!(!cache.is_in_string_or_comment(2)); // inside SELECT
    assert!(cache.is_in_string_or_comment(26)); // inside comment
}

fn large_sql() -> String {
    let mut text = String::from("SELECT id FROM orders WHERE status = 'open';\n");
    for index in 0..180 {
        text.push_str(&format!("SELECT col_{index} FROM table_{index} WHERE id = {index};\n"));
    }
    text
}

fn assert_tokens_match(actual: &[SyntaxToken], expected: &[SyntaxToken]) {
    assert_eq!(actual.len(), expected.len(), "token count");
    for (index, (left, right)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(left, right, "token {index}");
    }
}

#[test]
fn incremental_token_splice_matches_full_tokenize_outside_a_string() {
    let text = large_sql();
    let needle = "FROM table_90";
    let insert_at = text.find(needle).expect("needle") + 2;
    let mut buf = TextBuffer::from_string(&text);
    let mut cache = CachedSqlTokens::new();
    cache.get_or_recompute(&buf, SqlDialect::Postgres);
    buf.insert(insert_at, "X");

    let spliced = cache.get_or_recompute(&buf, SqlDialect::Postgres).to_vec();
    assert!(
        cache.last_incremental_from.is_some_and(|from| from > 0),
        "edit outside a string must not retokenize from byte 0"
    );
    let full = SqlHighlighter::new(SqlDialect::Postgres).tokenize(buf.text());
    assert_tokens_match(&spliced, &full);
    let string_at = buf.text().find("'open'").expect("string") + 2;
    assert!(cache.is_in_string_or_comment(string_at));
    assert!(!cache.is_in_string_or_comment(insert_at));
}

#[test]
fn incremental_token_multiline_comment_falls_back_to_full_tokenize() {
    let mut commented = String::from("/*\n");
    commented.push_str(&large_sql());
    commented.push_str("*/\nSELECT ready;\n");
    let insert_at = commented.find("FROM table_90").expect("needle") + 2;
    let mut buf = TextBuffer::from_string(&commented);
    let mut cache = CachedSqlTokens::new();
    cache.get_or_recompute(&buf, SqlDialect::Postgres);
    buf.insert(insert_at, "X");

    let spliced = cache.get_or_recompute(&buf, SqlDialect::Postgres).to_vec();
    assert!(cache.last_incremental_from.is_none());
    let full = SqlHighlighter::new(SqlDialect::Postgres).tokenize(buf.text());
    assert_tokens_match(&spliced, &full);
}

#[test]
fn postgres_block_comment_nests() {
    // `/* a /* b */ c */` is ONE comment under Postgres; the inner `*/` only
    // drops the nested level (#474).
    let text = "/* a /* b */ c */ SELECT 1;";
    let tokens = SqlHighlighter::new(SqlDialect::Postgres).tokenize(text);
    assert_eq!(tokens[0].kind, SyntaxTokenKind::Comment);
    assert_eq!(tokens[0].range, (0, 17));
    let select = tokens.iter().find(|t| t.kind == SyntaxTokenKind::Keyword).expect("SELECT token");
    assert_eq!(&text[select.range.0..select.range.1], "SELECT");
}

#[test]
fn generic_dialect_block_comment_stops_at_first_close() {
    let text = "/* a /* b */ c */";
    let tokens = SqlHighlighter::new(SqlDialect::Generic).tokenize(text);
    assert_eq!(tokens[0].kind, SyntaxTokenKind::Comment);
    assert_eq!(tokens[0].range, (0, 12), "non-nesting dialects still end at the first */");
}

#[test]
fn unterminated_nested_comment_consumes_to_eof() {
    let tokens = SqlHighlighter::new(SqlDialect::Postgres).tokenize("/* a /* b */ c");
    assert_eq!(tokens[0].kind, SyntaxTokenKind::Comment);
    assert_eq!(tokens[0].range, (0, 14));
}
