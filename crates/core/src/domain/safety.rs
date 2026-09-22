use serde::{Deserialize, Serialize};

use safety_cte::classify_cte_safety;
use safety_lexer::{
    is_identifier_continue, is_identifier_start, matching_parenthesis_end, scan_identifier, skip_block_comment,
    skip_dollar_quote, skip_line_comment, skip_quoted, special_token_end, token_is_word, tokenize_sql, SqlTokenKind,
};

#[path = "safety_cte.rs"]
mod safety_cte;
#[path = "safety_lexer.rs"]
mod safety_lexer;

/// Safety policy enforced at the backend/application layer for a connection.
///
/// This is NOT a frontend-only toggle. The backend MUST reject operations that
/// violate the policy, even if the frontend sends them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSafetyPolicy {
    /// If true, only SELECT / read queries are allowed.
    pub read_only: bool,
    /// If true, DDL operations (CREATE, ALTER, DROP) are allowed.
    pub allow_ddl: bool,
    /// If true, destructive operations (DROP, TRUNCATE, DELETE without WHERE) are allowed.
    pub allow_destructive: bool,
    /// Maximum number of rows a query can return. None means use connection default.
    pub max_rows: Option<u64>,
    /// Query timeout in milliseconds. None means use connection default.
    pub query_timeout_ms: Option<u64>,
}

impl ConnectionSafetyPolicy {
    /// Default policy: full access, no restrictions beyond connection defaults.
    pub fn full_access() -> Self {
        Self {
            read_only: false,
            allow_ddl: true,
            allow_destructive: true,
            max_rows: None,
            query_timeout_ms: None,
        }
    }

    /// Read-only policy: no writes, no DDL, no destructive operations.
    pub fn read_only() -> Self {
        Self {
            read_only: true,
            allow_ddl: false,
            allow_destructive: false,
            max_rows: None,
            query_timeout_ms: None,
        }
    }
}

impl Default for ConnectionSafetyPolicy {
    fn default() -> Self {
        Self::full_access()
    }
}

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

