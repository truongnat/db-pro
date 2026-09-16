//! PostgreSQL logical replication models (#230).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationInfo {
    pub name: String,
    pub owner: Option<String>,
    pub all_tables: bool,
    pub insert: bool,
    pub update: bool,
    pub delete: bool,
    pub truncate: bool,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub name: String,
    pub owner: Option<String>,
    pub enabled: bool,
    pub slot_name: Option<String>,
    pub publications: Vec<String>,
    /// Always redacted in inventory — never store raw conninfo here.
    pub conninfo_redacted: String,
    pub has_conninfo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationSlotInfo {
    pub slot_name: String,
    pub plugin: Option<String>,
    pub slot_type: Option<String>,
    pub database: Option<String>,
    pub active: bool,
    pub restart_lsn: Option<String>,
    pub confirmed_flush_lsn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReplicationInventory {
    pub connection_id: String,
    pub publications: Vec<PublicationInfo>,
    pub subscriptions: Vec<SubscriptionInfo>,
    pub slots: Vec<ReplicationSlotInfo>,
    pub message: String,
}

pub fn redact_conninfo(raw: &str) -> String {
    let mut out = raw.to_owned();
    for key in ["password=", "PASSWORD=", "passfile=", "sslkey=", "sslpassword="] {
        if let Some(idx) = out.find(key) {
            let start = idx + key.len();
            let end = out[start..]
                .find(|c: char| c.is_whitespace())
                .map(|i| start + i)
                .unwrap_or(out.len());
            out.replace_range(start..end, "[redacted]");
        }
    }
    out
}

pub fn validate_repl_ident(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 63 {
        return Err("invalid replication object name".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("name must be [A-Za-z0-9_]".into());
    }
    Ok(())
}

pub fn quote_ident(name: &str) -> Result<String, String> {
    validate_repl_ident(name)?;
    Ok(format!("\"{name}\""))
}

pub fn preview_create_publication_all(name: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    Ok(format!("CREATE PUBLICATION {name} FOR ALL TABLES;"))
}

pub fn preview_drop_publication(name: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    Ok(format!("DROP PUBLICATION IF EXISTS {name};"))
}

pub fn preview_drop_subscription(name: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    Ok(format!("DROP SUBSCRIPTION IF EXISTS {name};"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_password_in_conninfo() {
        let raw = "host=db user=u password=s3cret dbname=app";
        let red = redact_conninfo(raw);
        assert!(red.contains("password=[redacted]"));
        assert!(!red.contains("s3cret"));
    }

    #[test]
    fn preview_publication_sql() {
        assert_eq!(
            preview_create_publication_all("pub1").unwrap(),
            "CREATE PUBLICATION \"pub1\" FOR ALL TABLES;"
        );
    }

    #[test]
    fn preview_drop_sql() {
        assert_eq!(
            preview_drop_publication("pub1").unwrap(),
            "DROP PUBLICATION IF EXISTS \"pub1\";"
        );
        assert_eq!(
            preview_drop_subscription("sub1").unwrap(),
            "DROP SUBSCRIPTION IF EXISTS \"sub1\";"
        );
    }

    #[test]
    fn rejects_unsafe_ident() {
        assert!(validate_repl_ident("bad-name").is_err());
        assert!(quote_ident(";drop").is_err());
    }
}
