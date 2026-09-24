use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, SslMode};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::QueryParam;
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::DbConnector;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{Column, Row as SqlxRow};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

use super::introspect::MySqlIntrospect;
use super::query_mapper::MySqlQueryMapper;

pub struct MySqlConnector {
    pools: RwLock<HashMap<u64, MySqlPool>>,
    next_id: AtomicU64,
}

impl MySqlConnector {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub async fn get_pool(&self, handle: &ConnectionHandle) -> Option<MySqlPool> {
        self.pools.read().await.get(&handle.0).cloned()
    }
}

impl Default for MySqlConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the sqlx MySQL URL, including the stored TLS mode.
///
/// sqlx accepts `ssl-mode` as `DISABLED` / `REQUIRED` / `VERIFY_CA` / `VERIFY_IDENTITY`
/// (see `sqlx_mysql::options::parse`). Without this query parameter the connector always
/// negotiated `PREFERRED`, so an explicit `Disable` or `Require` in the connection form
/// was silently ignored.
pub(crate) fn mysql_connection_url(config: &ConnectionConfig, password: &str) -> String {
    let ssl_mode = match config.ssl_mode {
        SslMode::Disable => "DISABLED",
        SslMode::Require => "REQUIRED",
        SslMode::VerifyCa => "VERIFY_CA",
        SslMode::VerifyFull => "VERIFY_IDENTITY",
    };
    format!(
        "mysql://{}:{}@{}:{}/{}?ssl-mode={}",
        config.username, password, config.host, config.port, config.database, ssl_mode
    )
}

