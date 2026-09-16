use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, SslMode};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult, Row};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{
    DbConnector, ParameterizedTransactionStatement, SqlDialect, TransactionFailure, TransactionFailureOutcome,
    TransactionFailurePhase, TransactionStatementResult,
};
use futures_util::TryStreamExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, QueryItem};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::introspect::SqlServerIntrospect;
use super::query_mapper::{bind_query, decode_cell};

type SqlServerClient = Client<Compat<TcpStream>>;
type ClientHandle = Arc<Mutex<SqlServerClient>>;

pub struct SqlServerConnector {
    clients: RwLock<HashMap<u64, ClientHandle>>,
    next_id: AtomicU64,
}

impl SqlServerConnector {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    async fn client(&self, handle: &ConnectionHandle) -> Result<ClientHandle, DbError> {
        self.clients
            .read()
            .await
            .get(&handle.0)
            .cloned()
            .ok_or_else(|| DbError::ConnectionFailed(format!("no SQL Server client for handle {}", handle.0)))
    }

    async fn query_with_params(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<QueryResult, DbError> {
        let client = self.client(handle).await?;
        let mut client = client.lock().await;
        let query = bind_query(sql, params)?;
        let started = Instant::now();
        let mut stream = query
            .query(&mut *client)
            .await
            .map_err(|error| DbError::QueryFailed(format!("SQL Server query failed: {error}")))?;
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        while let Some(item) = stream
            .try_next()
            .await
            .map_err(|error| DbError::QueryFailed(format!("SQL Server result stream failed: {error}")))?
        {
            match item {
                QueryItem::Metadata(metadata) if metadata.result_index() == 0 => {
                    columns = metadata
                        .columns()
                        .iter()
                        .map(|column| db_pro_core::domain::query::ColumnMeta {
                            name: column.name().to_owned(),
                            data_type: format!("{:?}", column.column_type()),
                            nullable: true,
                        })
                        .collect();
                }
                QueryItem::Row(row) if row.result_index() == 0 => {
                    let cells = (0..row.len())
                        .map(|index| decode_cell(&row, index))
                        .collect::<Result<Vec<_>, _>>()?;
                    rows.push(Row(cells));
                }
                _ => {}
            }
        }
        let row_count = rows.len() as u64;
        Ok(QueryResult {
            columns,
            rows,
            row_count,
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    async fn execute_one(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        let client = self.client(handle).await?;
        let mut client = client.lock().await;
        let query = bind_query(sql, params)?;
        let result = query
            .execute(&mut *client)
            .await
            .map_err(|error| DbError::QueryFailed(format!("SQL Server execute failed: {error}")))?;
        Ok(result.total())
    }

    async fn execute_control(&self, handle: &ConnectionHandle, sql: &str) -> Result<(), DbError> {
        self.execute_one(handle, sql, &[]).await.map(|_| ())
    }

    async fn rollback(&self, handle: &ConnectionHandle) -> TransactionFailureOutcome {
        match self.execute_control(handle, "ROLLBACK TRANSACTION").await {
            Ok(()) => TransactionFailureOutcome::RolledBack,
            Err(_) => TransactionFailureOutcome::Unknown,
        }
    }
}

impl Default for SqlServerConnector {
    fn default() -> Self {
        Self::new()
    }
}

fn sql_server_config(config: &ConnectionConfig, password: &str) -> Result<Config, DbError> {
    let mut server = Config::new();
    server.host(&config.host);
    server.port(config.port);
    server.database(&config.database);
    server.authentication(AuthMethod::sql_server(&config.username, password));
    match config.ssl_mode {
        SslMode::Disable => server.encryption(EncryptionLevel::Off),
        SslMode::Require => {
            server.encryption(EncryptionLevel::Required);
            server.trust_cert();
        }
        SslMode::VerifyCa | SslMode::VerifyFull => {
            server.encryption(EncryptionLevel::Required);
            let ca_path = config.ssl_root_cert_path.as_deref().ok_or_else(|| {
                DbError::Validation("SQL Server VerifyCa/VerifyFull requires a CA certificate path".into())
            })?;
            server.trust_cert_ca(ca_path);
        }
    }
    Ok(server)
}

#[async_trait]
impl DbConnector for SqlServerConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let server = sql_server_config(config, password)?;
        let tcp = timeout(Duration::from_secs(10), TcpStream::connect(server.get_addr()))
            .await
            .map_err(|_| DbError::ConnectionTimeout("SQL Server TCP connect timed out".into()))?
            .map_err(|error| DbError::ConnectionRefused(format!("SQL Server TCP connect failed: {error}")))?;
        tcp.set_nodelay(true)
            .map_err(|error| DbError::Io(format!("SQL Server TCP setup failed: {error}")))?;
        let client = timeout(Duration::from_secs(10), Client::connect(server, tcp.compat_write()))
            .await
            .map_err(|_| DbError::ConnectionTimeout("SQL Server login timed out".into()))?
            .map_err(|error| DbError::ConnectionFailed(format!("SQL Server login failed: {error}")))?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.clients.write().await.insert(id, Arc::new(Mutex::new(client)));
        Ok(ConnectionHandle::new(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        self.clients.write().await.remove(&handle.0);
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let handle = self.connect(config, password).await?;
        self.disconnect(&handle).await
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        self.query_with_params(handle, sql, params).await
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        self.execute_one(handle, sql, params).await
    }

    async fn cancel(&self, _handle: &ConnectionHandle) -> Result<(), DbError> {
        Err(DbError::Unsupported(
            "SQL Server query cancellation is not exposed by the TDS adapter yet".into(),
        ))
    }

    async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
        let results = self
            .execute_transaction(handle, statements, &vec![false; statements.len()])
            .await
            .map_err(|failure| failure.error)?;
        Ok(results
            .into_iter()
            .map(|result| match result {
                TransactionStatementResult::Affected { row_count, .. } => row_count,
                TransactionStatementResult::Query(_) => 0,
            })
            .sum())
    }

    async fn execute_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        if statements.len() != read_statements.len() {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Validation,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Validation("SQL Server transaction statement metadata length mismatch".into()),
            });
        }
        self.execute_control(handle, "BEGIN TRANSACTION")
            .await
            .map_err(|error| TransactionFailure {
                phase: TransactionFailurePhase::Begin,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error,
            })?;
        let mut results = Vec::with_capacity(statements.len());
        for (index, (statement, is_read)) in statements.iter().zip(read_statements).enumerate() {
            let outcome = if *is_read {
                self.query(handle, statement, &[])
                    .await
                    .map(TransactionStatementResult::Query)
            } else {
                self.execute(handle, statement, &[])
                    .await
                    .map(|row_count| TransactionStatementResult::Affected {
                        row_count,
                        duration_ms: 0,
                    })
            };
            match outcome {
                Ok(result) => results.push(result),
                Err(error) => {
                    let rollback = self.rollback(handle).await;
                    return Err(TransactionFailure {
                        phase: TransactionFailurePhase::Statement,
                        statement_index: index,
                        outcome: rollback,
                        results,
                        error,
                    });
                }
            }
        }
        if let Err(error) = self.execute_control(handle, "COMMIT TRANSACTION").await {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Commit,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::Unknown,
                results,
                error,
            });
        }
        Ok(results)
    }

    async fn execute_parameterized_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[ParameterizedTransactionStatement],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        self.execute_control(handle, "BEGIN TRANSACTION")
            .await
            .map_err(|error| TransactionFailure {
                phase: TransactionFailurePhase::Begin,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error,
            })?;
        let mut results = Vec::with_capacity(statements.len());
        for (index, statement) in statements.iter().enumerate() {
            match self.execute(handle, &statement.sql, &statement.params).await {
                Ok(row_count) if statement.expect_affected_rows && row_count == 0 => {
                    let error = DbError::Conflict("SQL Server mutation affected no rows".into());
                    let rollback = self.rollback(handle).await;
                    return Err(TransactionFailure {
                        phase: TransactionFailurePhase::Statement,
                        statement_index: index,
                        outcome: rollback,
                        results,
                        error,
                    });
                }
                Ok(row_count) => {
                    if statement.max_affected_rows.is_some_and(|max| row_count > max) {
                        let error = DbError::Conflict("SQL Server mutation affected more rows than allowed".into());
                        let rollback = self.rollback(handle).await;
                        return Err(TransactionFailure {
                            phase: TransactionFailurePhase::Statement,
                            statement_index: index,
                            outcome: rollback,
                            results,
                            error,
                        });
                    }
                    results.push(TransactionStatementResult::Affected {
                        row_count,
                        duration_ms: 0,
                    });
                }
                Err(error) => {
                    let rollback = self.rollback(handle).await;
                    return Err(TransactionFailure {
                        phase: TransactionFailurePhase::Statement,
                        statement_index: index,
                        outcome: rollback,
                        results,
                        error,
                    });
                }
            }
        }
        if let Err(error) = self.execute_control(handle, "COMMIT TRANSACTION").await {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Commit,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::Unknown,
                results,
                error,
            });
        }
        Ok(results)
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        SqlServerIntrospect::introspect(self, handle).await
    }

    async fn explain(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        _analyze: bool,
    ) -> Result<serde_json::Value, DbError> {
        let result = self
            .query(
                handle,
                &format!("SET SHOWPLAN_TEXT ON; {sql}; SET SHOWPLAN_TEXT OFF"),
                &[],
            )
            .await?;
        Ok(serde_json::to_value(result.rows)
            .map_err(|error| DbError::Internal(format!("SQL Server plan serialization failed: {error}")))?)
    }

    fn dialect(&self, _handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError> {
        Ok(Box::new(SqlServerDialect))
    }
}

pub struct SqlServerDialect;

impl SqlDialect for SqlServerDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("@p{index}")
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("[{}]", name.replace(']', "]]"))
    }

    fn pagination_clause(&self, limit_placeholder: &str, offset_placeholder: &str) -> String {
        format!(" OFFSET {offset_placeholder} ROWS FETCH NEXT {limit_placeholder} ROWS ONLY")
    }

    fn pagination_requires_order_by(&self) -> bool {
        true
    }
}
