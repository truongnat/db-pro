//! Tests for `../monitoring.rs` (moved out of the source file).
#![allow(clippy::result_large_err)] // DbError / TransactionFailure mirror the trait's large Err.
use super::PostgresMonitoringPort;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam, QueryResult, Row};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{
    DbConnector, MonitoringPort, ParameterizedTransactionStatement, SqlDialect, TransactionFailure,
    TransactionStatementResult,
};
use std::sync::Arc;

mockall::mock! {
    pub Connector {}
    #[async_trait::async_trait]
    impl DbConnector for Connector {
        async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError>;
        async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError>;
        async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError>;
        async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError>;
        async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError>;
        async fn cancel(&self, handle: &ConnectionHandle) -> Result<(), DbError>;
        async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError>;
        async fn execute_transaction(
            &self,
            handle: &ConnectionHandle,
            statements: &[String],
            read_statements: &[bool],
        ) -> Result<Vec<TransactionStatementResult>, TransactionFailure>;
        async fn execute_parameterized_transaction(
            &self,
            handle: &ConnectionHandle,
            statements: &[ParameterizedTransactionStatement],
        ) -> Result<Vec<TransactionStatementResult>, TransactionFailure>;
        async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError>;
        async fn explain(
            &self,
            handle: &ConnectionHandle,
            sql: &str,
            params: &[QueryParam],
            analyze: bool,
        ) -> Result<serde_json::Value, DbError>;
        fn dialect(&self, handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError>;
    }
}

fn matching_row() -> QueryResult {
    QueryResult {
        columns: Vec::new(),
        rows: vec![Row(vec![CellValue::Bool(true)])],
        row_count: 1,
        duration_ms: 0,
    }
}

#[tokio::test]
async fn cancel_verifies_pid_and_backend_start_in_one_statement() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector.expect_query().times(1).returning(|_, sql, params| {
        assert!(sql.contains("pg_cancel_backend"), "{sql}");
        assert!(sql.contains("pg_catalog.pg_stat_activity"), "{sql}");
        assert!(sql.contains("a.pid = $1"), "{sql}");
        assert!(sql.contains("a.backend_start = $2::timestamptz"), "{sql}");
        assert!(matches!(params.first(), Some(QueryParam::Int64(42))));
        assert!(matches!(
            params.get(1),
            Some(QueryParam::Text(identity)) if identity == "2024-01-01 00:00:00+00"
        ));
        Ok(matching_row())
    });

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    assert!(port
        .cancel_backend(&handle, 42, "2024-01-01 00:00:00+00")
        .await
        .unwrap());
}

#[tokio::test]
async fn cancel_rejects_empty_identity_without_querying() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector.expect_query().times(0);

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    let err = port.cancel_backend(&handle, 42, "  ").await.unwrap_err();
    assert!(err.to_string().contains("backend_start"));
}

#[tokio::test]
async fn terminate_verifies_pid_and_backend_start_in_one_statement() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector.expect_query().times(1).returning(|_, sql, params| {
        assert!(sql.contains("pg_terminate_backend"), "{sql}");
        assert!(sql.contains("pg_catalog.pg_stat_activity"), "{sql}");
        assert!(sql.contains("a.pid = $1"), "{sql}");
        assert!(sql.contains("a.backend_start = $2::timestamptz"), "{sql}");
        assert!(matches!(params.first(), Some(QueryParam::Int64(42))));
        assert!(matches!(
            params.get(1),
            Some(QueryParam::Text(identity)) if identity == "2024-01-01 00:00:00+00"
        ));
        Ok(matching_row())
    });

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    assert!(port
        .terminate_backend(&handle, 42, "2024-01-01 00:00:00+00")
        .await
        .unwrap());
}

#[tokio::test]
async fn terminate_returns_false_when_identity_no_longer_matches() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector
        .expect_query()
        .times(1)
        .returning(|_, _, _| Ok(QueryResult::empty()));

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    assert!(!port.terminate_backend(&handle, 42, "stale").await.unwrap());
}

#[tokio::test]
async fn terminate_rejects_empty_identity_without_querying() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector.expect_query().times(0);

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    let err = port.terminate_backend(&handle, 42, "  ").await.unwrap_err();
    assert!(err.to_string().contains("backend_start"));
}

#[tokio::test]
async fn list_locks_uses_postgres_blocking_pids_and_preserves_blocker_ids() {
    let handle = ConnectionHandle::new(1);
    let mut connector = MockConnector::new();
    connector.expect_query().times(1).returning(|_, sql, params| {
        assert!(params.is_empty());
        assert!(sql.contains("pg_catalog.pg_blocking_pids(a.pid)"), "{sql}");
        assert!(!sql.contains("JOIN pg_catalog.pg_locks blocking"), "{sql}");
        Ok(QueryResult {
            columns: Vec::new(),
            rows: vec![
                Row(vec![
                    CellValue::Int64(42),
                    CellValue::Int64(77),
                    CellValue::Text("orders".into()),
                    CellValue::Text("relation".into()),
                    CellValue::Text("AccessExclusiveLock".into()),
                    CellValue::Bool(false),
                    CellValue::Int64(1200),
                ]),
                Row(vec![
                    CellValue::Int64(42),
                    CellValue::Int64(88),
                    CellValue::Text("orders".into()),
                    CellValue::Text("relation".into()),
                    CellValue::Text("AccessExclusiveLock".into()),
                    CellValue::Bool(false),
                    CellValue::Int64(1200),
                ]),
            ],
            row_count: 2,
            duration_ms: 0,
        })
    });

    let port = PostgresMonitoringPort::new(Arc::new(connector) as Arc<dyn DbConnector>);
    let locks = port.list_locks(&handle).await.unwrap();
    assert_eq!(locks.len(), 2);
    assert_eq!(locks[0].locked_pid, 42);
    assert_eq!(locks[0].blocker_pid, Some(77));
    assert_eq!(locks[1].blocker_pid, Some(88));
}
