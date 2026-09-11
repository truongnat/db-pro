use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{DbConnector, SqlDialect, TransactionFailure, TransactionStatementResult};
use sqlx::{Executor as _, PgPool};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

struct PostgresDialect;
impl SqlDialect for PostgresDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

pub struct PoolEntry {
    pub pool: PgPool,
    pub query_timeout: std::time::Duration,
    pub max_rows: u64,
}

pub struct PostgresConnector {
    pools: RwLock<HashMap<u64, PoolEntry>>,
    next_id: AtomicU64,
}

pub(crate) async fn with_query_timeout<T, F>(timeout: Duration, future: F) -> Result<T, DbError>
where
    F: Future<Output = Result<T, DbError>>,
{
    tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| DbError::QueryTimeout {
            timeout_ms: timeout.as_millis() as u64,
        })?
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

    pub async fn get_pool(&self, handle: &ConnectionHandle) -> Option<PgPool> {
        let pools = self.pools.read().await;
        pools.get(&handle.0).map(|entry| entry.pool.clone())
    }

    pub async fn query_timeout(&self, handle: &ConnectionHandle) -> Result<Duration, DbError> {
        let pools = self.pools.read().await;
        pools
            .get(&handle.0)
            .map(|entry| entry.query_timeout)
            .ok_or_else(|| DbError::ConnectionFailed("no pool for handle".into()))
    }
}

#[async_trait]
impl DbConnector for PostgresConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let options = super::connection_string::build_options(config, password)?;
        let timeout = Duration::from_millis(config.query_timeout_ms);
        let pool = with_query_timeout(timeout, async {
            PgPool::connect_with(options).await.map_err(crate::error::from_sqlx)
        })
        .await?;

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
        let timeout = Duration::from_millis(config.query_timeout_ms);
        let pool = with_query_timeout(timeout, async {
            PgPool::connect_with(options).await.map_err(crate::error::from_sqlx)
        })
        .await?;
        let result = with_query_timeout(timeout, async {
            sqlx::query("SELECT 1")
                .execute(&pool)
                .await
                .map_err(crate::error::from_sqlx)
                .map(|_| ())
        })
        .await;
        pool.close().await;
        result
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

        with_query_timeout(timeout, future).await
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

        with_query_timeout(timeout, future).await
    }

    async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let timeout = entry.query_timeout;
        let pool = entry.pool.clone();
        drop(pools);

        let future = async {
            let mut tx = pool.begin().await.map_err(crate::error::from_sqlx)?;
            let mut total_affected: u64 = 0;

            for stmt in statements {
                let result = sqlx::query(stmt)
                    .execute(&mut *tx)
                    .await
                    .map_err(crate::error::from_sqlx)?;
                total_affected += result.rows_affected();
            }

            tx.commit().await.map_err(crate::error::from_sqlx)?;
            Ok(total_affected)
        };

        with_query_timeout(timeout, future).await
    }

    async fn execute_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        if statements.len() != read_statements.len() {
            return Err(TransactionFailure {
                statement_index: 0,
                results: Vec::new(),
                error: DbError::Internal("transaction statement metadata length mismatch".into()),
            });
        }

        let pools = self.pools.read().await;
        let entry = pools.get(&handle.0).ok_or_else(|| TransactionFailure {
            statement_index: 0,
            results: Vec::new(),
            error: DbError::ConnectionFailed("handle not found".into()),
        })?;
        let timeout = entry.query_timeout;
        let max_rows = entry.max_rows;
        let pool = entry.pool.clone();
        drop(pools);

        let future = async {
            let mut tx = match pool.begin().await.map_err(crate::error::from_sqlx) {
                Ok(tx) => tx,
                Err(error) => {
                    return Err(TransactionFailure {
                        statement_index: 0,
                        results: Vec::new(),
                        error,
                    });
                }
            };
            let mut results = Vec::with_capacity(statements.len());

            for (index, (statement, is_read)) in statements.iter().zip(read_statements).enumerate() {
                let started = std::time::Instant::now();
                let statement_result: Result<TransactionStatementResult, DbError> = async {
                    if *is_read {
                        let describe = tx.describe(statement).await.map_err(crate::error::from_sqlx)?;
                        let columns = super::query_mapper::columns_from_describe(&describe);
                        use futures_util::StreamExt;
                        let mut stream = sqlx::query(statement).fetch(&mut *tx);
                        let mut rows = Vec::new();
                        while (rows.len() as u64) < max_rows {
                            let row = match stream.next().await {
                                Some(Ok(row)) => row,
                                Some(Err(error)) => return Err(crate::error::from_sqlx(error)),
                                None => break,
                            };
                            rows.push(super::query_mapper::map_row(&row, &columns)?);
                        }
                        Ok(TransactionStatementResult::Query(QueryResult {
                            row_count: rows.len() as u64,
                            columns,
                            rows,
                            duration_ms: started.elapsed().as_millis() as u64,
                        }))
                    } else {
                        let affected = sqlx::query(statement)
                            .execute(&mut *tx)
                            .await
                            .map_err(crate::error::from_sqlx)?
                            .rows_affected();
                        Ok(TransactionStatementResult::Affected {
                            row_count: affected,
                            duration_ms: started.elapsed().as_millis() as u64,
                        })
                    }
                }
                .await;
                match statement_result {
                    Ok(result) => results.push(result),
                    Err(error) => {
                        let error = match tx.rollback().await.map_err(crate::error::from_sqlx) {
                            Ok(()) => error,
                            Err(rollback_error) => DbError::Internal(format!(
                                "statement failed: {error}; rollback failed: {rollback_error}"
                            )),
                        };
                        return Err(TransactionFailure {
                            statement_index: index,
                            results,
                            error,
                        });
                    }
                }
            }

            if let Err(error) = tx.commit().await.map_err(crate::error::from_sqlx) {
                return Err(TransactionFailure {
                    statement_index: statements.len(),
                    results,
                    error,
                });
            }
            Ok(results)
        };

        match tokio::time::timeout(timeout, future).await {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(TransactionFailure {
                statement_index: 0,
                results: Vec::new(),
                error: DbError::QueryTimeout {
                    timeout_ms: timeout.as_millis() as u64,
                },
            }),
        }
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let timeout = entry.query_timeout;
        let pool = entry.pool.clone();
        drop(pools);

        with_query_timeout(timeout, super::introspect::run_introspection(&pool)).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        let pools = self.pools.read().await;
        let entry = pools
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        let timeout = entry.query_timeout;
        let pool = entry.pool.clone();
        drop(pools);

        if sql.contains(';') && sql.trim().trim_end_matches(';').contains(';') {
            return Err(DbError::QueryFailed("multi-statement execution is disabled".into()));
        }

        let explain_sql = format!("EXPLAIN (FORMAT JSON) {sql}");
        with_query_timeout(timeout, async {
            let row: (serde_json::Value,) = sqlx::query_as(&explain_sql)
                .fetch_one(&pool)
                .await
                .map_err(crate::error::from_sqlx)?;
            Ok(row.0)
        })
        .await
    }

    fn dialect(&self, _handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError> {
        Ok(Box::new(PostgresDialect))
    }
}

