use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub name: String,
    pub is_super: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_login: bool,
    /// Provider-defined role (PostgreSQL `pg_*` and similar) — not droppable via the UI.
    #[serde(default)]
    pub is_predefined: bool,
}

pub fn is_predefined_role_name(name: &str) -> bool {
    name == "postgres" || name.starts_with("pg_")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeObjectKind {
    #[default]
    Table,
    Schema,
    Database,
    Sequence,
}

impl PrivilegeObjectKind {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Schema => "schema",
            Self::Database => "database",
            Self::Sequence => "sequence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Privilege {
    pub object_kind: PrivilegeObjectKind,
    /// Schema name when applicable; empty for database-level grants.
    pub schema: String,
    /// Table/sequence name, schema name (for schema grants), or database name.
    pub object_name: String,
    pub privilege_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleMembership {
    /// Role that `member` belongs to.
    pub role: String,
    pub member: String,
    pub admin_option: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoleAttributes {
    pub login: Option<bool>,
    pub superuser: Option<bool>,
    pub createdb: Option<bool>,
    pub createrole: Option<bool>,
}
