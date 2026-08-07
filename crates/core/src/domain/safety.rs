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
        "SELECT" | "SHOW" | "EXPLAIN" | "TABLE" => Some(StatementSafety::Read),
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
        _ => Some(StatementSafety::Write),
    }
}

/// Check whether a DELETE statement lacks a WHERE clause.
fn is_delete_without_where(sql: &str) -> bool {
    let upper = sql.to_ascii_uppercase();
    // Simple heuristic: if "WHERE" doesn't appear after "DELETE"
    !upper.contains("WHERE")
}

/// For WITH (CTE) statements, find the main keyword after the CTE definitions.
fn classify_cte_safety(sql: &str) -> Option<StatementSafety> {
    let _upper = sql.to_ascii_uppercase();
    let trimmed = sql.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();

    // Skip past "WITH"
    let mut i = 4;
    let mut depth: i32 = 0;
    let mut in_string = false;

    while i < len {
        if in_string {
            if chars[i] == '\'' {
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 1;
                } else {
                    in_string = false;
                }
            }
            i += 1;
            continue;
        }

        match chars[i] {
            '\'' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    while i < len && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < len && chars[i] == ',' {
                        i += 1;
                        continue;
                    }
                    let remaining: String = chars[i..].iter().collect();
                    let kw = remaining
                        .trim_start()
                        .split_whitespace()
                        .next()?
                        .to_ascii_uppercase();
                    return match kw.as_str() {
                        "SELECT" | "SHOW" | "EXPLAIN" => Some(StatementSafety::Read),
                        "INSERT" | "UPDATE" => Some(StatementSafety::Write),
                        "DELETE" => {
                            if remaining.to_ascii_uppercase().contains("WHERE") {
                                Some(StatementSafety::Write)
                            } else {
                                Some(StatementSafety::Destructive)
                            }
                        }
                        _ => Some(StatementSafety::Write),
                    };
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Fallback: treat WITH as read
    Some(StatementSafety::Read)
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
pub fn validate_against_policy(
    sql: &str,
    policy: &ConnectionSafetyPolicy,
) -> Result<(), String> {
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
}
