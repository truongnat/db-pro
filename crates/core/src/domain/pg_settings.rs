//! PostgreSQL `pg_settings` models (#232).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgSetting {
    pub name: String,
    pub setting: String,
    pub unit: Option<String>,
    pub category: String,
    pub short_desc: Option<String>,
    pub context: String,
    pub vartype: String,
    pub source: String,
    pub min_val: Option<String>,
    pub max_val: Option<String>,
    pub enumvals: Option<Vec<String>>,
    pub boot_val: Option<String>,
    pub reset_val: Option<String>,
    pub pending_restart: bool,
    /// True when the value looks secret-bearing and should be redacted in UI/logs.
    pub sensitive: bool,
}

impl PgSetting {
    pub fn display_setting(&self) -> &str {
        if self.sensitive {
            "[redacted]"
        } else {
            &self.setting
        }
    }

    pub fn session_mutable(&self) -> bool {
        matches!(self.context.as_str(), "user" | "superuser" | "superuser-backend")
    }

    pub fn mutability_reason(&self) -> &'static str {
        match self.context.as_str() {
            "user" => "Session SET/RESET allowed for this context",
            "superuser" | "superuser-backend" => "Session SET/RESET (superuser context)",
            "internal" => "Internal setting — not user-mutable",
            "postmaster" => "Requires server restart (postmaster)",
            "sighup" => "Requires reload (SIGHUP) — not applied via session SET",
            "backend" => "Backend-start only",
            "" => "Unknown context",
            _ => "Not session-mutable in this context",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgSettingsSnapshot {
    pub connection_id: String,
    pub settings: Vec<PgSetting>,
    pub fetched_at_ms: u64,
    pub message: String,
}

/// Preview-only SQL for higher-privilege changes (never auto-executed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgSettingPreviewSql {
    pub kind: String,
    pub sql: String,
    pub note: String,
}

pub fn is_sensitive_setting_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("password")
        || n.contains("ssl_key")
        || n.contains("ssl_pass")
        || n.contains("secret")
        || n.contains("private")
}

pub fn preview_alter_system(name: &str, value: &str) -> Result<PgSettingPreviewSql, String> {
    validate_setting_ident(name)?;
    if is_sensitive_setting_name(name) {
        return Err("refusing to preview ALTER SYSTEM for sensitive setting names".into());
    }
    let lit = escape_literal(value);
    Ok(PgSettingPreviewSql {
        kind: "ALTER SYSTEM".into(),
        sql: format!("ALTER SYSTEM SET {name} = {lit};"),
        note: "Preview only — never auto-applied. Requires admin + reload/restart as appropriate.".into(),
    })
}

pub fn validate_setting_ident(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err("invalid setting name".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        return Err("setting name contains illegal characters".into());
    }
    Ok(())
}

fn escape_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_names_detected() {
        assert!(is_sensitive_setting_name("ssl_key_file"));
        assert!(!is_sensitive_setting_name("work_mem"));
    }

    #[test]
    fn preview_alter_system_quotes_value() {
        let preview = preview_alter_system("work_mem", "64MB").unwrap();
        assert!(preview.sql.contains("ALTER SYSTEM SET work_mem = '64MB'"));
        assert!(preview_alter_system("ssl_key_file", "x").is_err());
    }

    #[test]
    fn session_mutable_contexts() {
        let mut s = PgSetting {
            name: "work_mem".into(),
            setting: "4MB".into(),
            unit: Some("kB".into()),
            category: "Resource Usage".into(),
            short_desc: None,
            context: "user".into(),
            vartype: "string".into(),
            source: "default".into(),
            min_val: None,
            max_val: None,
            enumvals: None,
            boot_val: None,
            reset_val: None,
            pending_restart: false,
            sensitive: false,
        };
        assert!(s.session_mutable());
        s.context = "postmaster".into();
        assert!(!s.session_mutable());
    }
}
