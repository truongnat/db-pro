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
#[path = "tests/query_row_limit_tests.rs"]
mod tests;

