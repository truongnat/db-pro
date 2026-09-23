use safety_cte::classify_cte_safety;
use safety_lexer::{
    is_identifier_continue, is_identifier_start, matching_parenthesis_end, scan_identifier, skip_block_comment,
    skip_dollar_quote, skip_line_comment, skip_quoted, special_token_end, token_is_word, tokenize_sql, SqlTokenKind,
};
pub use safety_policy::{validate_against_policy, ConnectionSafetyPolicy};

#[path = "safety_cte.rs"]
mod safety_cte;
#[path = "safety_lexer.rs"]
mod safety_lexer;

#[path = "safety_policy.rs"]
mod safety_policy;

/// Classification of a SQL statement for safety enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementSafety {
    /// SELECT, SHOW, EXPLAIN, WITH ... SELECT
    Read,
    /// INSERT, UPDATE, DELETE (with WHERE)
    Write,
    /// CREATE, ALTER
    Ddl,
    /// DROP, TRUNCATE, DELETE without WHERE
    Destructive,
}

impl StatementSafety {
    /// Severity of this classification relative to the others, ordered by how
    /// hard the effect is to undo: read < write < DDL < destructive.
    ///
    /// A script is reduced to its most dangerous statement with this ranking,
    /// so no statement can hide behind a harmless one.
    pub fn severity_rank(self) -> u8 {
        match self {
            Self::Read => 0,
            Self::Write => 1,
            Self::Ddl => 2,
            Self::Destructive => 3,
        }
    }
}

/// Classify a SQL statement for safety enforcement.
/// This is a best-effort heuristic, not a full SQL parser.
pub fn classify_statement_safety(sql: &str) -> Option<StatementSafety> {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip leading comments
    let trimmed = strip_leading_comments(trimmed);
    let upper = trimmed.to_ascii_uppercase();
    let keyword = upper.split_whitespace().next()?;

    match keyword {
        "SELECT" | "SHOW" | "TABLE" => Some(StatementSafety::Read),
        "EXPLAIN" => classify_explain_safety(trimmed),
        "WITH" => classify_cte_safety(trimmed),
        "INSERT" | "UPDATE" => Some(StatementSafety::Write),
        "DELETE" => {
            if is_delete_without_where(trimmed) {
                Some(StatementSafety::Destructive)
            } else {
                Some(StatementSafety::Write)
            }
        }
        "CREATE" | "ALTER" => Some(StatementSafety::Ddl),
        "DROP" => Some(StatementSafety::Destructive),
        "TRUNCATE" => Some(StatementSafety::Destructive),
        // These statements can execute server-side or dynamically prepared
        // mutations that the client-side classifier cannot inspect safely.
        "DO" | "CALL" | "EXECUTE" => Some(StatementSafety::Destructive),
        "MERGE" => classify_merge_safety(trimmed),
        _ => Some(StatementSafety::Write),
    }
}

/// Classify a SQL script that may hold several statements by its **most
/// dangerous** statement.
///
/// A script must never be classified as read-only merely because it starts with
/// `SELECT`: the decision that can auto-execute a script has to see every
/// statement in it (#147, #129). Returns `None` only when the script holds no
/// statement at all.
pub fn classify_script_safety(sql: &str) -> Option<StatementSafety> {
    split_statements(sql)
        .into_iter()
        .filter_map(|statement| classify_statement_safety(&statement))
        .max_by_key(|safety| safety.severity_rank())
}

/// The transaction-control verb a statement starts with, if it is one.
///
/// `BEGIN`/`START TRANSACTION`, `COMMIT`/`END`, `ROLLBACK`/`ABORT`, `SAVEPOINT` and `RELEASE`
/// take control of a transaction explicitly. A multi-statement batch that contains any mutation is
/// executed inside a transaction of its own (`Connector::execute_transaction`, `#129`), so a
/// statement like this one does not join that transaction — it ends or re-scopes it. The statements
/// after it run outside the wrapper, and the wrapper's rollback then has nothing left to undo while
/// the failure envelope still reports `RolledBack` (#147).
///
/// Only the leading keyword is inspected: a `BEGIN` inside `DO $body$ … $body$` belongs to that
/// statement's own body and is classified by its leading `DO`, and quoted text is not a keyword.
pub fn transaction_control_verb(sql: &str) -> Option<&'static str> {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    let trimmed = strip_leading_comments(trimmed);
    if trimmed.is_empty() {
        return None;
    }

    let upper = trimmed.to_ascii_uppercase();
    let mut words = upper.split_whitespace();
    match words.next()? {
        "BEGIN" => Some("BEGIN"),
        "COMMIT" => Some("COMMIT"),
        "END" => Some("END"),
        "ROLLBACK" => Some("ROLLBACK"),
        "ABORT" => Some("ABORT"),
        "SAVEPOINT" => Some("SAVEPOINT"),
        "RELEASE" => Some("RELEASE"),
        "START" if words.next() == Some("TRANSACTION") => Some("START TRANSACTION"),
        _ => None,
    }
}

