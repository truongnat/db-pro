use serde::{Deserialize, Serialize};

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
        "MERGE" => classify_merge_safety(trimmed),
        _ => Some(StatementSafety::Write),
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

fn matching_parenthesis_end(sql: &str) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut depth = 0;
    let mut index = 0;
    while index < bytes.len() {
        index = match bytes[index] {
            b'\'' => skip_quoted(bytes, index, b'\''),
            b'"' => skip_quoted(bytes, index, b'"'),
            b'-' if bytes.get(index + 1) == Some(&b'-') => skip_line_comment(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'*') => skip_block_comment(bytes, index),
            b'$' => skip_dollar_quote(bytes, index).unwrap_or(index + 1),
            b'(' => {
                depth += 1;
                index + 1
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
                index + 1
            }
            _ => index + 1,
        };
    }

    None
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

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn scan_identifier(bytes: &[u8], start: usize) -> usize {
    let mut end = start + 1;
    while end < bytes.len() && is_identifier_continue(bytes[end]) {
        end += 1;
    }
    end
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if quote == b'\'' && bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
            } else {
                return index + 1;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| start + 2 + offset + 1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        if &bytes[index..index + 2] == b"/*" {
            depth += 1;
            index += 2;
        } else if &bytes[index..index + 2] == b"*/" {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn skip_dollar_quote(bytes: &[u8], start: usize) -> Option<usize> {
    if start > 0 && is_identifier_continue(bytes[start - 1]) {
        return None;
    }

    let delimiter_end = bytes[start + 1..]
        .iter()
        .position(|byte| *byte == b'$')
        .map(|offset| start + 1 + offset)?;
    let tag = &bytes[start + 1..delimiter_end];
    if !tag.is_empty() && (!is_identifier_start(tag[0]) || !tag[1..].iter().all(|byte| is_identifier_continue(*byte))) {
        return None;
    }

    let delimiter = &bytes[start..=delimiter_end];
    let content_start = delimiter_end + 1;
    bytes[content_start..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
        .map_or(Some(bytes.len()), |offset| {
            Some(content_start + offset + delimiter.len())
        })
}

/// A significant SQL token used by the safety classifier. Quoted strings,
/// quoted identifiers, comments, and dollar-quoted bodies are skipped before
/// these tokens are emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SqlTokenKind {
    Word,
    OpenParen,
    CloseParen,
    Comma,
}

#[derive(Debug, Clone, Copy)]
struct SqlToken {
    kind: SqlTokenKind,
    start: usize,
    end: usize,
}

fn tokenize_sql(sql: &str) -> Vec<SqlToken> {
    let bytes = sql.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(end) = match bytes[index] {
            b'\'' => Some(skip_quoted(bytes, index, b'\'')),
            b'"' => Some(skip_quoted(bytes, index, b'"')),
            b'-' if bytes.get(index + 1) == Some(&b'-') => Some(skip_line_comment(bytes, index)),
            b'/' if bytes.get(index + 1) == Some(&b'*') => Some(skip_block_comment(bytes, index)),
            b'$' => skip_dollar_quote(bytes, index),
            _ => None,
        } {
            index = end;
            continue;
        }

        let kind = match bytes[index] {
            b'(' => Some(SqlTokenKind::OpenParen),
            b')' => Some(SqlTokenKind::CloseParen),
            b',' => Some(SqlTokenKind::Comma),
            byte if is_identifier_start(byte) => {
                let end = scan_identifier(bytes, index);
                tokens.push(SqlToken {
                    kind: SqlTokenKind::Word,
                    start: index,
                    end,
                });
                index = end;
                continue;
            }
            _ => None,
        };

        if let Some(kind) = kind {
            tokens.push(SqlToken {
                kind,
                start: index,
                end: index + 1,
            });
        }
        index += 1;
    }

    tokens
}

fn token_is_word(sql: &str, token: SqlToken, expected: &str) -> bool {
    token.kind == SqlTokenKind::Word && sql[token.start..token.end].eq_ignore_ascii_case(expected)
}

/// For WITH (CTE) statements, find the main keyword after the CTE definitions.
/// Data-modifying CTE bodies execute as side effects even when the outer query
/// returns rows, so their mutations must participate in safety classification.
fn classify_cte_safety(sql: &str) -> Option<StatementSafety> {
    let tokens = tokenize_sql(sql);
    let with_index = tokens.iter().position(|token| token_is_word(sql, *token, "WITH"))?;
    let mut cursor = with_index + 1;
    if tokens
        .get(cursor)
        .is_some_and(|token| token_is_word(sql, *token, "RECURSIVE"))
    {
        cursor += 1;
    }

    let mut has_mutation = false;
    let mut has_destructive_delete = false;
    let outer_index = loop {
        let as_index = find_cte_as_token(sql, &tokens, cursor)?;
        let mut body_index = as_index + 1;
        if tokens
            .get(body_index)
            .is_some_and(|token| token_is_word(sql, *token, "NOT"))
            && tokens
                .get(body_index + 1)
                .is_some_and(|token| token_is_word(sql, *token, "MATERIALIZED"))
        {
            body_index += 2;
        } else if tokens
            .get(body_index)
            .is_some_and(|token| token_is_word(sql, *token, "MATERIALIZED"))
        {
            body_index += 1;
        }

        if tokens.get(body_index).map(|token| token.kind) != Some(SqlTokenKind::OpenParen) {
            return Some(StatementSafety::Destructive);
        }
        let close_index = matching_token_parenthesis(&tokens, body_index)?;
        let (body_mutation, body_destructive_delete) = classify_cte_body(&tokens, body_index + 1, close_index, sql);
        has_mutation |= body_mutation;
        has_destructive_delete |= body_destructive_delete;

        let next_index = close_index + 1;
        if tokens.get(next_index).map(|token| token.kind) == Some(SqlTokenKind::Comma) {
            cursor = next_index + 1;
            continue;
        }
        break next_index;
    };

    let outer_safety = tokens
        .get(outer_index)
        .and_then(|token| classify_statement_safety(&sql[token.start..]));
    combine_cte_safety(outer_safety, has_mutation, has_destructive_delete)
}

fn find_cte_as_token(sql: &str, tokens: &[SqlToken], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
            SqlTokenKind::Word if depth == 0 && token_is_word(sql, *token, "AS") => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_token_parenthesis(tokens: &[SqlToken], open_index: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn classify_cte_body(tokens: &[SqlToken], start: usize, end: usize, sql: &str) -> (bool, bool) {
    let mut depth = 0i32;
    let mut has_mutation = false;
    let mut has_destructive_delete = false;

    for index in start..end {
        let token = tokens[index];
        if token.kind == SqlTokenKind::Word {
            if token_is_word(sql, token, "DROP") || token_is_word(sql, token, "TRUNCATE") {
                has_mutation = true;
                has_destructive_delete = true;
            } else if token_is_word(sql, token, "INSERT") || token_is_word(sql, token, "UPDATE") {
                has_mutation = true;
            } else if token_is_word(sql, token, "DELETE") {
                has_mutation = true;
                if !delete_has_same_level_where(tokens, index, end, depth, sql) {
                    has_destructive_delete = true;
                }
            }
        }

        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    (has_mutation, has_destructive_delete)
}

fn delete_has_same_level_where(
    tokens: &[SqlToken],
    delete_index: usize,
    end: usize,
    delete_depth: i32,
    sql: &str,
) -> bool {
    let mut depth = delete_depth;
    for token in tokens.iter().take(end).skip(delete_index + 1) {
        match token.kind {
            SqlTokenKind::OpenParen => depth += 1,
            SqlTokenKind::CloseParen => depth = depth.saturating_sub(1),
            SqlTokenKind::Word if depth == delete_depth && token_is_word(sql, *token, "WHERE") => return true,
            _ => {}
        }
    }
    false
}

fn combine_cte_safety(
    outer_safety: Option<StatementSafety>,
    has_mutation: bool,
    has_destructive_delete: bool,
) -> Option<StatementSafety> {
    if !has_mutation {
        return outer_safety;
    }
    if has_destructive_delete || outer_safety == Some(StatementSafety::Destructive) {
        Some(StatementSafety::Destructive)
    } else if outer_safety == Some(StatementSafety::Read) {
        Some(StatementSafety::Write)
    } else {
        outer_safety
    }
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
}
