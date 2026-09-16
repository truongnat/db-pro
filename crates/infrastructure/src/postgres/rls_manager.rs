use std::sync::Arc;

use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::domain::rls::{RlsPolicy, TableRlsState};
use db_pro_core::ports::{DbConnector, RlsManager};

pub struct PostgresRlsManager {
    connector: Arc<dyn DbConnector>,
}

impl PostgresRlsManager {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }
}

fn cell_text(cell: &CellValue) -> Option<String> {
    match cell {
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Null => None,
        other => Some(format!("{other:?}")),
    }
}

fn cell_bool(cell: &CellValue) -> bool {
    matches!(cell, CellValue::Bool(true)) || matches!(cell, CellValue::Text(s) if s == "t" || s == "true")
}

fn parse_roles(raw: Option<String>) -> Vec<String> {
    let Some(raw) = raw.filter(|s| !s.is_empty() && s != "{}") else {
        return Vec::new();
    };
    raw.trim_matches(|c| c == '{' || c == '}')
        .split(',')
        .map(|part| part.trim().trim_matches('"').to_owned())
        .filter(|part| !part.is_empty() && part != "0")
        .collect()
}

#[async_trait]
impl RlsManager for PostgresRlsManager {
    async fn table_rls_state(
        &self,
        handle: &ConnectionHandle,
        schema: &str,
        table: &str,
    ) -> Result<TableRlsState, DbError> {
        let flags = self
            .connector
            .query(
                handle,
                "SELECT c.relrowsecurity, c.relforcerowsecurity \
                 FROM pg_class c \
                 JOIN pg_namespace n ON n.oid = c.relnamespace \
                 WHERE n.nspname = $1 AND c.relname = $2 AND c.relkind = 'r'",
                &[QueryParam::Text(schema.to_owned()), QueryParam::Text(table.to_owned())],
            )
            .await?;
        let Some(flag_row) = flags.rows.first() else {
            return Err(DbError::QueryFailed(format!("table {schema}.{table} not found")));
        };
        let rls_enabled = flag_row.0.first().map(cell_bool).unwrap_or(false);
        let rls_forced = flag_row.0.get(1).map(cell_bool).unwrap_or(false);

        let policies = self
            .connector
            .query(
                handle,
                "SELECT p.polname, \
                        CASE p.polcmd \
                          WHEN 'r' THEN 'SELECT' \
                          WHEN 'a' THEN 'INSERT' \
                          WHEN 'w' THEN 'UPDATE' \
                          WHEN 'd' THEN 'DELETE' \
                          WHEN '*' THEN 'ALL' \
                          ELSE p.polcmd::text \
                        END AS command, \
                        p.polpermissive, \
                        COALESCE(ARRAY( \
                          SELECT rolname FROM pg_roles WHERE oid = ANY (p.polroles) \
                        )::text, '{}') AS roles, \
                        pg_get_expr(p.polqual, p.polrelid) AS using_expr, \
                        pg_get_expr(p.polwithcheck, p.polrelid) AS with_check_expr \
                 FROM pg_policy p \
                 JOIN pg_class c ON c.oid = p.polrelid \
                 JOIN pg_namespace n ON n.oid = c.relnamespace \
                 WHERE n.nspname = $1 AND c.relname = $2 \
                 ORDER BY p.polname",
                &[QueryParam::Text(schema.to_owned()), QueryParam::Text(table.to_owned())],
            )
            .await?;

        let policies = policies
            .rows
            .into_iter()
            .filter_map(|row| {
                let cells = &row.0;
                Some(RlsPolicy {
                    schema: schema.to_owned(),
                    table: table.to_owned(),
                    name: cells.first().and_then(cell_text)?,
                    command: cells.get(1).and_then(cell_text).unwrap_or_else(|| "ALL".into()),
                    permissive: cells.get(2).map(cell_bool).unwrap_or(true),
                    roles: parse_roles(cells.get(3).and_then(cell_text)),
                    using_expr: cells.get(4).and_then(cell_text),
                    with_check_expr: cells.get(5).and_then(cell_text),
                })
            })
            .collect();

        Ok(TableRlsState {
            schema: schema.to_owned(),
            table: table.to_owned(),
            rls_enabled,
            rls_forced,
            policies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::parse_roles;

    #[test]
    fn parse_roles_handles_pg_array_text() {
        assert!(parse_roles(None).is_empty());
        assert!(parse_roles(Some("{}".into())).is_empty());
        assert_eq!(
            parse_roles(Some("{app_reader,app_writer}".into())),
            vec!["app_reader".to_owned(), "app_writer".to_owned()]
        );
    }
}
