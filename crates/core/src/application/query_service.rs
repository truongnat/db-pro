use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::{QueryHistory, SavedQuery, SavedQueryFolder};
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::run_config::RunConfig;
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::ports::{ConnectionRepository, DbConnector, QueryHistoryRepository, RunConfigRepository, SavedQueryRepository};

use super::registry::ConnectionRegistry;
use super::sql_policy::{reject_multi_statement, split_statements};

pub struct MultiQueryResult {
    pub results: Vec<QueryResult>,
    pub total_duration_ms: u64,
    /// If a statement failed, this holds the 0-based index and error message.
    /// Earlier results were committed; later statements were not executed.
    pub error: Option<(usize, String)>,
}

pub struct QueryService {
    connector: Box<dyn DbConnector>,
    history: Box<dyn QueryHistoryRepository>,
    saved_queries: Box<dyn SavedQueryRepository>,
    run_configs: Box<dyn RunConfigRepository>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl QueryService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        history: Box<dyn QueryHistoryRepository>,
        saved_queries: Box<dyn SavedQueryRepository>,
        run_configs: Box<dyn RunConfigRepository>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            history,
            saved_queries,
            run_configs,
            registry,
            connections,
        }
    }

    /// Build the safety policy for a connection based on its persisted config.
    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self.connections.get_config(connection_id).await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    pub async fn execute(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
        params: &[QueryParam],
        database: Option<&str>,
        schema: Option<&str>,
    ) -> Result<QueryResult, DbError> {
        reject_multi_statement(sql)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        // Enforce safety policy (readonly, etc.)
        let policy = self.safety_policy_for(connection_id).await?;
        validate_against_policy(sql, &policy).map_err(|e| DbError::QueryFailed(e))?;

        let result = self.connector.query(&handle, sql, params).await?;
        result.validate().map_err(DbError::QueryFailed)?;

        if let Err(e) = self.history.save(connection_id, sql, &result, database.map(str::to_owned), schema.map(str::to_owned)).await {
            tracing::warn!("failed to save query history: {e}");
        }

        Ok(result)
    }

    pub async fn execute_multi(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
        database: Option<&str>,
        schema: Option<&str>,
    ) -> Result<MultiQueryResult, DbError> {
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        // Enforce safety policy for each statement in the script.
        let policy = self.safety_policy_for(connection_id).await?;

        let statements = split_statements(sql);
        if statements.is_empty() {
            return Err(DbError::QueryFailed("empty SQL statement".into()));
        }

        let start = std::time::Instant::now();
        let mut results = Vec::with_capacity(statements.len());

        for (idx, stmt) in statements.iter().enumerate() {
            let stmt_start = std::time::Instant::now();

            // Validate each statement against the safety policy before execution.
            if let Err(msg) = validate_against_policy(stmt, &policy) {
                return Ok(MultiQueryResult {
                    results,
                    total_duration_ms: start.elapsed().as_millis() as u64,
                    error: Some((idx, msg)),
                });
            }

            match classify_statement(stmt) {
                StatementClass::Read => {
                    match self.connector.query(&handle, stmt, &[]).await {
                        Ok(result) => {
                            if let Err(e) = result.validate() {
                                return Ok(MultiQueryResult {
                                    results,
                                    total_duration_ms: start.elapsed().as_millis() as u64,
                                    error: Some((idx, e)),
                                });
                            }
                            results.push(result);
                        }
                        Err(e) => {
                            return Ok(MultiQueryResult {
                                results,
                                total_duration_ms: start.elapsed().as_millis() as u64,
                                error: Some((idx, e.to_string())),
                            });
                        }
                    }
                }
                StatementClass::Write => {
                    match self.connector.execute(&handle, stmt, &[]).await {
                        Ok(affected) => {
                            let elapsed = stmt_start.elapsed().as_millis() as u64;
                            results.push(QueryResult {
                                columns: Vec::new(),
                                rows: Vec::new(),
                                row_count: affected,
                                duration_ms: elapsed,
                            });
                        }
                        Err(e) => {
                            return Ok(MultiQueryResult {
                                results,
                                total_duration_ms: start.elapsed().as_millis() as u64,
                                error: Some((idx, e.to_string())),
                            });
                        }
                    }
                }
            }
        }

        let total_duration_ms = start.elapsed().as_millis() as u64;

        if let Some(first) = results.first() {
            if let Err(e) = self.history.save(connection_id, sql, first, database.map(str::to_owned), schema.map(str::to_owned)).await {
                tracing::warn!("failed to save query history: {e}");
            }
        }

        Ok(MultiQueryResult {
            results,
            total_duration_ms,
            error: None,
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

enum StatementClass {
    Read,
    Write,
}

fn classify_statement(sql: &str) -> StatementClass {
    let keyword = effective_keyword(sql);
    match keyword {
        Some(k) if is_read_keyword(&k) => StatementClass::Read,
        _ => StatementClass::Write,
    }
}

fn is_read_keyword(word: &str) -> bool {
    matches!(word, "SELECT" | "SHOW" | "EXPLAIN" | "TABLE")
}

/// Extracts the effective first keyword of a SQL statement, handling
/// leading comments and WITH...CTE chains. For `WITH cte AS (...) UPDATE ...`,
/// returns "UPDATE" (not "WITH").
fn effective_keyword(sql: &str) -> Option<String> {
    let trimmed = strip_leading_comments(sql).trim_start();
    let upper = trimmed.to_ascii_uppercase();

    let first = upper.split_whitespace().next()?;

    if first != "WITH" && !first.starts_with("WITH") {
        return Some(first.to_string());
    }

    // Check it's actually the keyword WITH (not WITHDRAWAL etc.)
    let after_with = &trimmed[4..];
    if !after_with.is_empty() && after_with.chars().next().unwrap().is_alphanumeric() {
        return Some(first.to_string());
    }

    // WITH statement: scan past CTE definitions to find the main keyword.
    // Track parenthesis depth and string literals; after the last CTE closes
    // at depth 0, the next keyword is the actual statement type.
    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();
    let mut i = 4; // skip "WITH"
    let mut depth: i32 = 0;
    let mut in_string = false;

    while i < len {
        if in_string {
            if chars[i] == '\'' {
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 1; // skip escaped quote
                } else {
                    in_string = false;
                }
            }
            i += 1;
            continue;
        }

        match chars[i] {
            '\'' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    // Skip whitespace after closing paren
                    i += 1;
                    while i < len && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < len && chars[i] == ',' {
                        // Another CTE follows — skip name + AS
                        i += 1;
                        continue;
                    }
                    // No comma: next word is the main statement keyword
                    let remaining: String = chars[i..].iter().collect();
                    return remaining
                        .trim_start()
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Fallback: couldn't parse past CTEs, treat WITH as read
    Some("WITH".to_string())
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if s.starts_with("--") {
            // Line comment: skip to end of line
            s = s.find('\n').map(|i| &s[i + 1..]).unwrap_or("").trim_start();
        } else if s.starts_with("/*") {
            // Block comment: skip to */
            s = s.find("*/").map(|i| &s[i + 2..]).unwrap_or("").trim_start();
        } else {
            break;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::{CellValue, ColumnMeta, Row};
    use crate::ports::{MockConnectionRepository, MockDbConnector, MockQueryHistoryRepository, MockRunConfigRepository, MockSavedQueryRepository};

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
            Box::new(MockConnectionRepository::new()),
        )
    }

    /// Create a mock ConnectionRepository that returns a non-readonly config for any connection.
    fn mock_connections_full_access() -> MockConnectionRepository {
        let mut repo = MockConnectionRepository::new();
        repo.expect_get_config().returning(|_id| {
            Ok(Some(crate::domain::connection::ConnectionConfig {
                name: "test".into(),
                host: "localhost".into(),
                port: 5432,
                database: "testdb".into(),
                username: "user".into(),
                driver: crate::domain::connection::DriverType::Postgres,
                ssl_mode: crate::domain::connection::SslMode::Disable,
                ssh_tunnel: None,
                query_timeout_ms: 30_000,
                max_rows: 500,
                color: None,
                tags: vec![],
                group: None,
                readonly: false,
            }))
        });
        repo
    }

    #[tokio::test]
    async fn execute_success() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(test_result()));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute(&conn_id, "SELECT 1", &[], None, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().row_count, 1);
    }

    #[tokio::test]
    async fn execute_not_active() {
        let svc = build_service(MockDbConnector::new(), Arc::new(ConnectionRegistry::new()));
        let result = svc.execute(&ConnectionId::new(), "SELECT 1", &[], None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_multi_statement_rejected() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let svc = build_service(MockDbConnector::new(), Arc::clone(&registry));
        let result = svc.execute(&conn_id, "SELECT 1; SELECT 2", &[], None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_empty_sql_rejected() {
        let svc = build_service(MockDbConnector::new(), Arc::new(ConnectionRegistry::new()));
        let result = svc.execute(&ConnectionId::new(), "  ", &[], None, None).await;
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
        history.expect_save().returning(|_, _, _, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute(&conn_id, "SELECT 1;", &[], None, None).await;
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
        history.expect_save().returning(|_, _, _, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute(&conn_id, "SELECT ';' FROM t", &[], None, None).await;
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
            .returning(|_, _, _, _, _| Err(DbError::Internal("disk full".into())));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute(&conn_id, "SELECT 1", &[], None, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn classify_simple_select_is_read() {
        assert!(matches!(classify_statement("SELECT 1"), StatementClass::Read));
    }

    #[test]
    fn classify_simple_insert_is_write() {
        assert!(matches!(classify_statement("INSERT INTO t VALUES (1)"), StatementClass::Write));
    }

    #[test]
    fn classify_with_select_is_read() {
        let sql = "WITH cte AS (SELECT id FROM t) SELECT * FROM cte";
        assert!(matches!(classify_statement(sql), StatementClass::Read));
    }

    #[test]
    fn classify_with_update_is_write() {
        let sql = "WITH changed AS (SELECT id FROM t) UPDATE t SET x = 1 WHERE id IN (SELECT id FROM changed)";
        assert!(matches!(classify_statement(sql), StatementClass::Write));
    }

    #[test]
    fn classify_with_delete_is_write() {
        let sql = "WITH deleted AS (SELECT id FROM t) DELETE FROM t WHERE id IN (SELECT id FROM deleted)";
        assert!(matches!(classify_statement(sql), StatementClass::Write));
    }

    #[test]
    fn classify_with_multiple_ctes_update_is_write() {
        let sql = "WITH a AS (SELECT 1), b AS (SELECT 2) UPDATE t SET x = 1";
        assert!(matches!(classify_statement(sql), StatementClass::Write));
    }

    #[test]
    fn classify_leading_comment_stripped() {
        let sql = "-- comment\nSELECT 1";
        assert!(matches!(classify_statement(sql), StatementClass::Read));
    }

    #[test]
    fn classify_leading_block_comment_stripped() {
        let sql = "/* block */ INSERT INTO t VALUES (1)";
        assert!(matches!(classify_statement(sql), StatementClass::Write));
    }

    #[test]
    fn classify_with_paren_in_string_literal() {
        let sql = "WITH cte AS (SELECT 'value )' AS text) UPDATE t SET x = 1";
        assert!(matches!(classify_statement(sql), StatementClass::Write));
    }

    #[test]
    fn classify_with_escaped_quote_in_string() {
        let sql = "WITH cte AS (SELECT 'it''s )' AS text) SELECT * FROM cte";
        assert!(matches!(classify_statement(sql), StatementClass::Read));
    }

    #[tokio::test]
    async fn execute_multi_routes_select_then_update() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query()
            .withf(|_, sql, _| sql == "SELECT 1")
            .returning(|_, _, _| Ok(test_result()));
        connector.expect_execute()
            .withf(|_, sql, _| sql == "UPDATE t SET x = 1")
            .returning(|_, _, _| Ok(3));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute_multi(&conn_id, "SELECT 1; UPDATE t SET x = 1", None, None).await.unwrap();
        assert!(result.error.is_none());
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].row_count, 1);
        assert_eq!(result.results[1].row_count, 3);
    }

    #[tokio::test]
    async fn execute_multi_with_update_routes_to_execute() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_execute()
            .returning(|_, _, _| Ok(5));

        let mut history = MockQueryHistoryRepository::new();
        history.expect_save().returning(|_, _, _, _, _| Ok(()));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(history),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute_multi(
            &conn_id,
            "WITH cte AS (SELECT id FROM t) UPDATE t SET x = 1",
            None, None,
        ).await.unwrap();
        assert!(result.error.is_none());
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].row_count, 5);
    }

    #[tokio::test]
    async fn execute_multi_partial_failure_preserves_earlier_results() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query()
            .returning(|_, _, _| Ok(test_result()));
        connector.expect_execute()
            .returning(|_, _, _| Err(DbError::QueryFailed("permission denied".into())));

        let svc = QueryService::new(
            Box::new(connector),
            Box::new(MockQueryHistoryRepository::new()),
            Box::new(MockSavedQueryRepository::new()),
            Box::new(MockRunConfigRepository::new()),
            Arc::clone(&registry),
            Box::new(mock_connections_full_access()),
        );

        let result = svc.execute_multi(
            &conn_id,
            "SELECT 1; UPDATE t SET x = 1; SELECT 2",
            None, None,
        ).await.unwrap();

        assert!(result.error.is_some());
        let (idx, msg) = result.error.unwrap();
        assert_eq!(idx, 1);
        assert!(msg.contains("permission denied"));
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].row_count, 1);
    }
}
