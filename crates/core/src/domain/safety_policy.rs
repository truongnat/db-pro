use serde::{Deserialize, Serialize};

use super::{classify_statement_safety, StatementSafety};

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

/// Validate a SQL statement against a safety policy.
/// Returns `Ok(())` if the statement is allowed, or an error message.
pub fn validate_against_policy(sql: &str, policy: &ConnectionSafetyPolicy) -> Result<(), String> {
    let safety = match classify_statement_safety(sql) {
        Some(safety) => safety,
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
