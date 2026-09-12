use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::domain::safety::ConnectionSafetyPolicy;
use crate::ports::{ConnectionRepository, DbConnector};

use super::registry::ConnectionRegistry;
use super::sql_builder::{self, SortClause, TableFilter};

pub struct TableDataService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl TableDataService {
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

    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn fetch_rows(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
        sorts: &[SortClause],
        limit: u64,
        offset: u64,
    ) -> Result<(QueryResult, u64), DbError> {
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;

        let (select_sql, select_params) =
            sql_builder::build_select(dialect.as_ref(), schema, table, filters, sorts, limit, offset)?;
        let (count_sql, count_params) = sql_builder::build_count(dialect.as_ref(), schema, table, filters);

        let count_result = self.connector.query(&handle, &count_sql, &count_params).await?;
        count_result.validate().map_err(DbError::QueryFailed)?;
        let data_result = self.connector.query(&handle, &select_sql, &select_params).await?;
        data_result.validate().map_err(DbError::QueryFailed)?;

        let total_count = parse_total_count(&count_result)?;

        let duration_ms = data_result.duration_ms;
        Ok((
            QueryResult {
                duration_ms,
                ..data_result
            },
            total_count,
        ))
    }

    pub async fn insert_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
    ) -> Result<u64, DbError> {
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot insert rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) = sql_builder::build_insert(dialect.as_ref(), schema, table, columns, values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        if affected_rows == 0 {
            return Err(DbError::NotFound(format!("row not found in {schema}.{table}")));
        }
        Ok(affected_rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation("update requires a primary key".into()));
        }
        if columns.iter().any(|column| pk_columns.iter().any(|pk| pk == column)) {
            return Err(DbError::Validation(
                "updating primary-key columns is not supported by the row mutation contract".into(),
            ));
        }
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot update rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) =
            sql_builder::build_update(dialect.as_ref(), schema, table, columns, values, pk_columns, pk_values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        if affected_rows == 0 {
            return Err(DbError::NotFound(format!("row not found in {schema}.{table}")));
        }
        Ok(affected_rows)
    }

    pub async fn delete_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation("delete requires a primary key".into()));
        }
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only; cannot delete rows".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let dialect = self.connector.dialect(&handle)?;
        let (sql, params) = sql_builder::build_delete(dialect.as_ref(), schema, table, pk_columns, pk_values)?;
        let affected_rows = self.connector.execute(&handle, &sql, &params).await?;
        if affected_rows == 0 {
            return Err(DbError::NotFound(format!("row not found in {schema}.{table}")));
        }
        Ok(affected_rows)
    }

    fn resolve_handle(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<crate::domain::connection::ConnectionHandle, DbError> {
        self.registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))
    }
}

