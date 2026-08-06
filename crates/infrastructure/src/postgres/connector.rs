use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::DbConnector;
use sqlx::{Executor as _, PgPool};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

pub struct PoolEntry {
    pub pool: PgPool,
    pub query_timeout: std::time::Duration,
    pub max_rows: u64,
}

pub struct PostgresConnector {
    pools: RwLock<HashMap<u64, PoolEntry>>,
    next_id: AtomicU64,
}

impl Default for PostgresConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresConnector {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl DbConnector for PostgresConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let options = super::connection_string::build_options(config, password)?;
        let pool = PgPool::connect_with(options).await.map_err(crate::error::from_sqlx)?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let entry = PoolEntry {
            pool,
            query_timeout: std::time::Duration::from_millis(config.query_timeout_ms),
            max_rows: config.max_rows,
        };
        self.pools.write().await.insert(id, entry);
        Ok(ConnectionHandle::new(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        let entry = self.pools.write().await.remove(&handle.0);
        if let Some(entry) = entry {
            entry.pool.close().await;
        }
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let options = super::connection_string::build_options(config, password)?;
        let pool = PgPool::connect_with(options).await.map_err(crate::error::from_sqlx)?;
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(crate::error::from_sqlx)?;
        pool.close().await;
        Ok(())
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let timeout = entry.query_timeout;
        let max_rows = entry.max_rows;
        let pool = entry.pool.clone();
        drop(pools);

        let future = async {
            let mut pg_args = sqlx::postgres::PgArguments::default();
            super::query_mapper::bind_params(params, &mut pg_args)?;

            let describe = pool.describe(sql).await.map_err(crate::error::from_sqlx)?;
            let columns = super::query_mapper::columns_from_describe(&describe);

            use futures_util::StreamExt;
            let mut stream = sqlx::query_with(sql, pg_args).fetch(&pool);
            let mut result_rows = Vec::with_capacity(max_rows.min(1024) as usize);
            while (result_rows.len() as u64) < max_rows {
                let pg_row = match stream.next().await {
                    Some(Ok(row)) => row,
                    Some(Err(e)) => return Err(crate::error::from_sqlx(e)),
                    None => break,
                };
                let row = super::query_mapper::map_row(&pg_row, &columns)?;
                result_rows.push(row);
            }

            let row_count = result_rows.len() as u64;
            Ok(QueryResult {
                columns,
                rows: result_rows,
                row_count,
                duration_ms: 0,
            })
        };

        tokio::time::timeout(timeout, future)
            .await
            .map_err(|_| DbError::Timeout(format!("query timed out after {timeout:?}")))?
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let timeout = entry.query_timeout;
        let pool = entry.pool.clone();
        drop(pools);

        let future = async {
            let mut pg_args = sqlx::postgres::PgArguments::default();
            super::query_mapper::bind_params(params, &mut pg_args)?;

            let result = sqlx::query_with(sql, pg_args)
                .execute(&pool)
                .await
                .map_err(crate::error::from_sqlx)?;

            Ok(result.rows_affected())
        };

        tokio::time::timeout(timeout, future)
            .await
            .map_err(|_| DbError::Timeout(format!("execute timed out after {timeout:?}")))?
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let pool = entry.pool.clone();
        drop(pools);

        super::introspect::run_introspection(&pool).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let pool = entry.pool.clone();
        drop(pools);

        if sql.contains(';') && sql.trim().trim_end_matches(';').contains(';') {
            return Err(DbError::QueryFailed("multi-statement execution is disabled".into()));
        }

        let explain_sql = format!("EXPLAIN (FORMAT JSON) {sql}");
        let row: (serde_json::Value,) = sqlx::query_as(&explain_sql)
            .fetch_one(&pool)
            .await
            .map_err(crate::error::from_sqlx)?;
        Ok(row.0)
    }
}