#[async_trait]
impl DbConnector for MySqlConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let url = mysql_connection_url(config, password);

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&url)
            .await
            .map_err(|e| DbError::ConnectionFailed(format!("MySQL connect failed: {}", e)))?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.pools.write().await.insert(id, pool);
        Ok(ConnectionHandle::new(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        if let Some(pool) = self.pools.write().await.remove(&handle.0) {
            pool.close().await;
        }
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let handle = self.connect(config, password).await?;
        self.disconnect(&handle).await
    }

    async fn query(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<db_pro_core::domain::query::QueryResult, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let mut args = sqlx::mysql::MySqlArguments::default();
        super::query_mapper::bind_params(params, &mut args)?;

        let rows = sqlx::query_with(sql, args)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::QueryFailed(format!("MySQL query failed: {}", e)))?;

        MySqlQueryMapper::map_rows(rows)
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let mut args = sqlx::mysql::MySqlArguments::default();
        super::query_mapper::bind_params(params, &mut args)?;

        let result = sqlx::query_with(sql, args)
            .execute(&pool)
            .await
            .map_err(|e| DbError::QueryFailed(format!("MySQL execute failed: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn cancel(&self, _handle: &ConnectionHandle) -> Result<(), DbError> {
        Err(DbError::Unsupported(
            "MySQL connector does not support cancellation".into(),
        ))
    }

    async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let mut total = 0;
        for stmt in statements {
            let result = sqlx::query(stmt)
                .execute(&pool)
                .await
                .map_err(|e| DbError::QueryFailed(format!("MySQL batch statement failed: {}", e)))?;
            total += result.rows_affected();
        }
        Ok(total)
    }

    async fn execute_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
        if statements.len() != read_statements.len() {
            return Err(db_pro_core::ports::TransactionFailure {
                phase: db_pro_core::ports::TransactionFailurePhase::Validation,
                statement_index: 0,
                outcome: db_pro_core::ports::TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Validation("MySQL transaction statement metadata length mismatch".into()),
            });
        }

        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| db_pro_core::ports::TransactionFailure {
                phase: db_pro_core::ports::TransactionFailurePhase::Validation,
                statement_index: 0,
                outcome: db_pro_core::ports::TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::ConnectionFailed("no MySQL pool for handle".into()),
            })?;

        let mut tx = pool.begin().await.map_err(|e| db_pro_core::ports::TransactionFailure {
            phase: db_pro_core::ports::TransactionFailurePhase::Begin,
            statement_index: 0,
            outcome: db_pro_core::ports::TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: DbError::QueryFailed(format!("MySQL transaction begin failed: {}", e)),
        })?;

        let mut results = Vec::with_capacity(statements.len());
        for (idx, (stmt, is_read)) in statements.iter().zip(read_statements).enumerate() {
            let started = std::time::Instant::now();
            if *is_read {
                match sqlx::query(stmt).fetch_all(&mut *tx).await {
                    Ok(rows) => match MySqlQueryMapper::map_rows(rows) {
                        Ok(mut query_result) => {
                            query_result.duration_ms = started.elapsed().as_millis() as u64;
                            results.push(db_pro_core::ports::TransactionStatementResult::Query(query_result));
                        }
                        Err(e) => {
                            if let Err(rollback_error) = tx.rollback().await {
                                tracing::warn!(statement_index = idx, error = %rollback_error, "MySQL rollback after row mapping error also failed");
                            }
                            return Err(db_pro_core::ports::TransactionFailure {
                                phase: db_pro_core::ports::TransactionFailurePhase::Statement,
                                statement_index: idx,
                                outcome: db_pro_core::ports::TransactionFailureOutcome::RolledBack,
                                results,
                                error: e,
                            });
                        }
                    },
                    Err(e) => {
                        if let Err(rollback_error) = tx.rollback().await {
                            tracing::warn!(statement_index = idx, error = %rollback_error, "MySQL rollback after failed query also failed — transaction may still be open on the connection");
                        }
                        return Err(db_pro_core::ports::TransactionFailure {
                            phase: db_pro_core::ports::TransactionFailurePhase::Statement,
                            statement_index: idx,
                            outcome: db_pro_core::ports::TransactionFailureOutcome::RolledBack,
                            results,
                            error: DbError::QueryFailed(format!("MySQL statement {} failed: {}", idx, e)),
                        });
                    }
                }
            } else {
                match sqlx::query(stmt).execute(&mut *tx).await {
                    Ok(result) => {
                        results.push(db_pro_core::ports::TransactionStatementResult::Affected {
                            row_count: result.rows_affected(),
                            duration_ms: started.elapsed().as_millis() as u64,
                        });
                    }
                    Err(e) => {
                        if let Err(rollback_error) = tx.rollback().await {
                            tracing::warn!(statement_index = idx, error = %rollback_error, "MySQL rollback after failed statement also failed — transaction may still be open on the connection");
                        }
                        return Err(db_pro_core::ports::TransactionFailure {
                            phase: db_pro_core::ports::TransactionFailurePhase::Statement,
                            statement_index: idx,
                            outcome: db_pro_core::ports::TransactionFailureOutcome::RolledBack,
                            results,
                            error: DbError::QueryFailed(format!("MySQL statement {} failed: {}", idx, e)),
                        });
                    }
                }
            }
        }

        tx.commit().await.map_err(|e| db_pro_core::ports::TransactionFailure {
            phase: db_pro_core::ports::TransactionFailurePhase::Commit,
            statement_index: statements.len(),
            outcome: db_pro_core::ports::TransactionFailureOutcome::Unknown,
            results: Vec::new(),
            error: DbError::QueryFailed(format!("MySQL commit failed: {}", e)),
        })?;

        Ok(results)
    }

    async fn execute_parameterized_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[db_pro_core::ports::ParameterizedTransactionStatement],
    ) -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
        use db_pro_core::ports::{
            TransactionFailure, TransactionFailureOutcome, TransactionFailurePhase, TransactionStatementResult,
        };

        let pool = self.get_pool(handle).await.ok_or_else(|| TransactionFailure {
            phase: TransactionFailurePhase::Validation,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: DbError::ConnectionFailed("no MySQL pool for handle".into()),
        })?;

        let mut tx = pool.begin().await.map_err(|e| TransactionFailure {
            phase: TransactionFailurePhase::Begin,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: DbError::QueryFailed(format!("MySQL transaction begin failed: {e}")),
        })?;

        let mut results = Vec::with_capacity(statements.len());
        for (index, statement) in statements.iter().enumerate() {
            let mut args = sqlx::mysql::MySqlArguments::default();
            if let Err(error) = super::query_mapper::bind_params(&statement.params, &mut args) {
                let (error, outcome) = match tx.rollback().await {
                    Ok(()) => (error, TransactionFailureOutcome::RolledBack),
                    Err(rollback_error) => (
                        DbError::Internal(format!("bind failed: {error}; rollback failed: {rollback_error}")),
                        TransactionFailureOutcome::Unknown,
                    ),
                };
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Statement,
                    statement_index: index,
                    outcome,
                    results,
                    error,
                });
            }

            let affected = match sqlx::query_with(&statement.sql, args).execute(&mut *tx).await {
                Ok(result) => result.rows_affected(),
                Err(e) => {
                    let error = DbError::QueryFailed(format!("MySQL statement {index} failed: {e}"));
                    let (error, outcome) = match tx.rollback().await {
                        Ok(()) => (error, TransactionFailureOutcome::RolledBack),
                        Err(rollback_error) => (
                            DbError::Internal(format!("statement failed: {error}; rollback failed: {rollback_error}")),
                            TransactionFailureOutcome::Unknown,
                        ),
                    };
                    return Err(TransactionFailure {
                        phase: TransactionFailurePhase::Statement,
                        statement_index: index,
                        outcome,
                        results,
                        error,
                    });
                }
            };

            if statement.expect_affected_rows && affected == 0 {
                let error = DbError::Conflict("table mutation affected no rows".into());
                let (error, outcome) = match tx.rollback().await {
                    Ok(()) => (error, TransactionFailureOutcome::RolledBack),
                    Err(rollback_error) => (
                        DbError::Internal(format!("mutation affected no rows; rollback failed: {rollback_error}")),
                        TransactionFailureOutcome::Unknown,
                    ),
                };
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Statement,
                    statement_index: index,
                    outcome,
                    results,
                    error,
                });
            }
            if statement.max_affected_rows.is_some_and(|maximum| affected > maximum) {
                let error = DbError::Internal(format!(
                    "table mutation affected {affected} rows; expected at most {}",
                    statement.max_affected_rows.unwrap_or_default()
                ));
                let (error, outcome) = match tx.rollback().await {
                    Ok(()) => (error, TransactionFailureOutcome::RolledBack),
                    Err(rollback_error) => (
                        DbError::Internal(format!(
                            "mutation invariant failed: {error}; rollback failed: {rollback_error}"
                        )),
                        TransactionFailureOutcome::Unknown,
                    ),
                };
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Statement,
                    statement_index: index,
                    outcome,
                    results,
                    error,
                });
            }

            results.push(TransactionStatementResult::Affected {
                row_count: affected,
                duration_ms: 0,
            });
        }

        if let Err(e) = tx.commit().await {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Commit,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::Unknown,
                results,
                error: DbError::QueryFailed(format!("MySQL commit failed: {e}")),
            });
        }

        Ok(results)
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;
        MySqlIntrospect::introspect(&pool).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str, analyze: bool) -> Result<serde_json::Value, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let explain_sql = if analyze {
            format!("EXPLAIN ANALYZE {}", sql)
        } else {
            format!("EXPLAIN {}", sql)
        };
        let rows = sqlx::query(&explain_sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::QueryFailed(format!("MySQL EXPLAIN failed: {}", e)))?;

        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in row.columns().iter().enumerate() {
                    let val: Option<String> = row.try_get(i).ok().flatten();
                    map.insert(
                        col.name().to_string(),
                        val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                    );
                }
                serde_json::Value::Object(map)
            })
            .collect();

        Ok(serde_json::Value::Array(json_rows))
    }

    fn dialect(&self, _handle: &ConnectionHandle) -> Result<Box<dyn db_pro_core::ports::SqlDialect>, DbError> {
        Ok(Box::new(MySqlDialect))
    }
}

