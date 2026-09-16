//! PostgreSQL Foreign Data Wrapper administration models (#229).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignDataWrapper {
    pub name: String,
    pub handler: Option<String>,
    pub validator: Option<String>,
    pub options: Vec<FdwOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignServer {
    pub name: String,
    pub fdw_name: String,
    pub type_: Option<String>,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub options: Vec<FdwOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserMapping {
    pub server_name: String,
    pub user_name: String,
    pub options: Vec<FdwOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignTableInfo {
    pub schema: String,
    pub name: String,
    pub server_name: String,
    pub options: Vec<FdwOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FdwOption {
    pub key: String,
    pub value: String,
    pub sensitive: bool,
}

impl FdwOption {
    pub fn display_value(&self) -> &str {
        if self.sensitive {
            "[redacted]"
        } else {
            &self.value
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FdwInventory {
    pub connection_id: String,
    pub wrappers: Vec<ForeignDataWrapper>,
    pub servers: Vec<ForeignServer>,
    pub user_mappings: Vec<UserMapping>,
    pub foreign_tables: Vec<ForeignTableInfo>,
    pub message: String,
    pub extension_hint: Option<String>,
}

pub fn is_sensitive_option_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    k.contains("password")
        || k.contains("passwd")
        || k.contains("secret")
        || k.contains("token")
        || k.contains("private")
}

pub fn validate_sql_ident(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 63 {
        return Err("invalid identifier".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("identifier must be [A-Za-z0-9_]".into());
    }
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Err("identifier cannot start with a digit".into());
    }
    Ok(())
}

pub fn quote_ident(name: &str) -> Result<String, String> {
    validate_sql_ident(name)?;
    Ok(format!("\"{}\"", name.replace('"', "\"\"")))
}

pub fn preview_drop_server(name: &str, cascade: bool) -> Result<String, String> {
    let ident = quote_ident(name)?;
    Ok(format!(
        "DROP SERVER IF EXISTS {ident}{};",
        if cascade { " CASCADE" } else { "" }
    ))
}

pub fn preview_drop_user_mapping(server: &str, user: &str) -> Result<String, String> {
    let server = quote_ident(server)?;
    let user = if user.eq_ignore_ascii_case("public") {
        "PUBLIC".to_owned()
    } else {
        quote_ident(user)?
    };
    Ok(format!("DROP USER MAPPING IF EXISTS FOR {user} SERVER {server};"))
}

pub fn preview_create_server(name: &str, fdw: &str, host: &str, dbname: &str, port: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    let fdw = quote_ident(fdw)?;
    let mut options = Vec::new();
    if !host.trim().is_empty() {
        options.push(format!("host '{}'", host.replace('\'', "''")));
    }
    if !dbname.trim().is_empty() {
        options.push(format!("dbname '{}'", dbname.replace('\'', "''")));
    }
    if !port.trim().is_empty() {
        options.push(format!("port '{}'", port.replace('\'', "''")));
    }
    let opts = if options.is_empty() {
        String::new()
    } else {
        format!(" OPTIONS ({})", options.join(", "))
    };
    Ok(format!("CREATE SERVER {name} FOREIGN DATA WRAPPER {fdw}{opts};"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_password_options() {
        assert!(is_sensitive_option_key("password"));
        assert!(!is_sensitive_option_key("host"));
    }

    #[test]
    fn preview_sql_quotes_idents() {
        let sql = preview_drop_server("remote_pg", true).unwrap();
        assert!(sql.contains("DROP SERVER IF EXISTS \"remote_pg\" CASCADE"));
        assert!(preview_create_server("s1", "postgres_fdw", "h", "db", "5432")
            .unwrap()
            .contains("FOREIGN DATA WRAPPER \"postgres_fdw\""));
    }
}