impl PostgresConnector {
    pub async fn get_object_dependencies(
        &self,
        handle: &ConnectionHandle,
        schema: &str,
        object_name: &str,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::ObjectDependency>, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no pool for handle".into()))?;
        let timeout = self.query_timeout(handle).await?;
        with_query_timeout(
            timeout,
            super::cross_connection::get_object_dependencies(&pool, schema, object_name),
        )
        .await
    }

    pub async fn list_partitions(
        &self,
        handle: &ConnectionHandle,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::PartitionInfo>, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no pool for handle".into()))?;
        let timeout = self.query_timeout(handle).await?;
        with_query_timeout(timeout, super::cross_connection::list_partitions(&pool)).await
    }

    pub async fn list_tablespaces(
        &self,
        handle: &ConnectionHandle,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::TablespaceInfo>, DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no pool for handle".into()))?;
        let timeout = self.query_timeout(handle).await?;
        with_query_timeout(timeout, super::cross_connection::list_tablespaces(&pool)).await
    }

    pub async fn rename_schema_object(
        &self,
        handle: &ConnectionHandle,
        object_type: &str,
        schema: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DbError> {
        let pool = self
            .get_pool(handle)
            .await
            .ok_or_else(|| DbError::ConnectionFailed("no pool for handle".into()))?;
        let timeout = self.query_timeout(handle).await?;
        with_query_timeout(
            timeout,
            super::cross_connection::rename_schema_object(&pool, object_type, schema, old_name, new_name),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::{with_query_timeout, PostgresDialect};
    use db_pro_core::application::sql_builder::{build_select, SortClause, SortDir};
    use db_pro_core::domain::error::DbError;
    use std::time::Duration;

    #[test]
    fn table_pagination_uses_postgres_placeholders() {
        let (sql, params) = build_select(
            &PostgresDialect,
            "public",
            "customers",
            &[],
            &[SortClause {
                column: "id".to_owned(),
                direction: SortDir::Asc,
            }],
            100,
            200,
        )
        .expect("PostgreSQL pagination should build");

        assert_eq!(
            sql,
            r#"SELECT * FROM "public"."customers" ORDER BY "id" ASC LIMIT $1 OFFSET $2"#
        );
        assert_eq!(params.len(), 2);
    }

    #[tokio::test]
    async fn postgres_operation_timeout_returns_query_timeout() {
        let error = with_query_timeout(Duration::from_millis(1), async {
            tokio::time::sleep(Duration::from_millis(25)).await;
            Ok::<_, DbError>(())
        })
        .await
        .expect_err("operation should exceed its configured deadline");

        assert!(matches!(error, DbError::QueryTimeout { timeout_ms: 1 }));
    }
}
