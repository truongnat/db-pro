use serde::{Deserialize, Serialize};

/// Per-table RLS enable/force state plus attached policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRlsState {
    pub schema: String,
    pub table: String,
    pub rls_enabled: bool,
    pub rls_forced: bool,
    pub policies: Vec<RlsPolicy>,
}

/// One PostgreSQL row-level security policy. Expressions are preserved losslessly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlsPolicy {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub command: String,
    pub permissive: bool,
    /// Empty means PUBLIC.
    pub roles: Vec<String>,
    pub using_expr: Option<String>,
    pub with_check_expr: Option<String>,
}
