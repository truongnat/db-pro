//! PostgreSQL FDW inventory and safe admin DDL (#229).

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::fdw::{
    is_sensitive_option_key, validate_sql_ident, FdwInventory, FdwOption, ForeignDataWrapper, ForeignServer,
    ForeignTableInfo, UserMapping,
};
use db_pro_core::domain::query::CellValue;
use db_pro_core::ports::DbConnector;
use std::sync::Arc;

pub struct PostgresFdwPort {
    connector: Arc<dyn DbConnector>,
}

impl PostgresFdwPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }

    pub async fn inventory(&self, handle: &ConnectionHandle, connection_id: &str) -> Result<FdwInventory, DbError> {
        let wrappers = self.list_wrappers(handle).await?;
        let servers = self.list_servers(handle).await?;
        let user_mappings = self.list_user_mappings(handle).await?;
        let foreign_tables = self.list_foreign_tables(handle).await?;

        let extension_hint = if wrappers.is_empty() && servers.is_empty() {
            Some("No FDWs found. Install e.g. CREATE EXTENSION postgres_fdw (never auto-installed by DB Pro).".into())
        } else {
            None
        };

        let message = format!(
            "{} wrapper(s) · {} server(s) · {} mapping(s) · {} foreign table(s)",
            wrappers.len(),
            servers.len(),
            user_mappings.len(),
            foreign_tables.len()
        );

        Ok(FdwInventory {
            connection_id: connection_id.to_owned(),
            wrappers,
            servers,
            user_mappings,
            foreign_tables,
            message,
            extension_hint,
        })
    }

    async fn list_wrappers(&self, handle: &ConnectionHandle) -> Result<Vec<ForeignDataWrapper>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    fdwname::text,
    fdwhandler::regproc::text,
    fdwvalidator::regproc::text,
    COALESCE((
        SELECT string_agg(option_name || '=' || option_value, ',')
        FROM pg_options_to_table(fdwoptions)
    ), '')
FROM pg_catalog.pg_foreign_data_wrapper
ORDER BY fdwname
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let name = cell_text(row.0.first())?;
                Some(ForeignDataWrapper {
                    name,
                    handler: cell_text(row.0.get(1)),
                    validator: cell_text(row.0.get(2)),
                    options: parse_options(cell_text(row.0.get(3)).unwrap_or_default()),
                })
            })
            .collect())
    }

    async fn list_servers(&self, handle: &ConnectionHandle) -> Result<Vec<ForeignServer>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    s.srvname::text,
    w.fdwname::text,
    s.srvtype::text,
    s.srvversion::text,
    pg_catalog.pg_get_userbyid(s.srvowner)::text,
    COALESCE((
        SELECT string_agg(option_name || '=' || option_value, ',')
        FROM pg_options_to_table(s.srvoptions)
    ), '')
FROM pg_catalog.pg_foreign_server s
JOIN pg_catalog.pg_foreign_data_wrapper w ON w.oid = s.srvfdw
ORDER BY s.srvname
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                let name = cell_text(row.0.first())?;
                Some(ForeignServer {
                    name,
                    fdw_name: cell_text(row.0.get(1)).unwrap_or_default(),
                    type_: cell_text(row.0.get(2)),
                    version: cell_text(row.0.get(3)),
                    owner: cell_text(row.0.get(4)),
                    options: parse_options(cell_text(row.0.get(5)).unwrap_or_default()),
                })
            })
            .collect())
    }

    async fn list_user_mappings(&self, handle: &ConnectionHandle) -> Result<Vec<UserMapping>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    s.srvname::text,
    COALESCE(u.rolname::text, 'public'),
    COALESCE((
        SELECT string_agg(option_name || '=' || option_value, ',')
        FROM pg_options_to_table(m.umoptions)
    ), '')
