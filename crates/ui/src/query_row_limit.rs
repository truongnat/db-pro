//! Row-cap rewriting for dispatched queries — backs the toolbar limit selector.
//!
//! The cap is a guardrail, not an override: a statement that already carries
//! its own `LIMIT`/`TOP`/`FETCH` clause keeps it, and anything that is not a
//! plain read is dispatched verbatim. Scripts with several statements pass
//! through untouched — splitting and re-joining would drop text between
//! statements.
use db_pro_core::domain::safety::{classify_statement_safety, split_statements, StatementSafety};

use crate::editor::{SqlDialect, SqlHighlighter, SyntaxToken, SyntaxTokenKind};

/// Apply the configured row cap to the SQL about to be dispatched. `None`
/// (or a script with several statements) returns the text untouched so the
/// history records exactly what was sent.
pub(super) fn apply_row_limit(sql: &str, limit: Option<u64>, driver: &str) -> String {
    let Some(limit) = limit.filter(|limit| *limit > 0) else {
        return sql.to_owned();
    };
    if split_statements(sql).len() != 1 {
        return sql.to_owned();
    }
    // Only plain reads get capped — SHOW/VALUES/classified-writes keep their text.
    if !matches!(classify_statement_safety(sql), Some(StatementSafety::Read)) {
        return sql.to_owned();
    }
    let dialect = if driver.eq_ignore_ascii_case("sqlite") {
        SqlDialect::SQLite
    } else {
        SqlDialect::Postgres
    };
    let tokens = SqlHighlighter::new(dialect).tokenize(sql);
    if !starts_with_select_or_with(sql, &tokens) {
        return sql.to_owned();
    }
    if has_row_cap_or_locking_tail(sql, &tokens) {
        return sql.to_owned();
    }
    if is_sqlserver(driver) {
        // TOP caps only the SELECT it prefixes — a set operation would cap one
        // branch, not the result, so those statements pass through.
        if has_set_operation(sql, &tokens) {
            return sql.to_owned();
        }
        inject_top(sql, limit, &tokens).unwrap_or_else(|| sql.to_owned())
    } else {
        format!("{} LIMIT {limit}", sql.trim().trim_end_matches(';').trim_end())
    }
}

fn is_sqlserver(driver: &str) -> bool {
    driver.eq_ignore_ascii_case("sql server") || driver.eq_ignore_ascii_case("sqlserver")
}

fn is_word_token(token: &SyntaxToken) -> bool {
    matches!(
        token.kind,
        SyntaxTokenKind::Keyword | SyntaxTokenKind::Identifier | SyntaxTokenKind::Type | SyntaxTokenKind::Function
    )
}

fn word_at<'a>(sql: &'a str, token: &SyntaxToken) -> &'a str {
    &sql[token.range.0..token.range.1]
}

/// First meaningful token must open a plain read: SELECT, WITH … SELECT or the
/// PostgreSQL `TABLE name` shorthand. This refuses SHOW and friends even though
/// the safety classifier labels them Read — they take no LIMIT clause.
fn starts_with_select_or_with(sql: &str, tokens: &[SyntaxToken]) -> bool {
    tokens
        .iter()
        .find(|token| is_word_token(token))
        .is_some_and(|token| {
            let word = word_at(sql, token);
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            word.eq_ignore_ascii_case("select") || word.eq_ignore_ascii_case("with") || word.eq_ignore_ascii_case("table")
        })
}

/// A depth-0 clause the cap would fight: an existing row cap, a locking tail
/// (`FOR UPDATE` — LIMIT must precede it, not follow), `SELECT … INTO`, or a
/// `RETURNING`/`OUTPUT` producer that already bounds its rows.
fn has_row_cap_or_locking_tail(sql: &str, tokens: &[SyntaxToken]) -> bool {
    const SKIP_WORDS: [&str; 7] = ["limit", "fetch", "top", "into", "for", "returning", "output"];
    for_each_top_level_word(sql, tokens, |word| SKIP_WORDS.contains(&word))
}

fn has_set_operation(sql: &str, tokens: &[SyntaxToken]) -> bool {
    for_each_top_level_word(sql, tokens, |word| {
        word == "union" || word == "intersect" || word == "except"
    })
}

fn for_each_top_level_word(sql: &str, tokens: &[SyntaxToken], matches: impl Fn(&str) -> bool) -> bool {
    let mut depth = 0usize;
    for token in tokens {
        let text = word_at(sql, token);
        match token.kind {
            SyntaxTokenKind::Punctuation if text == "(" => depth += 1,
            SyntaxTokenKind::Punctuation if text == ")" => depth = depth.saturating_sub(1),
            _ if depth == 0 && is_word_token(token) && matches(&text.to_ascii_lowercase()) => return true,
            _ => {}
        }
    }
    false
}