struct MySqlDialect;
impl db_pro_core::ports::SqlDialect for MySqlDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
    fn quote_identifier(&self, name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_pro_core::domain::connection::DriverType;

    fn sample_config(ssl_mode: SslMode) -> ConnectionConfig {
        ConnectionConfig {
            name: "mysql".into(),
            host: "127.0.0.1".into(),
            port: 3306,
            database: "app".into(),
            username: "root".into(),
            driver: DriverType::Mysql,
            ssl_mode,
            ssh_tunnel: None,
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 10_000,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
            readonly: false,
        }
    }

    use db_pro_core::ports::{TransactionFailureOutcome, TransactionFailurePhase};

    #[tokio::test]
    async fn execute_transaction_reports_validation_failure_on_mismatched_read_statements_length() {
        let connector = MySqlConnector::new();
        let handle = ConnectionHandle::new(999);
        let statements = vec!["SELECT 1".to_string(), "SELECT 2".to_string()];
        let read_statements = vec![true]; // Mismatched length: 2 statements vs 1 read flag

        let failure = connector
            .execute_transaction(&handle, &statements, &read_statements)
            .await
            .expect_err("mismatched statement lengths must fail validation");

        assert_eq!(failure.phase, TransactionFailurePhase::Validation);
        assert_eq!(
            failure.statement_index, 0,
            "statement_index must be 0 for Validation phase failure"
        );
        assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
    }

    #[tokio::test]
    async fn execute_transaction_reports_begin_failure_on_unknown_handle() {
        let connector = MySqlConnector::new();
        let handle = ConnectionHandle::new(999); // Handle not connected
        let statements = vec!["SELECT 1".to_string(), "SELECT 2".to_string()];
        let read_statements = vec![true, true];

        let failure = connector
            .execute_transaction(&handle, &statements, &read_statements)
            .await
            .expect_err("begin transaction on unknown handle must fail validation/connection lookup");

        assert_eq!(failure.phase, TransactionFailurePhase::Validation);
        assert_eq!(
            failure.statement_index, 0,
            "statement_index must be 0 for Validation phase failure"
        );
        assert_eq!(failure.outcome, TransactionFailureOutcome::NotStarted);
    }

    #[test]
    fn mysql_connection_url_maps_every_ssl_mode() {
        let cases = [
            (SslMode::Disable, "DISABLED"),
            (SslMode::Require, "REQUIRED"),
            (SslMode::VerifyCa, "VERIFY_CA"),
            (SslMode::VerifyFull, "VERIFY_IDENTITY"),
        ];
        for (mode, expected) in cases {
            let url = mysql_connection_url(&sample_config(mode), "secret");
            assert!(
                url.ends_with(&format!("?ssl-mode={expected}")),
                "expected ssl-mode={expected} in {url}"
            );
            assert!(url.starts_with("mysql://root:secret@127.0.0.1:3306/app"));
        }
    }
}
