use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle};
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

#[async_trait]
impl DbConnector for MySqlConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, password, config.host, config.port, config.database
        );

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
        _params: &[QueryParam],
    ) -> Result<db_pro_core::domain::query::QueryResult, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let rows = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::QueryFailed(format!("MySQL query failed: {}", e)))?;

        MySqlQueryMapper::map_rows(rows)
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, _params: &[QueryParam]) -> Result<u64, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let result = sqlx::query(sql)
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
        _read_statements: &[bool],
    ) -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
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

        let mut results = Vec::new();
        for (idx, stmt) in statements.iter().enumerate() {
            match sqlx::query(stmt).execute(&mut *tx).await {
                Ok(result) => {
                    results.push(db_pro_core::ports::TransactionStatementResult::Affected {
                        row_count: result.rows_affected(),
                        duration_ms: 0,
                    });
                }
                Err(e) => {
                    tx.rollback().await.ok();
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

        tx.commit().await.map_err(|e| db_pro_core::ports::TransactionFailure {
            phase: db_pro_core::ports::TransactionFailurePhase::Commit,
            statement_index: statements.len(),
            outcome: db_pro_core::ports::TransactionFailureOutcome::Unknown,
            results: Vec::new(),
            error: DbError::QueryFailed(format!("MySQL commit failed: {}", e)),
        })?;

        Ok(results)
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;
        MySqlIntrospect::introspect(&pool).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

        let explain_sql = format!("EXPLAIN {}", sql);
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
