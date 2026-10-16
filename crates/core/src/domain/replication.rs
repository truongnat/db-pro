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

const SENSITIVE_CONNINFO_KEYS: [&str; 5] = ["password=", "PASSWORD=", "passfile=", "sslkey=", "sslpassword="];
const REDACTED_CONNINFO_VALUE: &str = "[redacted]";

pub fn redact_conninfo(raw: &str) -> String {
    let mut out = raw.to_owned();
    for key in SENSITIVE_CONNINFO_KEYS {
        let mut search_from = 0;
        while let Some(offset) = out[search_from..].find(key) {
            let value_start = search_from + offset + key.len();
            let value_end = conninfo_value_end(out.as_bytes(), value_start);
            out.replace_range(value_start..value_end, REDACTED_CONNINFO_VALUE);
            search_from = value_start + REDACTED_CONNINFO_VALUE.len();
        }
    }
    out
}

/// End index of a libpq conninfo `key=value` pair: a single-quoted value runs to
/// its closing quote (honouring `\'`/`\\` escapes), any other value to the next
/// whitespace.
fn conninfo_value_end(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start) == Some(&b'\'') {
        let mut index = start + 1;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' if index + 1 < bytes.len() => index += 2,
                b'\'' => return index + 1,
                _ => index += 1,
            }
        }
        return bytes.len();
    }
    bytes[start..]
        .iter()
        .position(|byte| byte.is_ascii_whitespace())
        .map_or(bytes.len(), |offset| start + offset)
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
#[path = "tests/replication_tests.rs"]
mod tests;