fn parse_total_count(result: &QueryResult) -> Result<u64, DbError> {
    if result.columns.len() != 1 || result.rows.len() != 1 || result.row_count != 1 {
        return Err(DbError::Internal(
            "count query must return exactly one row and one column".into(),
        ));
    }

    let cell = result.rows[0]
        .0
        .first()
        .ok_or_else(|| DbError::Internal("count query returned no value".into()))?;

    match cell {
        CellValue::Int64(value) => {
            u64::try_from(*value).map_err(|_| DbError::Internal("count query returned a negative value".into()))
        }
        _ => Err(DbError::Internal("count query returned a non-integer value".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::*;
    use crate::ports::dialect::SqlDialect;
    use crate::ports::{MockConnectionRepository, MockDbConnector};

    struct QuestionDialect;
    impl SqlDialect for QuestionDialect {
        fn placeholder(&self, _index: usize) -> String {
            "?".to_string()
        }
        fn quote_identifier(&self, name: &str) -> String {
            let escaped = name.replace('"', "\"\"");
            format!("\"{escaped}\"")
        }
    }

    fn setup() -> (ConnectionId, Arc<ConnectionRegistry>) {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));
        (conn_id, registry)
    }

    fn mock_connections() -> MockConnectionRepository {
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

    #[test]
    fn parse_total_count_rejects_negative_values() {
        let result = QueryResult {
            columns: vec![ColumnMeta {
                name: "count".into(),
                data_type: "INT".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Int64(-1)])],
            row_count: 1,
            duration_ms: 0,
        };

        assert!(matches!(
            parse_total_count(&result),
            Err(DbError::Internal(message)) if message.contains("negative")
        ));
    }

    #[test]
    fn parse_total_count_rejects_missing_or_wrong_type() {
        let empty = QueryResult::empty();
        assert!(matches!(
            parse_total_count(&empty),
            Err(DbError::Internal(message)) if message.contains("exactly one row")
        ));

        let wrong_type = QueryResult {
            columns: vec![ColumnMeta {
                name: "count".into(),
                data_type: "TEXT".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Text("42".into())])],
            row_count: 1,
            duration_ms: 0,
        };
        assert!(matches!(
            parse_total_count(&wrong_type),
            Err(DbError::Internal(message)) if message.contains("non-integer")
        ));
    }

    #[test]
    fn parse_total_count_rejects_non_scalar_result() {
        let result = QueryResult {
            columns: vec![ColumnMeta {
                name: "count".into(),
                data_type: "INT".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Int64(42)]), Row(vec![CellValue::Int64(43)])],
            row_count: 2,
            duration_ms: 0,
        };

        assert!(matches!(
            parse_total_count(&result),
            Err(DbError::Internal(message)) if message.contains("exactly one row")
        ));
    }

    #[tokio::test]
    async fn fetch_rows_returns_data_and_count() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_query().times(2).returning(|_handle, sql, _params| {
            if sql.contains("COUNT(*)") {
                Ok(QueryResult {
                    columns: vec![ColumnMeta {
                        name: "count".into(),
                        data_type: "INT".into(),
                        nullable: false,
                    }],
                    rows: vec![Row(vec![CellValue::Int64(42)])],
                    row_count: 1,
                    duration_ms: 1,
                })
            } else {
                Ok(QueryResult {
                    columns: vec![ColumnMeta {
                        name: "id".into(),
                        data_type: "INT".into(),
                        nullable: false,
                    }],
                    rows: vec![Row(vec![CellValue::Int64(1)])],
                    row_count: 1,
                    duration_ms: 2,
                })
            }
        });

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let (result, total) = svc
            .fetch_rows(&conn_id, "public", "users", &[], &[], 50, 0)
            .await
            .unwrap();
        assert_eq!(total, 42);
        assert_eq!(result.rows.len(), 1);
    }

    #[tokio::test]
    async fn fetch_rows_rejects_malformed_data_result_shape() {
        let (conn_id, registry) = setup();
        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_query().times(2).returning(|_, sql, _| {
            if sql.contains("COUNT(*)") {
                Ok(QueryResult {
                    columns: vec![ColumnMeta {
                        name: "count".into(),
                        data_type: "INT".into(),
                        nullable: false,
                    }],
                    rows: vec![Row(vec![CellValue::Int64(1)])],
                    row_count: 1,
                    duration_ms: 0,
                })
            } else {
                Ok(QueryResult {
                    columns: vec![ColumnMeta {
                        name: "id".into(),
                        data_type: "INT".into(),
                        nullable: false,
                    }],
                    rows: vec![Row(Vec::new())],
                    row_count: 1,
                    duration_ms: 0,
                })
            }
        });

        let service = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let error = service
            .fetch_rows(&conn_id, "public", "users", &[], &[], 50, 0)
            .await
            .expect_err("table data must reject an invalid provider result");

        assert!(matches!(error, DbError::QueryFailed(message) if message.contains("expected 1")));
    }

    #[tokio::test]
    async fn insert_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_execute().returning(|_, sql, _| {
            assert!(sql.contains("INSERT INTO"));
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let affected = svc
            .insert_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("alice".into())],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn update_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_execute().returning(|_, sql, params| {
            assert!(sql.contains("UPDATE"));
            assert!(sql.contains("WHERE"));
            assert_eq!(params.len(), 2);
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let affected = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("bob".into())],
                &["id".into()],
                &[CellValue::Int64(1)],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn delete_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_execute().returning(|_, sql, params| {
            assert!(sql.contains("DELETE FROM"));
            assert_eq!(params.len(), 1);
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let affected = svc
            .delete_row(&conn_id, "public", "users", &["id".into()], &[CellValue::Int64(1)])
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn update_row_returns_not_found_when_no_row_is_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_execute().returning(|_, _, _| Ok(0));

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let error = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("bob".into())],
                &["id".into()],
                &[CellValue::Int64(999)],
            )
            .await
            .expect_err("zero-row update must not be reported as success");

        assert!(matches!(error, DbError::NotFound(_)));
    }

    #[tokio::test]
    async fn delete_row_returns_not_found_when_no_row_is_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Ok(Box::new(QuestionDialect) as Box<dyn SqlDialect>));
        connector.expect_execute().returning(|_, _, _| Ok(0));

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let error = svc
            .delete_row(&conn_id, "public", "users", &["id".into()], &[CellValue::Int64(999)])
            .await
            .expect_err("zero-row delete must not be reported as success");

        assert!(matches!(error, DbError::NotFound(_)));
    }

    #[tokio::test]
    async fn fetch_rows_connection_not_active() {
        let registry = Arc::new(ConnectionRegistry::new());
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let fake_id = ConnectionId::new();
        let result = svc.fetch_rows(&fake_id, "public", "users", &[], &[], 50, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_row_rejects_empty_pk() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("x".into())],
                &[],
                &[],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn update_row_rejects_primary_key_column() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["id".into()],
                &[CellValue::Int64(2)],
                &["id".into()],
                &[CellValue::Int64(1)],
            )
            .await;

        assert!(matches!(result, Err(DbError::Validation(message)) if message.contains("primary-key")));
    }

    #[tokio::test]
    async fn delete_row_rejects_empty_pk() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc.delete_row(&conn_id, "public", "users", &[], &[]).await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn update_row_rejects_pk_length_mismatch() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("x".into())],
                &["id".into(), "org_id".into()],
                &[CellValue::Int64(1)],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_row_rejects_pk_length_mismatch() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc
            .delete_row(
                &conn_id,
                "public",
                "users",
                &["id".into(), "org_id".into()],
                &[CellValue::Int64(1)],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn fetch_rows_fails_when_dialect_errors() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector
            .expect_dialect()
            .returning(|_| Err(DbError::ConnectionFailed("unknown handle".into())));

        let svc = TableDataService::new(Box::new(connector), registry, Box::new(mock_connections()));
        let result = svc.fetch_rows(&conn_id, "public", "users", &[], &[], 50, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_row_blocked_on_readonly() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
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
                readonly: true,
            }))
        });
        let svc = TableDataService::new(Box::new(connector), registry, Box::new(repo));
        let result = svc
            .insert_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("alice".into())],
            )
            .await;
        assert!(result.is_err());
    }
}