/// Insert `TOP n` after the first depth-0 SELECT (skipping ALL/DISTINCT, which
/// SQL Server requires before TOP). WITH … SELECT finds the outer SELECT
/// because CTE bodies sit inside parens.
// cc-scan:allow COMPLEXITY,DEEP_NESTING — classifier/dispatch ladder — one case per branch
fn inject_top(sql: &str, limit: u64, tokens: &[SyntaxToken]) -> Option<String> {
    let mut depth = 0usize;
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        let text = word_at(sql, token);
        match token.kind {
            SyntaxTokenKind::Punctuation if text == "(" => depth += 1,
            SyntaxTokenKind::Punctuation if text == ")" => depth = depth.saturating_sub(1),
            _ if depth == 0 && text.eq_ignore_ascii_case("select") => {
                let mut insert = token.range.1;
                while let Some(next) = iter.peek() {
                    if !is_word_token(next) {
                        if matches!(next.kind, SyntaxTokenKind::Whitespace | SyntaxTokenKind::Comment) {
                            iter.next();
                            continue;
                        }
                        break;
                    }
                    let word = word_at(sql, next);
                    if !(word.eq_ignore_ascii_case("all") || word.eq_ignore_ascii_case("distinct")) {
                        break;
                    }
                    insert = next.range.1;
                    iter.next();
                }
                return Some(format!("{} TOP {limit}{}", &sql[..insert], &sql[insert..]));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::apply_row_limit;

    #[test]
    fn read_statement_gets_limit() {
        assert_eq!(
            apply_row_limit("SELECT * FROM users", Some(500), "PostgreSQL"),
            "SELECT * FROM users LIMIT 500"
        );
        assert_eq!(
            apply_row_limit("SELECT * FROM users;", Some(100), "SQLite"),
            "SELECT * FROM users LIMIT 100"
        );
    }

    #[test]
    fn explicit_cap_wins() {
        let sql = "SELECT * FROM users LIMIT 10;";
        assert_eq!(apply_row_limit(sql, Some(500), "PostgreSQL"), sql);
        let fetch = "SELECT * FROM users FETCH FIRST 10 ROWS ONLY";
        assert_eq!(apply_row_limit(fetch, Some(500), "PostgreSQL"), fetch);
    }

    #[test]
    fn limit_inside_a_string_does_not_block_the_cap() {
        assert_eq!(
            apply_row_limit("SELECT 'no limit here' FROM t", Some(50), "PostgreSQL"),
            "SELECT 'no limit here' FROM t LIMIT 50"
        );
    }

    #[test]
    fn writes_and_ddl_pass_through() {
        for sql in ["DELETE FROM users WHERE id = 1", "UPDATE t SET x = 1", "CREATE TABLE t (a int)"] {
            assert_eq!(apply_row_limit(sql, Some(500), "PostgreSQL"), sql);
        }
    }

    #[test]
    fn locking_and_into_tails_pass_through() {
        for sql in ["SELECT * FROM t FOR UPDATE", "SELECT * INTO backup FROM t"] {
            assert_eq!(apply_row_limit(sql, Some(500), "PostgreSQL"), sql);
        }
    }

    #[test]
    fn union_still_caps_on_limit_dialects() {
        assert_eq!(
            apply_row_limit("SELECT a FROM t UNION SELECT a FROM u", Some(500), "PostgreSQL"),
            "SELECT a FROM t UNION SELECT a FROM u LIMIT 500"
        );
    }

    #[test]
    fn multi_statement_scripts_pass_through() {
        let sql = "SELECT 1; SELECT 2;";
        assert_eq!(apply_row_limit(sql, Some(500), "PostgreSQL"), sql);
    }

    #[test]
    fn no_limit_and_zero_pass_through() {
        let sql = "SELECT * FROM t";
        assert_eq!(apply_row_limit(sql, None, "PostgreSQL"), sql);
        assert_eq!(apply_row_limit(sql, Some(0), "PostgreSQL"), sql);
    }

    #[test]
    fn sqlserver_gets_top_after_select() {
        assert_eq!(
            apply_row_limit("SELECT u.id FROM users u ORDER BY u.id", Some(500), "SQL Server"),
            "SELECT TOP 500 u.id FROM users u ORDER BY u.id"
        );
        assert_eq!(
            apply_row_limit("SELECT DISTINCT role FROM users", Some(100), "sqlserver"),
            "SELECT DISTINCT TOP 100 role FROM users"
        );
        // TOP on one UNION branch would cap the wrong thing — leave it alone.
        let union = "SELECT a FROM t UNION SELECT a FROM u";
        assert_eq!(apply_row_limit(union, Some(500), "SQL Server"), union);
    }

    #[test]
    fn with_cte_caps_the_outer_read() {
        assert_eq!(
            apply_row_limit("WITH x AS (SELECT 1) SELECT * FROM x", Some(500), "PostgreSQL"),
            "WITH x AS (SELECT 1) SELECT * FROM x LIMIT 500"
        );
        assert_eq!(
            apply_row_limit("WITH x AS (SELECT 1) SELECT * FROM x", Some(500), "SQL Server"),
            "WITH x AS (SELECT 1) SELECT TOP 500 * FROM x"
        );
    }
}
