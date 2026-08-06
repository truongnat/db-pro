use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::{QueryHistory, SavedQuery, SavedQueryFolder};
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::run_config::RunConfig;
use crate::ports::{DbConnector, QueryHistoryRepository, RunConfigRepository, SavedQueryRepository};

use super::registry::ConnectionRegistry;
use super::sql_policy::{reject_multi_statement, split_statements};

pub struct MultiQueryResult {
    pub results: Vec<QueryResult>,
    pub total_duration_ms: u64,
}

pub struct QueryService {
    connector: Box<dyn DbConnector>,
    history: Box<dyn QueryHistoryRepository>,
    saved_queries: Box<dyn SavedQueryRepository>,
    run_configs: Box<dyn RunConfigRepository>,
    registry: Arc<ConnectionRegistry>,
}

impl QueryService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        history: Box<dyn QueryHistoryRepository>,
        saved_queries: Box<dyn SavedQueryRepository>,
        run_configs: Box<dyn RunConfigRepository>,
        registry: Arc<ConnectionRegistry>,
    ) -> Self {
        Self {
            connector,
            history,
            saved_queries,
            run_configs,
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

    pub async fn execute_multi(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
    ) -> Result<MultiQueryResult, DbError> {
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let statements = split_statements(sql);
        if statements.is_empty() {
            return Err(DbError::QueryFailed("empty SQL statement".into()));
        }

        let start = std::time::Instant::now();
        let mut results = Vec::with_capacity(statements.len());

        for stmt in &statements {
            let result = self.connector.query(&handle, stmt, &[]).await?;
            result.validate().map_err(DbError::QueryFailed)?;
            results.push(result);
        }

        let total_duration_ms = start.elapsed().as_millis() as u64;

        if let Err(e) = self.history.save(connection_id, sql, results.first().unwrap()).await {
            tracing::warn!("failed to save query history: {e}");
        }

        Ok(MultiQueryResult {
            results,
            total_duration_ms,
        })
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

    pub async fn create_folder(
        &self,
        connection_id: &ConnectionId,
        name: &str,
    ) -> Result<SavedQueryFolder, DbError> {
        let folder = SavedQueryFolder {
            id: uuid::Uuid::new_v4(),
            connection_id: *connection_id,
            name: name.to_string(),
            created_at: chrono::Utc::now(),
        };
        self.saved_queries.create_folder(&folder).await?;
        Ok(folder)
    }

    pub async fn list_folders(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQueryFolder>, DbError> {
        self.saved_queries.list_folders(connection_id).await
    }

    pub async fn delete_folder(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.saved_queries.delete_folder(id).await
    }

    pub async fn save_run_config(
        &self,
        connection_id: &ConnectionId,
        name: &str,
        sql: &str,
        timeout_ms: u64,
        max_rows: u64,
    ) -> Result<RunConfig, DbError> {
        let config = RunConfig {
            id: uuid::Uuid::new_v4(),
            connection_id: *connection_id,
            name: name.to_string(),
            sql: sql.to_string(),
            timeout_ms,
            max_rows,
            created_at: chrono::Utc::now(),
        };
        self.run_configs.save(&config).await?;
        Ok(config)
    }

    pub async fn list_run_configs(&self, connection_id: &ConnectionId) -> Result<Vec<RunConfig>, DbError> {
        self.run_configs.list(connection_id).await
    }

    pub async fn delete_run_config(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.run_configs.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::{CellValue, ColumnMeta, Row};
    use crate::ports::{MockDbConnector, MockQueryHistoryRepository, MockRunConfigRepository, MockSavedQueryRepository};

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
            Box::new(MockRunConfigRepository::new()),
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
            Box::new(MockRunConfigRepository::new()),
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
            Box::new(MockRunConfigRepository::new()),
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
            Box::new(MockRunConfigRepository::new()),
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
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
        );

        let result = svc.execute(&conn_id, "SELECT 1", &[]).await;
        assert!(result.is_ok());
    }
}
