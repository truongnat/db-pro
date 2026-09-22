use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::{QueryHistory, SavedQuery, SavedQueryFolder};
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::run_config::RunConfig;
use crate::domain::safety::{
    classify_statement_safety, validate_against_policy, ConnectionSafetyPolicy, StatementSafety,
};
use crate::ports::{
    ConnectionRepository, DbConnector, IntrospectionCache, QueryHistoryRepository, RunConfigRepository,
    SavedQueryRepository, TransactionFailureOutcome, TransactionFailurePhase, TransactionStatementResult,
};

use super::registry::ConnectionRegistry;
use super::sql_policy::{reject_multi_statement, split_statements};
use multi_query_execution::MultiQueryExecution;
use query_classification::{classify_statement, StatementClass};

#[path = "multi_query_execution.rs"]
mod multi_query_execution;
#[path = "query_classification.rs"]
mod query_classification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementResultKind {
    ResultSet,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiQueryError {
    pub code: String,
    pub message: String,
    pub position: Option<usize>,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

impl MultiQueryError {
    fn message(message: impl Into<String>) -> Self {
        Self {
            code: "QUERY_FAILED".to_owned(),
            message: message.into(),
            position: None,
            detail: None,
            hint: None,
        }
    }
}

impl From<DbError> for MultiQueryError {
    fn from(error: DbError) -> Self {
        Self {
            code: error.code().to_owned(),
            message: error.to_string(),
            position: error.position(),
            detail: None,
            hint: None,
        }
    }
}

#[derive(Debug)]
pub struct MultiQueryResult {
    pub results: Vec<QueryResult>,
    /// Explicitly identifies each result in `results`. A command can have no
    /// columns, but result shape must not be inferred from that representation
    /// detail by downstream adapters.
    pub result_kinds: Vec<StatementResultKind>,
    pub total_duration_ms: u64,
    /// If execution failed, this holds the relevant index and structured error.
    /// The transaction-level sentinel is `statements.len()`; the message
    /// distinguishes confirmed rollback from an unknown final outcome.
    pub error: Option<(usize, MultiQueryError)>,
}

pub struct QueryService {
    connector: Box<dyn DbConnector>,
    history: Box<dyn QueryHistoryRepository>,
    saved_queries: Box<dyn SavedQueryRepository>,
    run_configs: Box<dyn RunConfigRepository>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
    introspection_cache: Option<Box<dyn IntrospectionCache>>,
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
            introspection_cache: None,
        }
    }

    pub fn with_introspection_cache(mut self, cache: Box<dyn IntrospectionCache>) -> Self {
        self.introspection_cache = Some(cache);
        self
    }

    async fn invalidate_schema_cache(&self, connection_id: &ConnectionId) {
        let Some(cache) = self.introspection_cache.as_ref() else {
            return;
        };
        if let Err(error) = cache.invalidate(connection_id).await {
            tracing::warn!("failed to invalidate schema cache after query DDL: {error}");
        }
    }

    async fn invalidate_schema_cache_if_ddl(&self, connection_id: &ConnectionId, sql: &str) {
        if matches!(classify_statement_safety(sql), Some(StatementSafety::Ddl)) {
            self.invalidate_schema_cache(connection_id).await;
        }
    }

    /// Build the safety policy for a connection based on its persisted config.
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
        validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;

        let result = self.connector.query(&handle, sql, params).await?;
        result.validate().map_err(DbError::QueryFailed)?;
        self.invalidate_schema_cache_if_ddl(connection_id, sql).await;

        if let Err(e) = self
            .history
            .save(
                connection_id,
                sql,
                &result,
                database.map(str::to_owned),
                schema.map(str::to_owned),
            )
            .await
        {
            tracing::warn!("failed to save query history: {e}");
        }

