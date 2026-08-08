use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::QueryHistory;
use db_pro_core::domain::query::QueryResult;
use db_pro_core::ports::QueryHistoryRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl QueryHistoryRepository for SQLiteMetaStore {
    async fn save(&self, connection_id: &ConnectionId, sql: &str, result: &QueryResult, database: Option<String>, schema: Option<String>) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let executed_at = chrono::Utc::now().to_rfc3339();
        self.actor
            .raw_query(
                "INSERT INTO query_history (id, connection_id, sql, executed_at, duration_ms, row_count, database, schema) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)".into(),
                vec![
                    id,
                    connection_id.to_string(),
                    sql.into(),
                    executed_at,
                    result.duration_ms.to_string(),
                    result.row_count.to_string(),
                    database.unwrap_or_default().into(),
                    schema.unwrap_or_default().into(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn list(&self, connection_id: &ConnectionId, limit: u32) -> Result<Vec<QueryHistory>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT id, connection_id, sql, executed_at, duration_ms, row_count, database, schema FROM query_history WHERE connection_id = ?1 ORDER BY executed_at DESC LIMIT ?2".into(),
                vec![connection_id.to_string(), limit.to_string()],
            )
            .await?;
        let mut history = Vec::new();
        for row in rows {
            history.push(QueryHistory {
                id: uuid::Uuid::parse_str(&row[0]).map_err(|e| DbError::Internal(format!("invalid uuid: {e}")))?,
                connection_id: ConnectionId::parse(&row[1])
                    .map_err(|e| DbError::Internal(format!("invalid connection id: {e}")))?,
                sql: row[2].clone(),
                executed_at: chrono::DateTime::parse_from_rfc3339(&row[3])
                    .map_err(|e| DbError::Internal(format!("invalid datetime: {e}")))?
                    .with_timezone(&chrono::Utc),
                duration_ms: row[4].parse().unwrap_or(0),
                row_count: row[5].parse().unwrap_or(0),
                database: if row[6].is_empty() { None } else { Some(row[6].clone()) },
                schema: if row[7].is_empty() { None } else { Some(row[7].clone()) },
            });
        }
        Ok(history)
    }
}
