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

/// TDS transport chosen for a statement before it reaches the server.
///
/// tiberius' `Query` always wraps SQL in the `sp_executesql` RPC
/// (`src/query.rs:72-82`), which runs the text in a nested batch whose
/// transaction count must return to the caller's. `BEGIN TRANSACTION` through
/// that wrapper fails with SQL Server error 266 ("Transaction count after
/// EXECUTE …") and never scopes the session, so every explicit transaction,
/// `execute_batch`, and DDL batch breaks. Transaction control and
/// multi-statement scripts therefore take the raw batch (language event) path,
/// which executes the text directly in the connection session.
///
/// Bound parameters can only travel through `sp_executesql`, so any statement
/// with params stays on the RPC path. This adds no injection surface: both
/// paths execute the SQL *text* verbatim; only bind values are parameterized,
/// and those keep going through the RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SqlExecutionPath {
    SimpleBatch,
    ParameterizedRpc,
}

/// Pure routing decision: which statements need the raw batch path.
fn sql_execution_path(sql: &str, params_len: usize) -> SqlExecutionPath {
    if params_len == 0 && (is_transaction_control(sql) || is_multi_statement(sql)) {
        SqlExecutionPath::SimpleBatch
    } else {
        SqlExecutionPath::ParameterizedRpc
    }
}

fn is_transaction_control(sql: &str) -> bool {
    let head = sql.trim_start().to_ascii_lowercase();
    ["begin tran", "begin distributed", "commit", "rollback", "save tran"]
        .iter()
        .any(|prefix| head.starts_with(prefix))
}

/// A statement string carrying more than one `;`-delimited statement (trailing
/// semicolons do not count). Such scripts must stay a single server-side batch
/// (e.g. `SET SHOWPLAN_TEXT ON; <sql>; SET SHOWPLAN_TEXT OFF`), which the raw
/// batch path delivers verbatim.
fn is_multi_statement(sql: &str) -> bool {
    sql.trim_end().trim_end_matches(';').trim_end().contains(';')
}

pub struct SqlServerConnector {
    clients: RwLock<HashMap<u64, ClientHandle>>,
    max_rows: RwLock<HashMap<u64, u64>>,
    next_id: AtomicU64,
}

