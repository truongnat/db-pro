use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::{QueryHistory, SavedQuery};
use crate::domain::query::{QueryParam, QueryResult};
use crate::ports::{DbConnector, QueryHistoryRepository, SavedQueryRepository};

use super::registry::ConnectionRegistry;

pub struct QueryService {
    connector: Box<dyn DbConnector>,
    history: Box<dyn QueryHistoryRepository>,
    saved_queries: Box<dyn SavedQueryRepository>,
    registry: Arc<ConnectionRegistry>,
}

impl QueryService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        history: Box<dyn QueryHistoryRepository>,
        saved_queries: Box<dyn SavedQueryRepository>,
        registry: Arc<ConnectionRegistry>,
    ) -> Self {
        Self {
            connector,
            history,
            saved_queries,
            registry,
        }
    }

    pub async fn execute(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<QueryResult, DbError> {
        reject_multi_statement(sql)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let result = self.connector.query(&handle, sql, params).await?;
        result.validate().map_err(DbError::QueryFailed)?;

        if let Err(e) = self.history.save(connection_id, sql, &result).await {
            tracing::warn!("failed to save query history: {e}");
        }

        Ok(result)
    }

    pub async fn explain(&self, connection_id: &ConnectionId, sql: &str) -> Result<serde_json::Value, DbError> {
        reject_multi_statement(sql)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        self.connector.explain(&handle, sql).await
    }

    pub async fn get_history(&self, connection_id: &ConnectionId, limit: u32) -> Result<Vec<QueryHistory>, DbError> {
        self.history.list(connection_id, limit).await
    }

    pub async fn save_query(
        &self,
        connection_id: &ConnectionId,
        name: &str,
        sql: &str,
        folder: Option<&str>,
    ) -> Result<SavedQuery, DbError> {
        let query = SavedQuery {
            id: uuid::Uuid::new_v4(),
            connection_id: *connection_id,
            name: name.to_string(),
            sql: sql.to_string(),
            folder: folder.map(String::from),
            created_at: chrono::Utc::now(),
        };
        self.saved_queries.save(&query).await?;
        Ok(query)
    }

    pub async fn list_saved_queries(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError> {
        self.saved_queries.list(connection_id).await
    }

    pub async fn delete_saved_query(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.saved_queries.delete(id).await
    }
}

fn reject_multi_statement(sql: &str) -> Result<(), DbError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(DbError::QueryFailed("empty SQL statement".into()));
    }

    let trimmed = trimmed.trim_end_matches(';');
    let stripped = strip_single_quoted_strings(trimmed);
    if stripped.contains(';') {
        return Err(DbError::QueryFailed("multi-statement execution is disabled".into()));
    }

    Ok(())
}

fn strip_single_quoted_strings(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            loop {
                match chars.next() {
                    Some('\'') => {
                        if chars.peek() == Some(&'\'') {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::{CellValue, ColumnMeta, Row};
    use crate::ports::{MockDbConnector, MockQueryHistoryRepository, MockSavedQueryRepository};

    fn test_result() -> QueryResult {
        QueryResult {
            columns: vec![ColumnMeta {
                name: "id".into(),
                data_type: "INT".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Int64(1)])],
            row_count: 1,
            duration_ms: 0,
        }
    }

    fn build_service(connector: MockDbConnector, registry: Arc<ConnectionRegistry>) -> QueryService {
        QueryService::new(
            Box::new(connector),
            Box::new(MockQueryHistoryRepository::new()),
            Box::new(MockSavedQueryRepository::new()),
            registry,
        )
    }

    #[tokio::test]
    async fn execute_success() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(test_result()));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Arc::clone(&registry),
        );

        let result = svc.execute(&conn_id, "SELECT 1", &[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().row_count, 1);
    }

    #[tokio::test]
    async fn execute_not_active() {
        let svc = build_service(MockDbConnector::new(), Arc::new(ConnectionRegistry::new()));
        let result = svc.execute(&ConnectionId::new(), "SELECT 1", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_multi_statement_rejected() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let svc = build_service(MockDbConnector::new(), Arc::clone(&registry));
        let result = svc.execute(&conn_id, "SELECT 1; SELECT 2", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_empty_sql_rejected() {
        let svc = build_service(MockDbConnector::new(), Arc::new(ConnectionRegistry::new()));
        let result = svc.execute(&ConnectionId::new(), "  ", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_trailing_semicolon_ok() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(test_result()));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Arc::clone(&registry),
        );

        let result = svc.execute(&conn_id, "SELECT 1;", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn execute_semicolon_in_string_ok() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(test_result()));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Arc::clone(&registry),
        );

        let result = svc.execute(&conn_id, "SELECT ';' FROM t", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn execute_history_save_failure_non_fatal() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(test_result()));

        let mut history = MockQueryHistoryRepository::new();
        history
            .expect_save()
            .returning(|_, _, _| Err(DbError::Internal("disk full".into())));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Arc::clone(&registry),
        );

        let result = svc.execute(&conn_id, "SELECT 1", &[]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn reject_multi_statement_basic() {
        assert!(reject_multi_statement("SELECT 1").is_ok());
        assert!(reject_multi_statement("SELECT 1; SELECT 2").is_err());
        assert!(reject_multi_statement("  ").is_err());
        assert!(reject_multi_statement("SELECT 1;").is_ok());
    }

    #[test]
    fn strip_quoted_strings_handles_escapes() {
        assert_eq!(strip_single_quoted_strings("SELECT 'it''s'"), "SELECT ");
        assert_eq!(strip_single_quoted_strings("a;b'c;d'e"), "a;be");
    }
}
