use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::QueryResult;
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::ports::{ConnectionRepository, DbConnector};

use super::registry::ConnectionRegistry;
use super::sql_policy::reject_multi_statement_for_driver;
use export_formats::{render_csv, render_excel, render_json};

#[cfg(test)]
use export_formats::{excel_column_index, excel_integer_is_exact, excel_row_index, MAX_EXACT_EXCEL_INTEGER};

#[path = "export_formats.rs"]
mod export_formats;

#[derive(Debug)]
pub struct ExportResult {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
    pub row_count: u64,
}

pub struct ExportService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl ExportService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            registry,
            connections,
        }
    }

    // cc-scan:allow COMPLEXITY — classifier/dispatch ladder — one case per branch
    async fn execute_for_export(&self, connection_id: &ConnectionId, sql: &str) -> Result<QueryResult, DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        reject_multi_statement_for_driver(sql, config.driver)?;
        let policy = if config.readonly {
            ConnectionSafetyPolicy::read_only()
        } else {
            ConnectionSafetyPolicy::full_access()
        };
        validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let result = self.connector.query(&handle, sql, &[]).await?;
        result.validate().map_err(DbError::QueryFailed)?;
        Ok(result)
    }

    pub async fn export_csv(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        let content = render_csv(&result)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.csv"),
            mime_type: "text/csv".into(),
            row_count: result.row_count,
        })
    }

    pub async fn export_json(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        let content = render_json(&result)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.json"),
            mime_type: "application/json".into(),
            row_count: result.row_count,
        })
    }

    pub async fn export_excel(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        let content = render_excel(&result)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.xlsx"),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
            row_count: result.row_count,
        })
    }
}

#[cfg(test)]
#[path = "tests/export_service_tests.rs"]
mod tests;