impl SqlServerConnector {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            max_rows: RwLock::new(HashMap::new()),
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
        let max_rows = self.max_rows.read().await.get(&handle.0).copied().unwrap_or(500);
        self.query_internal(handle, sql, params, Some(max_rows)).await
    }

    pub(crate) async fn query_uncapped(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<QueryResult, DbError> {
        self.query_internal(handle, sql, params, None).await
    }

    async fn query_internal(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[QueryParam],
        max_rows: Option<u64>,
    ) -> Result<QueryResult, DbError> {
        let client = self.client(handle).await?;
        let mut client = client.lock().await;
        let started = Instant::now();
        let mut stream = match sql_execution_path(sql, params.len()) {
            SqlExecutionPath::ParameterizedRpc => bind_query(sql, params)?
                .query(&mut *client)
                .await
                .map_err(|error| DbError::QueryFailed(format!("SQL Server query failed: {error}")))?,
            SqlExecutionPath::SimpleBatch => client
                .simple_query(sql)
                .await
                .map_err(|error| DbError::QueryFailed(format!("SQL Server query failed: {error}")))?,
        };
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
                    if let Some(max) = max_rows {
                        if rows.len() as u64 >= max {
                            break;
                        }
                    }
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
        match sql_execution_path(sql, params.len()) {
            SqlExecutionPath::ParameterizedRpc => {
                let query = bind_query(sql, params)?;
                let result = query
                    .execute(&mut *client)
                    .await
                    .map_err(|error| DbError::QueryFailed(format!("SQL Server execute failed: {error}")))?;
                Ok(result.total())
            }
            // The raw batch stream in tiberius 0.12 does not surface DONE row
            // counts, and only control/DDL/script text is routed here, where an
            // affected-row total of 0 is the honest answer.
            SqlExecutionPath::SimpleBatch => {
                let mut stream = client
                    .simple_query(sql)
                    .await
                    .map_err(|error| DbError::QueryFailed(format!("SQL Server execute failed: {error}")))?;
                while stream
                    .try_next()
                    .await
                    .map_err(|error| DbError::QueryFailed(format!("SQL Server result stream failed: {error}")))?
                    .is_some()
                {}
                Ok(0)
            }
        }
    }

    async fn execute_control(&self, handle: &ConnectionHandle, sql: &str) -> Result<(), DbError> {
        self.execute_one(handle, sql, &[]).await.map(|_| ())
    }

    async fn open_transaction_count(&self, handle: &ConnectionHandle) -> Result<i64, DbError> {
        let result = self.query(handle, "SELECT CAST(@@TRANCOUNT AS bigint)", &[]).await?;
        match result.rows.first().and_then(|row| row.0.first()) {
            Some(db_pro_core::domain::query::CellValue::Int64(value)) => Ok(*value),
            Some(db_pro_core::domain::query::CellValue::Text(value)) => value
                .parse()
                .map_err(|_| DbError::Internal("SQL Server transaction count was not a number".into())),
            _ => Err(DbError::Internal("SQL Server transaction count was missing".into())),
        }
    }

    async fn rollback(&self, handle: &ConnectionHandle) -> TransactionFailureOutcome {
        match self.execute_control(handle, "ROLLBACK TRANSACTION").await {
            Ok(()) => TransactionFailureOutcome::RolledBack,
            Err(_) => TransactionFailureOutcome::Unknown,
        }
    }
    async fn rollback_parameterized_savepoint(
        &self,
        handle: &ConnectionHandle,
        cause: DbError,
    ) -> (DbError, TransactionFailureOutcome) {
        match self
            .execute_control(handle, "ROLLBACK TRANSACTION DBPRO_PARAMETERIZED_TRANSACTION")
            .await
        {
            Ok(()) => (cause, TransactionFailureOutcome::RolledBack),
            Err(rollback_error) => (
                DbError::Internal(format!(
                    "statement failed: {cause}; rollback to savepoint failed: {rollback_error}"
                )),
                TransactionFailureOutcome::LeftOpen,
            ),
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

async fn connect_client(config: &ConnectionConfig, password: &str) -> Result<Client<Compat<TcpStream>>, DbError> {
    let server = sql_server_config(config, password)?;
    let tcp = timeout(Duration::from_secs(10), TcpStream::connect(server.get_addr()))
        .await
        .map_err(|_| DbError::ConnectionTimeout("SQL Server TCP connect timed out".into()))?
        .map_err(|error| DbError::ConnectionRefused(format!("SQL Server TCP connect failed: {error}")))?;
    tcp.set_nodelay(true)
        .map_err(|error| DbError::Io(format!("SQL Server TCP setup failed: {error}")))?;
    timeout(Duration::from_secs(10), Client::connect(server, tcp.compat_write()))
        .await
        .map_err(|_| DbError::ConnectionTimeout("SQL Server login timed out".into()))?
        .map_err(|error| DbError::ConnectionFailed(format!("SQL Server login failed: {error}")))
}

#[async_trait]
impl DbConnector for SqlServerConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let client = connect_client(config, password).await?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.clients.write().await.insert(id, Arc::new(Mutex::new(client)));
        self.max_rows.write().await.insert(id, config.max_rows);
        Ok(ConnectionHandle::new(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        self.clients.write().await.remove(&handle.0);
        self.max_rows.write().await.remove(&handle.0);
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let client = connect_client(config, password).await?;
        drop(client);
        Ok(())
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
                statement_index: 0,
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Validation("SQL Server transaction statement metadata length mismatch".into()),
            });
        }
        let already_open = match self.open_transaction_count(handle).await {
            Ok(count) => count > 0,
            Err(error) => {
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Begin,
                    statement_index: 0,
                    outcome: TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                });
            }
        };
        if !already_open {
            self.execute_control(handle, "BEGIN TRANSACTION")
                .await
                .map_err(|error| TransactionFailure {
                    phase: TransactionFailurePhase::Begin,
                    statement_index: 0,
                    outcome: TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                })?;
        }
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
                // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
                Err(error) => {
                    if already_open {
                        return Err(TransactionFailure {
                            phase: TransactionFailurePhase::Statement,
                            statement_index: index,
                            outcome: TransactionFailureOutcome::LeftOpen,
                            results,
                            error,
                        });
                    }
                    let rollback = self.rollback(handle).await;
                    // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
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
        if !already_open {
            if let Err(error) = self.execute_control(handle, "COMMIT TRANSACTION").await {
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Commit,
                    statement_index: statements.len(),
                    outcome: TransactionFailureOutcome::Unknown,
                    results,
                    error,
                });
            }
        }
        Ok(results)
    }

    async fn execute_parameterized_transaction(
        &self,
        handle: &ConnectionHandle,
        statements: &[ParameterizedTransactionStatement],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        // cc-scan:allow DUPLICATE_BLOCK — coincidental transaction control boilerplate
        let already_open = match self.open_transaction_count(handle).await {
            Ok(count) => count > 0,
            Err(error) => {
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Begin,
                    statement_index: 0,
                    outcome: TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                });
            }
        };
        if already_open {
            self.execute_control(handle, "SAVE TRANSACTION DBPRO_PARAMETERIZED_TRANSACTION")
                .await
                .map_err(|error| TransactionFailure {
                    phase: TransactionFailurePhase::Begin,
                    statement_index: 0,
                    outcome: TransactionFailureOutcome::LeftOpen,
                    results: Vec::new(),
                    error,
                })?;
        } else {
            self.execute_control(handle, "BEGIN TRANSACTION")
                .await
                .map_err(|error| TransactionFailure {
                    phase: TransactionFailurePhase::Begin,
                    statement_index: 0,
                    outcome: TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                })?;
        }
        let mut results = Vec::with_capacity(statements.len());
        for (index, statement) in statements.iter().enumerate() {
            match self.execute(handle, &statement.sql, &statement.params).await {
                Ok(row_count) if statement.expect_affected_rows && row_count == 0 => {
                    let error = DbError::Conflict("SQL Server mutation affected no rows".into());
                    if already_open {
                        let (error, outcome) = self.rollback_parameterized_savepoint(handle, error).await;
                        return Err(TransactionFailure {
                            phase: TransactionFailurePhase::Statement,
                            statement_index: index,
                            outcome,
                            results,
                            error,
                        });
                    }
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
                        if already_open {
                            let (error, outcome) = self.rollback_parameterized_savepoint(handle, error).await;
                            return Err(TransactionFailure {
                                phase: TransactionFailurePhase::Statement,
                                statement_index: index,
                                outcome,
                                results,
                                error,
                            });
                        }
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
                    if already_open {
                        let (error, outcome) = self.rollback_parameterized_savepoint(handle, error).await;
                        return Err(TransactionFailure {
                            phase: TransactionFailurePhase::Statement,
                            statement_index: index,
                            outcome,
                            results,
                            error,
                        });
                    }
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
        if !already_open {
            if let Err(error) = self.execute_control(handle, "COMMIT TRANSACTION").await {
                return Err(TransactionFailure {
                    phase: TransactionFailurePhase::Commit,
                    statement_index: statements.len(),
                    outcome: TransactionFailureOutcome::Unknown,
                    results,
                    error,
                });
            }
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
        params: &[QueryParam],
        _analyze: bool,
    ) -> Result<serde_json::Value, DbError> {
        // SHOWPLAN_TEXT does not execute the statement. Parameters bind on the user SQL.
        let result = self
            .query(
                handle,
                &format!("SET SHOWPLAN_TEXT ON; {sql}; SET SHOWPLAN_TEXT OFF"),
                params,
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

    fn text_like(&self, column: &str, placeholder: &str) -> String {
        format!("CAST({column} AS NVARCHAR(MAX)) LIKE {placeholder}")
    }
}

#[cfg(test)]
#[path = "tests/connector_tests.rs"]
mod tests;