        Ok(result)
    }

    /// Cancel the query currently running on a connection.
    pub async fn cancel(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.connector.cancel(&handle).await
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
        let policy = self.safety_policy_for(connection_id).await?;
        let statements = split_statements(sql);
        if statements.is_empty() {
            return Err(DbError::QueryFailed("empty SQL statement".into()));
        }

        let query_statements = statements
            .iter()
            .map(|statement| matches!(classify_statement(statement), StatementClass::Read))
            .collect::<Vec<_>>();
        let has_mutation = statements
            .iter()
            .any(|statement| !matches!(classify_statement_safety(statement), Some(StatementSafety::Read)));
        let has_schema_change = statements
            .iter()
            .any(|statement| matches!(classify_statement_safety(statement), Some(StatementSafety::Ddl)));
        let execution = MultiQueryExecution {
            service: self,
            connection_id,
            handle: &handle,
            policy: &policy,
            statements: &statements,
            query_statements: &query_statements,
            has_schema_change,
            started_at: std::time::Instant::now(),
        };

        let mut result = if statements.len() > 1 && has_mutation {
            execution.execute_transactional().await
        } else {
            execution.execute_sequential().await
        };
        if result.error.is_some() {
            return Ok(result);
        }

        if statements.len() == 1 && has_schema_change {
            self.invalidate_schema_cache(connection_id).await;
        }
        result.total_duration_ms = execution.elapsed_ms();

        if let Some(first) = result.results.first() {
            if let Err(error) = self
                .history
                .save(
                    connection_id,
                    sql,
                    first,
                    database.map(str::to_owned),
                    schema.map(str::to_owned),
                )
                .await
            {
                tracing::warn!("failed to save query history: {error}");
            }
        }

        Ok(result)
    }

    pub async fn explain(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
        analyze: bool,
    ) -> Result<serde_json::Value, DbError> {
        reject_multi_statement(sql)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let policy = self.safety_policy_for(connection_id).await?;
        // EXPLAIN ANALYZE executes the statement — classify through the wrapped form.
        let policy_sql = if analyze {
            format!("EXPLAIN ANALYZE {sql}")
        } else {
            sql.to_owned()
        };
        validate_against_policy(&policy_sql, &policy).map_err(DbError::QueryFailed)?;

        self.connector.explain(&handle, sql, analyze).await
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
        self.save_query_with_id(connection_id, None, name, sql, folder).await
    }

    pub async fn save_query_with_id(
        &self,
        connection_id: &ConnectionId,
        saved_query_id: Option<uuid::Uuid>,
        name: &str,
        sql: &str,
        folder: Option<&str>,
    ) -> Result<SavedQuery, DbError> {
        let existing = if let Some(id) = saved_query_id {
            Some(
                self.saved_queries
                    .list(connection_id)
                    .await?
                    .into_iter()
                    .find(|query| query.id == id)
                    .ok_or_else(|| DbError::NotFound(format!("saved query not found: {id}")))?,
            )
        } else {
            None
        };
        let query = SavedQuery {
            id: existing.as_ref().map_or_else(uuid::Uuid::new_v4, |query| query.id),
            connection_id: *connection_id,
            name: name.to_string(),
            sql: sql.to_string(),
            folder: folder.map(String::from),
            created_at: existing.map_or_else(chrono::Utc::now, |query| query.created_at),
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

    pub async fn rename_saved_query(&self, id: &uuid::Uuid, new_name: &str) -> Result<(), DbError> {
        self.saved_queries.rename(id, new_name).await
    }

    pub async fn create_folder(&self, connection_id: &ConnectionId, name: &str) -> Result<SavedQueryFolder, DbError> {
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

fn format_transaction_failure(
    phase: TransactionFailurePhase,
    outcome: TransactionFailureOutcome,
    error: &DbError,
) -> String {
    match (phase, outcome) {
        (TransactionFailurePhase::Statement, TransactionFailureOutcome::RolledBack) => {
            format!("transaction rolled back: {error}")
        }
        (_, TransactionFailureOutcome::NotStarted) => format!("transaction did not start: {error}"),
        _ => format!("transaction failed; final outcome is unknown: {error}"),
    }
}

/// Message for a multi-statement batch that carries its own transaction control (#147).
///
/// It names the verb and the reason, and it states the two ways out, because the user's next
/// action depends on knowing that the batch — not the verb — is what is refused here.
fn transaction_control_rejection(verb: &str) -> String {
    format!(
        "`{verb}` is not allowed inside a multi-statement batch: the batch runs in one transaction, so `{verb}` \
         would end that transaction and leave the statements before it committed. Remove the transaction control, \
         or run each statement separately."
    )
}

fn transaction_result_to_query_result(
    result: TransactionStatementResult,
) -> Result<(StatementResultKind, QueryResult), DbError> {
    let (kind, query_result) = match result {
        TransactionStatementResult::Query(query) => (StatementResultKind::ResultSet, query),
        TransactionStatementResult::Affected { row_count, duration_ms } => (
            StatementResultKind::Command,
            QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count,
                duration_ms,
            },
        ),
    };
    query_result.validate().map_err(DbError::QueryFailed)?;
    Ok((kind, query_result))
}

#[cfg(test)]
#[path = "query_service/tests.rs"]
mod tests;