FROM pg_catalog.pg_user_mappings m
JOIN pg_catalog.pg_foreign_server s ON s.oid = m.srvid
LEFT JOIN pg_catalog.pg_roles u ON u.oid = m.umuser
ORDER BY s.srvname, 2
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                Some(UserMapping {
                    server_name: cell_text(row.0.first())?,
                    user_name: cell_text(row.0.get(1)).unwrap_or_else(|| "public".into()),
                    options: parse_options(cell_text(row.0.get(2)).unwrap_or_default()),
                })
            })
            .collect())
    }

    async fn list_foreign_tables(&self, handle: &ConnectionHandle) -> Result<Vec<ForeignTableInfo>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
SELECT
    n.nspname::text,
    c.relname::text,
    s.srvname::text,
    COALESCE((
        SELECT string_agg(option_name || '=' || option_value, ',')
        FROM pg_options_to_table(t.ftoptions)
    ), '')
FROM pg_catalog.pg_foreign_table t
JOIN pg_catalog.pg_class c ON c.oid = t.ftrelid
JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
JOIN pg_catalog.pg_foreign_server s ON s.oid = t.ftserver
ORDER BY n.nspname, c.relname
"#,
                &[],
            )
            .await?;
        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                Some(ForeignTableInfo {
                    schema: cell_text(row.0.first())?,
                    name: cell_text(row.0.get(1))?,
                    server_name: cell_text(row.0.get(2)).unwrap_or_default(),
                    options: parse_options(cell_text(row.0.get(3)).unwrap_or_default()),
                })
            })
            .collect())
    }

    pub async fn execute_admin_sql(&self, handle: &ConnectionHandle, sql: &str) -> Result<(), DbError> {
        let trimmed = sql.trim();
        if trimmed.is_empty() {
            return Err(DbError::Validation("empty FDW DDL".into()));
        }
        let upper = trimmed.to_ascii_uppercase();
        if !(upper.starts_with("CREATE SERVER")
            || upper.starts_with("DROP SERVER")
            || upper.starts_with("CREATE USER MAPPING")
            || upper.starts_with("DROP USER MAPPING")
            || upper.starts_with("CREATE FOREIGN TABLE")
            || upper.starts_with("DROP FOREIGN TABLE"))
        {
            return Err(DbError::Validation(
                "only FDW CREATE/DROP SERVER|USER MAPPING|FOREIGN TABLE DDL is allowed".into(),
            ));
        }
        self.connector.execute(handle, trimmed, &[]).await?;
        Ok(())
    }

    pub async fn drop_server(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        cascade: bool,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "dropping a foreign server requires explicit confirmation".into(),
            ));
        }
        validate_sql_ident(name).map_err(DbError::Validation)?;
        let sql = db_pro_core::domain::fdw::preview_drop_server(name, cascade).map_err(DbError::Validation)?;
        self.execute_admin_sql(handle, &sql).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_server(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        fdw: &str,
        host: &str,
        dbname: &str,
        port: &str,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "creating a foreign server requires explicit confirmation".into(),
            ));
        }
        let sql = db_pro_core::domain::fdw::preview_create_server(name, fdw, host, dbname, port)
            .map_err(DbError::Validation)?;
        self.execute_admin_sql(handle, &sql).await
    }
}

fn cell_text(cell: Option<&CellValue>) -> Option<String> {
    match cell? {
        CellValue::Null => None,
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Int64(v) => Some(v.to_string()),
        other => Some(format!("{other:?}")),
    }
}

fn parse_options(raw: String) -> Vec<FdwOption> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    raw.split(',')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            let key = key.trim().to_owned();
            if key.is_empty() {
                return None;
            }
            let sensitive = is_sensitive_option_key(&key);
            Some(FdwOption {
                key,
                value: if sensitive {
                    String::new()
                } else {
                    value.trim().to_owned()
                },
                sensitive,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options_redacts_password() {
        let opts = parse_options("host=db,password=secret,port=5432".into());
        assert_eq!(opts.len(), 3);
        assert!(opts
            .iter()
            .any(|o| o.key == "password" && o.sensitive && o.value.is_empty()));
        assert!(quote_ident("ok_name").is_ok());
    }
}
