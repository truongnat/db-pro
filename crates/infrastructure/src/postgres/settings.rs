//! PostgreSQL `pg_settings` inspection and safe session SET/RESET (#232).

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::pg_settings::{
    is_sensitive_setting_name, validate_setting_ident, PgSetting, PgSettingsSnapshot,
};
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::ports::DbConnector;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const LIST_SETTINGS_SQL: &str = r#"
SELECT
    name,
    setting,
    unit,
    category,
    short_desc,
    context,
    vartype,
    source,
    min_val,
    max_val,
    enumvals::text,
    boot_val,
    reset_val,
    pending_restart
FROM pg_catalog.pg_settings
ORDER BY category, name
"#;

pub struct PostgresSettingsPort {
    connector: Arc<dyn DbConnector>,
}

impl PostgresSettingsPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }

    pub async fn list_settings(
        &self,
        handle: &ConnectionHandle,
        connection_id: &str,
    ) -> Result<PgSettingsSnapshot, DbError> {
        let result = self.connector.query(handle, LIST_SETTINGS_SQL, &[]).await?;
        let mut settings = Vec::with_capacity(result.rows.len());
        for row in result.rows {
            let c = &row.0;
            let name = cell_text(c.first()).unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let sensitive = is_sensitive_setting_name(&name);
            let setting = cell_text(c.get(1)).unwrap_or_default();
            settings.push(PgSetting {
                name,
                setting: if sensitive { String::new() } else { setting },
                unit: cell_text(c.get(2)),
                category: cell_text(c.get(3)).unwrap_or_else(|| "Uncategorized".into()),
                short_desc: cell_text(c.get(4)),
                context: cell_text(c.get(5)).unwrap_or_default(),
                vartype: cell_text(c.get(6)).unwrap_or_default(),
                source: cell_text(c.get(7)).unwrap_or_default(),
                min_val: cell_text(c.get(8)),
                max_val: cell_text(c.get(9)),
                enumvals: cell_text(c.get(10)).and_then(parse_enumvals),
                boot_val: if sensitive { None } else { cell_text(c.get(11)) },
                reset_val: if sensitive { None } else { cell_text(c.get(12)) },
                pending_restart: cell_bool(c.get(13)),
                sensitive,
            });
        }
        let count = settings.len();
        Ok(PgSettingsSnapshot {
            connection_id: connection_id.to_owned(),
            settings,
            fetched_at_ms: now_ms(),
            message: format!("{count} setting(s)"),
        })
    }

    /// Session-local SET via `set_config`. Name must be a validated setting ident.
    pub async fn set_session(&self, handle: &ConnectionHandle, name: &str, value: &str) -> Result<(), DbError> {
        validate_setting_ident(name).map_err(DbError::Validation)?;
        if is_sensitive_setting_name(name) {
            return Err(DbError::Validation(
                "refusing to SET a sensitive setting name from the UI".into(),
            ));
        }
        self.connector
            .query(
                handle,
                "SELECT set_config($1, $2, false)",
                &[QueryParam::Text(name.to_owned()), QueryParam::Text(value.to_owned())],
            )
            .await?;
        Ok(())
    }

    /// Reset a session setting to its reset_val / default using `set_config(..., NULL, false)` is not
    /// enough for all GUC types — use RESET with a validated identifier.
    pub async fn reset_session(&self, handle: &ConnectionHandle, name: &str) -> Result<(), DbError> {
        validate_setting_ident(name).map_err(DbError::Validation)?;
        if is_sensitive_setting_name(name) {
            return Err(DbError::Validation(
                "refusing to RESET a sensitive setting name from the UI".into(),
            ));
        }
        // Identifier already validated to [A-Za-z0-9_.] — safe to interpolate.
        let sql = format!("RESET {name}");
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }
}

fn cell_text(cell: Option<&CellValue>) -> Option<String> {
    match cell? {
        CellValue::Null => None,
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Int64(v) => Some(v.to_string()),
        CellValue::Bool(v) => Some(v.to_string()),
        CellValue::Float64(v) => Some(v.to_string()),
        other => Some(format!("{other:?}")),
    }
}

fn cell_bool(cell: Option<&CellValue>) -> bool {
    match cell {
        Some(CellValue::Bool(v)) => *v,
        Some(CellValue::Text(s)) => s == "t" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

fn parse_enumvals(raw: String) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "NULL" {
        return None;
    }
    // Postgres array text: {a,b,c}
    let inner = trimmed
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(trimmed);
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        inner
            .split(',')
            .map(|part| part.trim().trim_matches('"').to_owned())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_enumvals_postgres_array() {
        assert_eq!(
            parse_enumvals("{on,off}".into()).as_deref(),
            Some(["on".to_owned(), "off".to_owned()].as_slice())
        );
    }
}