/// Validate a SQL statement against a safety policy.
/// Returns `Ok(())` if the statement is allowed, or an error message.
pub fn validate_against_policy(sql: &str, policy: &ConnectionSafetyPolicy) -> Result<(), String> {
    let safety = match classify_statement_safety(sql) {
        Some(s) => s,
        None => return Err("empty SQL statement".into()),
    };

    if policy.read_only && safety != StatementSafety::Read {
        return Err(format!(
            "connection is read-only; cannot execute {:?} operation",
            safety
        ));
    }

    if !policy.allow_ddl && safety == StatementSafety::Ddl {
        return Err("DDL operations are not allowed on this connection".into());
    }

    if !policy.allow_destructive && safety == StatementSafety::Destructive {
        return Err("destructive operations are not allowed on this connection".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Transaction-control keywords fall to the classifier's fail-safe `Write` default,
    /// so a script that manages its own transaction is never treated as read-only. This
    /// pins the contract the multi-statement execution path depends on: `BEGIN`/`COMMIT`/
    /// `ROLLBACK` are not mutations, but they must not hide one either.
    #[test]
    fn transaction_control_batches_classify_as_write_and_never_hide_a_mutation() {
        assert_eq!(
            classify_script_safety("BEGIN; UPDATE users SET name = 'x' WHERE id = 1; COMMIT;"),
            Some(StatementSafety::Write)
        );
        assert_eq!(
            classify_script_safety("BEGIN; SELECT 1; ROLLBACK;"),
            Some(StatementSafety::Write)
        );
        assert_eq!(
            classify_script_safety("SELECT 1; SELECT 2;"),
            Some(StatementSafety::Read)
        );
        // A destructive statement inside its own transaction is still destructive, so the
        // run-all confirmation gate sees it.
        assert_eq!(
            classify_script_safety("BEGIN; DROP TABLE users; COMMIT;"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn classify_select_is_read() {
        assert_eq!(classify_statement_safety("SELECT 1"), Some(StatementSafety::Read));
    }

    #[test]
    fn classify_insert_is_write() {
        assert_eq!(
            classify_statement_safety("INSERT INTO t VALUES (1)"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn classify_update_is_write() {
        assert_eq!(
            classify_statement_safety("UPDATE t SET x = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn classify_delete_with_where_is_write() {
        assert_eq!(
            classify_statement_safety("DELETE FROM t WHERE id = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn classify_delete_without_where_is_destructive() {
        assert_eq!(
            classify_statement_safety("DELETE FROM t"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn delete_identifier_containing_where_is_still_destructive() {
        assert_eq!(
            classify_statement_safety(r#"DELETE FROM \"somewhere\""#),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_statement_safety("DELETE FROM t -- WHERE id = 1"),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_statement_safety("DELETE FROM t RETURNING $tag$ WHERE $tag$"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn classify_drop_is_destructive() {
        assert_eq!(
            classify_statement_safety("DROP TABLE t"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn classify_truncate_is_destructive() {
        assert_eq!(
            classify_statement_safety("TRUNCATE TABLE t"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn classify_merge_delete_as_destructive_but_update_as_write() {
        assert_eq!(
            classify_statement_safety(
                "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN DELETE"
            ),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_statement_safety(
                "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
            ),
            Some(StatementSafety::Write)
        );
        assert_eq!(
            classify_statement_safety(
                "MERGE INTO target USING (SELECT 'DELETE' AS action) source ON target.id = 1 WHEN MATCHED THEN UPDATE SET value = 1"
            ),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn classify_opaque_server_side_execution_as_destructive() {
        for sql in [
            "DO $body$ BEGIN DELETE FROM users; END $body$",
            "CALL cleanup_expired_rows()",
            "EXECUTE prepared_cleanup",
        ] {
            assert_eq!(classify_statement_safety(sql), Some(StatementSafety::Destructive));
            assert!(validate_against_policy(
                sql,
                &ConnectionSafetyPolicy {
                    read_only: false,
                    allow_ddl: true,
                    allow_destructive: false,
                    max_rows: None,
                    query_timeout_ms: None,
                }
            )
            .is_err());
        }
    }

    #[test]
    fn classify_create_is_ddl() {
        assert_eq!(
            classify_statement_safety("CREATE TABLE t (id INT)"),
            Some(StatementSafety::Ddl)
        );
    }

    #[test]
    fn classify_alter_is_ddl() {
        assert_eq!(
            classify_statement_safety("ALTER TABLE t ADD COLUMN x INT"),
            Some(StatementSafety::Ddl)
        );
    }

    #[test]
    fn classify_with_select_is_read() {
        assert_eq!(
            classify_statement_safety("WITH cte AS (SELECT id FROM t) SELECT * FROM cte"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn classify_with_update_is_write() {
        assert_eq!(
            classify_statement_safety("WITH cte AS (SELECT id FROM t) UPDATE t SET x = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn read_only_policy_rejects_write() {
        let policy = ConnectionSafetyPolicy::read_only();
        assert!(validate_against_policy("SELECT 1", &policy).is_ok());
        assert!(validate_against_policy("INSERT INTO t VALUES (1)", &policy).is_err());
        assert!(validate_against_policy("UPDATE t SET x = 1", &policy).is_err());
        assert!(validate_against_policy("DELETE FROM t WHERE id = 1", &policy).is_err());
    }

    #[test]
    fn full_access_policy_allows_everything() {
        let policy = ConnectionSafetyPolicy::full_access();
        assert!(validate_against_policy("SELECT 1", &policy).is_ok());
        assert!(validate_against_policy("INSERT INTO t VALUES (1)", &policy).is_ok());
        assert!(validate_against_policy("DROP TABLE t", &policy).is_ok());
        assert!(validate_against_policy("CREATE TABLE t (id INT)", &policy).is_ok());
    }

    #[test]
    fn no_ddl_policy_rejects_create_alter() {
        let policy = ConnectionSafetyPolicy {
            read_only: false,
            allow_ddl: false,
            allow_destructive: true,
            max_rows: None,
            query_timeout_ms: None,
        };
        assert!(validate_against_policy("SELECT 1", &policy).is_ok());
        assert!(validate_against_policy("CREATE TABLE t (id INT)", &policy).is_err());
        assert!(validate_against_policy("ALTER TABLE t ADD COLUMN x INT", &policy).is_err());
    }

    #[test]
    fn empty_sql_is_rejected() {
        let policy = ConnectionSafetyPolicy::full_access();
        assert!(validate_against_policy("  ", &policy).is_err());
    }

    #[test]
    fn leading_comments_stripped() {
        assert_eq!(
            classify_statement_safety("-- comment\nSELECT 1"),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_statement_safety("/* block */ DROP TABLE t"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn explain_plain_select_is_read() {
        assert_eq!(
            classify_statement_safety("EXPLAIN SELECT * FROM t"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn explain_analyze_substrings_in_query_do_not_change_plain_explain_safety() {
        assert_eq!(
            classify_statement_safety("EXPLAIN SELECT 'ANALYZE' AS analyze_column"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn explain_analyze_allows_comments_between_keywords() {
        assert_eq!(
            classify_statement_safety("EXPLAIN /* options */ ANALYZE /* query */ DELETE FROM t WHERE id = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn explain_analyze_select_is_read() {
        assert_eq!(
            classify_statement_safety("EXPLAIN ANALYZE SELECT * FROM t"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn explain_analyze_delete_is_write() {
        // EXPLAIN ANALYZE actually executes the statement — must NOT be Read.
        assert_eq!(
            classify_statement_safety("EXPLAIN ANALYZE DELETE FROM users WHERE id = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn explain_analyze_insert_is_write() {
        assert_eq!(
            classify_statement_safety("EXPLAIN ANALYZE INSERT INTO t VALUES (1)"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn explain_parenthesized_analyze_classifies_the_inner_statement() {
        assert_eq!(
            classify_statement_safety("EXPLAIN (ANALYZE true, FORMAT JSON) DELETE FROM t WHERE id = 1"),
            Some(StatementSafety::Write)
        );
        assert_eq!(
            classify_statement_safety("EXPLAIN (FORMAT JSON) DELETE FROM t"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn explain_analyze_with_unicode_query_text_does_not_panic() {
        assert_eq!(
            classify_statement_safety("EXPLAIN ANALYZE SELECT 'straße'"),
            Some(StatementSafety::Read)
        );
    }

    #[test]
    fn readonly_rejects_explain_analyze_delete() {
        let policy = ConnectionSafetyPolicy::read_only();
        assert!(validate_against_policy("EXPLAIN ANALYZE DELETE FROM users WHERE id = 1", &policy).is_err());
    }

    #[test]
    fn mutating_cte_with_delete_is_write() {
        // Data-modifying CTE: even though outer query is SELECT, the CTE mutates.
        assert_eq!(
            classify_statement_safety(
                "WITH deleted AS (DELETE FROM users WHERE id = 1 RETURNING *) SELECT * FROM deleted"
            ),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn cte_classifier_respects_lexical_boundaries_and_comment_separators() {
        assert_eq!(
            classify_statement_safety(
                "WITH deleted AS (DELETE/* comment */FROM users RETURNING *) SELECT * FROM deleted"
            ),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_statement_safety(
                "WITH cte (delete_col) AS (SELECT \"DELETE FROM users\", $$UPDATE users$$) SELECT * FROM cte"
            ),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_statement_safety(r#"WITH cte AS (SELECT E'prefix\' DELETE FROM users') SELECT * FROM cte"#),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_statement_safety(
                "WITH deleted AS (DELETE FROM users USING (SELECT id FROM audit WHERE id > 0) a RETURNING *) SELECT * FROM deleted"
            ),
            Some(StatementSafety::Destructive)
        );
        assert!(validate_against_policy(
            "WITH deleted AS (DELETE/* comment */FROM users RETURNING *) SELECT * FROM deleted",
            &ConnectionSafetyPolicy::read_only()
        )
        .is_err());
    }

    #[test]
    fn destructive_delete_inside_cte_remains_destructive() {
        assert_eq!(
            classify_statement_safety("WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted"),
            Some(StatementSafety::Destructive)
        );

        let policy = ConnectionSafetyPolicy {
            read_only: false,
            allow_ddl: true,
            allow_destructive: false,
            max_rows: None,
            query_timeout_ms: None,
        };
        assert!(validate_against_policy(
            "WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted",
            &policy
        )
        .is_err());
    }

    #[test]
    fn cte_delete_requires_a_real_where_keyword() {
        assert_eq!(
            classify_statement_safety("WITH cte AS (SELECT 1) DELETE FROM users -- WHERE id = 1"),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_statement_safety("WITH cte AS (SELECT 1) DELETE FROM users WHERE id = 1"),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn mutating_cte_with_insert_is_write() {
        assert_eq!(
            classify_statement_safety(
                "WITH moved AS (INSERT INTO t2 SELECT * FROM t1 RETURNING *) SELECT * FROM moved"
            ),
            Some(StatementSafety::Write)
        );
    }

    #[test]
    fn readonly_rejects_mutating_cte() {
        let policy = ConnectionSafetyPolicy::read_only();
        assert!(validate_against_policy(
            "WITH deleted AS (DELETE FROM users RETURNING *) SELECT * FROM deleted",
            &policy
        )
        .is_err());
    }

    #[test]
    fn split_statements_basic() {
        let stmts = split_statements("SELECT 1; SELECT 2");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_statements_trailing_semicolon() {
        let stmts = split_statements("SELECT 1; SELECT 2;");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_statements_respects_quotes() {
        let stmts = split_statements("SELECT ';'; SELECT 2");
        assert_eq!(stmts, vec!["SELECT ';'", "SELECT 2"]);
    }

    #[test]
    fn split_statements_respects_comments_and_dollar_quotes() {
        let stmts =
            split_statements("DO $body$ BEGIN PERFORM 1; /* nested ; comment */ PERFORM 2; END $body$; SELECT 2");
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("PERFORM 1;"));
        assert_eq!(stmts[1], "SELECT 2");
    }

    #[test]
    fn split_statements_drops_comment_only_tail() {
        assert_eq!(split_statements("SELECT 1; -- trailing comment\n"), vec!["SELECT 1"]);
    }

    #[test]
    fn split_statements_does_not_treat_identifier_dollar_signs_as_quotes() {
        let stmts = split_statements("SELECT foo$bar$; SELECT 2");
        assert_eq!(stmts, vec!["SELECT foo$bar$", "SELECT 2"]);
    }

    #[test]
    fn split_statements_empty() {
        let stmts = split_statements("  ");
        assert!(stmts.is_empty());
    }

    #[test]
    fn script_safety_takes_the_most_dangerous_statement() {
        assert_eq!(
            classify_script_safety("SELECT 1; DROP TABLE t;"),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_script_safety("SELECT 1; UPDATE t SET a = 1"),
            Some(StatementSafety::Write)
        );
        assert_eq!(
            classify_script_safety("DROP TABLE t; SELECT 1"),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(
            classify_script_safety("SELECT 1; CREATE TABLE t (id int); SELECT 2"),
            Some(StatementSafety::Ddl)
        );
        assert_eq!(
            classify_script_safety("SELECT 1; SELECT 2;"),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_script_safety("DROP TABLE t"),
            Some(StatementSafety::Destructive)
        );
        assert_eq!(classify_script_safety(""), None);
        assert_eq!(classify_script_safety(";"), None);
        assert_eq!(classify_script_safety("-- only a comment"), None);
    }

    #[test]
    fn script_safety_ignores_semicolons_inside_literals_and_comments() {
        assert_eq!(
            classify_script_safety("SELECT ';'; SELECT 2"),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_script_safety("SELECT 1 -- ; not a statement\n"),
            Some(StatementSafety::Read)
        );
        assert_eq!(
            classify_script_safety("DO $body$ BEGIN PERFORM 1; END $body$"),
            Some(StatementSafety::Destructive)
        );
    }

    #[test]
    fn transaction_control_verbs_are_recognised_in_every_form() {
        for (statement, verb) in [
            ("BEGIN", "BEGIN"),
            ("BEGIN;", "BEGIN"),
            ("begin", "BEGIN"),
            ("BEGIN TRANSACTION", "BEGIN"),
            ("BEGIN IMMEDIATE", "BEGIN"),
            ("START TRANSACTION", "START TRANSACTION"),
            ("COMMIT", "COMMIT"),
            ("COMMIT WORK", "COMMIT"),
            ("END", "END"),
            ("ROLLBACK", "ROLLBACK"),
            ("ROLLBACK TO SAVEPOINT before_insert", "ROLLBACK"),
            ("ABORT", "ABORT"),
            ("SAVEPOINT s1", "SAVEPOINT"),
            ("RELEASE SAVEPOINT s1", "RELEASE"),
            ("-- note\nCOMMIT", "COMMIT"),
            ("/* note */ COMMIT", "COMMIT"),
        ] {
            assert_eq!(
                transaction_control_verb(statement),
                Some(verb),
                "statement: {statement}"
            );
        }
    }

    #[test]
    fn transaction_control_detection_does_not_match_inside_a_statement() {
        for statement in [
            "",
            "   ",
            "SELECT 1",
            // A keyword that merely starts with a control keyword.
            "BEGINNING",
            "COMMITTED",
            "ENDLESS",
            // Control words inside a statement are not transaction control.
            "SELECT 'begin' AS word",
            "SELECT 1 AS commit",
            "SELECT commit FROM audit_log",
            "INSERT INTO t (begin) VALUES (1)",
            // A PL/pgSQL body: the statement is a `DO`, not a `BEGIN`.
            "DO $body$ BEGIN PERFORM 1; END $body$",
            // `START` alone is not `START TRANSACTION`.
            "START",
        ] {
            assert_eq!(transaction_control_verb(statement), None, "statement: {statement}");
        }
    }
}
