use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::connection::{ConnectionConfig, ConnectionHandle};
use crate::domain::error::DbError;
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::schema::IntrospectResult;
use crate::ports::dialect::SqlDialect;

#[derive(Debug)]
pub enum TransactionStatementResult {
    Query(QueryResult),
    Affected { row_count: u64, duration_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionFailurePhase {
    Validation,
    Begin,
    Statement,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionFailureOutcome {
    NotStarted,
    RolledBack,
    Unknown,
}

#[derive(Debug)]
pub struct TransactionFailure {
    pub phase: TransactionFailurePhase,
    /// The failed statement index for `Statement`; `statements.len()` is the
    /// transaction-level sentinel for validation, begin, and commit failures.
    pub statement_index: usize,
    pub outcome: TransactionFailureOutcome,
    pub results: Vec<TransactionStatementResult>,
    pub error: DbError,
}

#[cfg_attr(test, mockall::automock)]
#[allow(clippy::result_large_err)]
#[async_trait]
pub trait DbConnector: Send + Sync {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError>;

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError>;

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError>;

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError>;

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError>;

    /// Execute multiple SQL statements atomically inside a single transaction.
    /// If any statement fails, all changes are rolled back.
    async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError>;

    /// Execute read and write statements on one transaction. Implementations
    /// must report whether rollback was confirmed before returning
    /// `TransactionFailure`. A commit failure has an unknown final outcome.
    async fn execute_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure>;

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError>;

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError>;

    fn dialect(&self, handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError>;
}

#[async_trait]
impl<T: DbConnector + ?Sized> DbConnector for Arc<T> {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        self.as_ref().connect(config, password).await
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        self.as_ref().disconnect(handle).await
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        self.as_ref().test_connection(config, password).await
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        self.as_ref().query(handle, sql, params).await
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        self.as_ref().execute(handle, sql, params).await
    }

    async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
        self.as_ref().execute_batch(handle, statements).await
    }

    async fn execute_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        self.as_ref()
            .execute_transaction(handle, statements, read_statements)
            .await
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        self.as_ref().introspect(handle).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        self.as_ref().explain(handle, sql).await
    }

    fn dialect(&self, handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError> {
        self.as_ref().dialect(handle)
    }
}
