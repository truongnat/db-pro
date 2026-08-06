use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub name: String,
    pub is_super: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Privilege {
    pub schema: String,
    pub table: String,
    pub privilege_type: String,
}
