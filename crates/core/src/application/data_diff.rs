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

        let source_qualified = if schema.is_empty() {
            source_dialect.quote_identifier(table)
        } else {
            format!(
                "{}.{}",
                source_dialect.quote_identifier(schema),
                source_dialect.quote_identifier(table)
            )
        };
        let target_qualified = if schema.is_empty() {
            target_dialect.quote_identifier(table)
        } else {
            format!(
                "{}.{}",
                target_dialect.quote_identifier(schema),
                target_dialect.quote_identifier(table)
            )
        };

        let source_sql = format!("SELECT COUNT(*) FROM {source_qualified}");
        let target_sql = format!("SELECT COUNT(*) FROM {target_qualified}");

        let source_result = self.connector.query(&source_handle, &source_sql, &[]).await?;
        let target_result = self.connector.query(&target_handle, &target_sql, &[]).await?;

        let source_count = extract_count(&source_result)?;
        let target_count = extract_count(&target_result)?;
        let row_count_diff = row_count_difference(source_count, target_count)?;

        Ok(DataDiff {
            schema: schema.to_string(),
            table: table.to_string(),
            source_row_count: source_count,
            target_row_count: target_count,
            row_count_diff,
        })
    }
}

fn extract_count(result: &crate::domain::query::QueryResult) -> Result<i64, DbError> {
    let count = result
        .rows
        .first()
        .and_then(|row| row.0.first())
        .and_then(|cell| match cell {
            crate::domain::query::CellValue::Int64(n) => Some(*n),
            _ => None,
        })
        .ok_or_else(|| DbError::Internal("failed to extract row count".into()))?;

    if count < 0 {
        return Err(DbError::Internal("row count cannot be negative".into()));
    }

    Ok(count)
}

fn row_count_difference(source: i64, target: i64) -> Result<i64, DbError> {
    source
        .checked_sub(target)
        .or_else(|| target.checked_sub(source).and_then(i64::checked_neg))
        .ok_or_else(|| DbError::Internal("row count difference overflowed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::query::{CellValue, QueryResult, Row};

    #[test]
    fn extract_count_rejects_negative_provider_value() {
        let result = QueryResult {
            columns: vec![],
            rows: vec![Row(vec![CellValue::Int64(-1)])],
            row_count: 1,
            duration_ms: 0,
        };

        assert!(matches!(
            extract_count(&result),
            Err(DbError::Internal(message)) if message.contains("negative")
        ));
    }

    #[test]
    fn row_count_difference_handles_both_directions() {
        assert_eq!(row_count_difference(10, 3).unwrap(), 7);
        assert_eq!(row_count_difference(3, 10).unwrap(), -7);
        assert!(matches!(
            row_count_difference(i64::MAX, i64::MIN),
            Err(DbError::Internal(message)) if message.contains("overflowed")
        ));
    }
}