/// Split a SQL script into its statements on statement-terminating semicolons.
///
/// Semicolons inside quoted strings, quoted identifiers, comments and
/// dollar-quoted bodies do not terminate a statement, and comment-only
/// fragments are dropped rather than returned as empty statements.
pub fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let bytes = sql.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(end) = special_token_end(bytes, index) {
            current.push_str(&sql[index..end]);
            index = end;
        } else if bytes[index] == b';' {
            push_statement(&mut statements, &current);
            current.clear();
            index += 1;
        } else {
            let Some(character) = sql[index..].chars().next() else {
                break;
            };
            current.push(character);
            index += character.len_utf8();
        }
    }

    push_statement(&mut statements, &current);
    statements
}

fn push_statement(statements: &mut Vec<String>, current: &str) {
    let statement = current.trim();
    if !strip_leading_comments(statement).trim().is_empty() {
        statements.push(statement.to_owned());
    }
}

fn classify_merge_safety(sql: &str) -> Option<StatementSafety> {
    if contains_sql_keyword(sql, "DELETE") {
        Some(StatementSafety::Destructive)
    } else {
        Some(StatementSafety::Write)
    }
}

/// For EXPLAIN statements: plain EXPLAIN is Read, but EXPLAIN ANALYZE actually
/// executes the inner statement, so its safety depends on the inner statement.
fn classify_explain_safety(sql: &str) -> Option<StatementSafety> {
    let mut remainder = strip_leading_comments(consume_keyword(sql, "EXPLAIN")?.1).trim_start();

    // PostgreSQL's parenthesized EXPLAIN options are metadata, not the query
    // being explained. Only ANALYZE causes the inner statement to execute.
    if remainder.starts_with('(') {
        let end = matching_parenthesis_end(remainder)?;
        let options = &remainder[1..end - 1];
        if !contains_sql_keyword(options, "ANALYZE") && !contains_sql_keyword(options, "ANALYSE") {
            return Some(StatementSafety::Read);
        }
        remainder = remainder[end..].trim_start();
        return classify_statement_safety(remainder);
    }

    // The legacy spelling is EXPLAIN [ANALYZE] [VERBOSE] statement. Match
    // whole keywords in the original SQL so literals/identifiers cannot alter
    // the decision and Unicode case conversion cannot invalidate byte offsets.
    let Some((_, after_analyze)) =
        consume_keyword(remainder, "ANALYZE").or_else(|| consume_keyword(remainder, "ANALYSE"))
    else {
        return Some(StatementSafety::Read);
    };

    let mut inner_sql = strip_leading_comments(after_analyze).trim_start();
    if let Some((_, after_verbose)) = consume_keyword(inner_sql, "VERBOSE") {
        inner_sql = strip_leading_comments(after_verbose).trim_start();
    }

    classify_statement_safety(inner_sql)
}

fn consume_keyword<'a>(sql: &'a str, expected: &str) -> Option<(&'a str, &'a str)> {
    let value = strip_leading_comments(sql).trim_start();
    let bytes = value.as_bytes();
    let first = *bytes.first()?;
    if !is_identifier_start(first) {
        return None;
    }

    let mut end = 1;
    while end < bytes.len() && is_identifier_continue(bytes[end]) {
        end += 1;
    }
    let keyword = &value[..end];
    if keyword.eq_ignore_ascii_case(expected) {
        Some((keyword, &value[end..]))
    } else {
        None
    }
}

/// Check whether a DELETE statement lacks a WHERE clause.
fn is_delete_without_where(sql: &str) -> bool {
    !contains_sql_keyword(sql, "WHERE")
}

/// Find a keyword while ignoring quoted strings, quoted identifiers, and SQL
/// comments. This is intentionally not a full SQL parser; it prevents a
/// keyword-like substring from changing the destructive-operation decision.
fn contains_sql_keyword(sql: &str, expected: &str) -> bool {
    let bytes = sql.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        index = match bytes[index] {
            b'\'' => skip_quoted(bytes, index, b'\''),
            b'"' => skip_quoted(bytes, index, b'"'),
            b'-' if bytes.get(index + 1) == Some(&b'-') => skip_line_comment(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => skip_block_comment(bytes, index),
            b'$' => skip_dollar_quote(bytes, index).unwrap_or(index + 1),
            current if is_identifier_start(current) => {
                let end = scan_identifier(bytes, index);
                if bytes[index..end].eq_ignore_ascii_case(expected.as_bytes()) {
                    return true;
                }
                end
            }
            _ => index + 1,
        }
    }

    false
}

pub(crate) fn has_top_level_sql_keyword(sql: &str, expected: &str) -> bool {
    let mut depth = 0usize;
    for token in tokenize_sql(sql) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
            SqlTokenKind::Word if depth == 0 && token_is_word(sql, token, expected) => return true,
            _ => {}
        }
    }
    false
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if s.starts_with("--") {
            s = s.find('\n').map(|i| &s[i + 1..]).unwrap_or("").trim_start();
        } else if s.starts_with("/*") {
            s = s.find("*/").map(|i| &s[i + 2..]).unwrap_or("").trim_start();
        } else {
            break;
        }
    }
    s
}

#[cfg(test)]
#[path = "safety/tests.rs"]
mod tests;
