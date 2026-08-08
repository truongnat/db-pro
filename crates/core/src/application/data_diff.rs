use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::cross_connection::DataDiff;
use crate::domain::error::DbError;
use crate::ports::DbConnector;

use super::registry::ConnectionRegistry;

pub struct DataDiffService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
}

impl DataDiffService {
    pub fn new(connector: Box<dyn DbConnector>, registry: Arc<ConnectionRegistry>) -> Self {
        Self { connector, registry }
    }

    pub async fn diff_table_data(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<DataDiff, DbError> {
        let source_handle = self
            .registry
            .get(source_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {source_id} is not active")))?;
        let target_handle = self
            .registry
            .get(target_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {target_id} is not active")))?;

        let source_dialect = self.connector.dialect(&source_handle)?;
        let target_dialect = self.connector.dialect(&target_handle)?;

        let source_qualified = format!(
            "{}.{}",
            source_dialect.quote_identifier(schema),
            source_dialect.quote_identifier(table)
        );
        let target_qualified = format!(
            "{}.{}",
            target_dialect.quote_identifier(schema),
            target_dialect.quote_identifier(table)
        );

        let source_sql = format!("SELECT COUNT(*) FROM {source_qualified}");
        let target_sql = format!("SELECT COUNT(*) FROM {target_qualified}");

        let source_result = self.connector.query(&source_handle, &source_sql, &[]).await?;
        let target_result = self.connector.query(&target_handle, &target_sql, &[]).await?;

        let source_count = extract_count(&source_result)?;
        let target_count = extract_count(&target_result)?;

        Ok(DataDiff {
            schema: schema.to_string(),
            table: table.to_string(),
            source_row_count: source_count,
            target_row_count: target_count,
            row_count_diff: source_count - target_count,
        })
    }
}

fn extract_count(result: &crate::domain::query::QueryResult) -> Result<i64, DbError> {
    result
        .rows
        .first()
        .and_then(|row| row.0.first())
        .and_then(|cell| match cell {
            crate::domain::query::CellValue::Int64(n) => Some(*n),
            _ => None,
        })
        .ok_or_else(|| DbError::Internal("failed to extract row count".into()))
}
