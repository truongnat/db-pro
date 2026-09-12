use std::sync::{mpsc, Arc};
use std::time::Instant;

use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult, Row};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{
    ParameterizedTransactionStatement, TransactionFailure, TransactionFailureOutcome, TransactionFailurePhase,
    TransactionStatementResult,
};
use tokio::sync::oneshot;
use tracing;

use super::query_mapper::{extract_columns, map_row_to_cells, to_rusqlite_params};

// ---------------------------------------------------------------------------
// Command enum – every variant carries a oneshot responder
// ---------------------------------------------------------------------------

pub enum SqliteCommand {
    Execute {
        sql: String,
        params: Vec<QueryParam>,
        max_rows: u64,
        responder: oneshot::Sender<Result<QueryResult, DbError>>,
    },
    Interrupt {
        responder: oneshot::Sender<Result<(), DbError>>,
    },
    Introspect {
        responder: oneshot::Sender<Result<IntrospectResult, DbError>>,
    },
    Explain {
        sql: String,
        responder: oneshot::Sender<Result<serde_json::Value, DbError>>,
    },
    RawQuery {
        sql: String,
        params: Vec<String>,
        responder: oneshot::Sender<Result<Vec<Vec<String>>, DbError>>,
    },
    ExecuteStatement {
        sql: String,
        responder: oneshot::Sender<Result<usize, DbError>>,
    },
    ExecuteStatementParam {
        sql: String,
        params: Vec<QueryParam>,
        responder: oneshot::Sender<Result<usize, DbError>>,
    },
    ExecuteBatch {
        statements: Vec<String>,
        responder: oneshot::Sender<Result<u64, DbError>>,
    },
    ExecuteTransaction {
        statements: Vec<String>,
        read_statements: Vec<bool>,
        max_rows: u64,
        responder: oneshot::Sender<Result<Vec<TransactionStatementResult>, TransactionFailure>>,
    },
    ExecuteParameterizedTransaction {
        statements: Vec<ParameterizedTransactionStatement>,
        responder: oneshot::Sender<Result<Vec<TransactionStatementResult>, TransactionFailure>>,
    },
    Shutdown,
}

// ---------------------------------------------------------------------------
// SqliteHandle – cheap-to-clone, async-facing handle
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SqliteHandle {
    sender: mpsc::Sender<SqliteCommand>,
    interrupt_handle: Arc<rusqlite::InterruptHandle>,
}

impl SqliteHandle {
    /// Execute a parameterised query, returning a full `QueryResult`.
    pub async fn execute(
        &self,
        sql: String,
        params: Vec<QueryParam>,
        max_rows: u64,
        timeout_ms: u64,
    ) -> Result<QueryResult, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Execute {
            sql,
            params,
            max_rows,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        self.await_result(rx, timeout_ms).await
    }

    /// Interrupt the active SQLite VM and wait until the actor has processed
    /// the interrupt command. The acknowledgement is the recovery boundary:
    /// callers must not report cancellation before the actor is ready again.
    pub async fn cancel(&self, timeout_ms: u64) -> Result<(), DbError> {
        self.interrupt_handle.interrupt();
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Interrupt { responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;

        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(DbError::Internal(format!("cancel acknowledgement failed: {error}"))),
            Err(_) => Err(DbError::Internal(
                "SQLite actor did not acknowledge query cancellation".into(),
            )),
        }
    }

    /// Run full schema introspection.
    pub async fn introspect(&self, timeout_ms: u64) -> Result<IntrospectResult, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Introspect { responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        self.await_result(rx, timeout_ms).await
    }

    /// Return an `EXPLAIN QUERY PLAN` result as JSON.
    pub async fn explain(&self, sql: String, timeout_ms: u64) -> Result<serde_json::Value, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Explain { sql, responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        self.await_result(rx, timeout_ms).await
    }

    /// Execute a raw SQL string with already-stringified params (for meta CRUD).
    pub async fn raw_query(&self, sql: String, params: Vec<String>) -> Result<Vec<Vec<String>>, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::RawQuery {
            sql,
            params,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
    }

    /// Execute a statement (INSERT / UPDATE / DELETE) and return rows affected.
    pub async fn execute_statement(&self, sql: String) -> Result<usize, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteStatement { sql, responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
    }

    /// Execute a parameterized metadata statement and return rows affected.
    pub async fn execute_param(&self, sql: String, params: Vec<QueryParam>) -> Result<usize, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteStatementParam {
            sql,
            params,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
    }

    /// Execute a parameterized database statement with a bounded timeout.
    pub async fn execute_param_with_timeout(
        &self,
        sql: String,
        params: Vec<QueryParam>,
        timeout_ms: u64,
    ) -> Result<usize, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteStatementParam {
            sql,
            params,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        self.await_result(rx, timeout_ms).await
    }

    /// Execute multiple statements atomically inside a transaction.
    pub async fn execute_batch(&self, statements: Vec<String>, timeout_ms: u64) -> Result<u64, DbError> {
        let (tx, mut rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteBatch {
            statements,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), &mut rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(DbError::Internal(format!("oneshot recv error: {error}"))),
            Err(_) => {
                self.interrupt_handle.interrupt();
                // The actor owns the SQLite transaction. Keep the receiver alive
                // until it has handled the interrupt and rolled back, otherwise a
                // following command can race the still-open transaction.
                match rx.await {
                    Ok(Ok(result)) => Ok(result),
                    Ok(Err(error)) => {
                        if matches!(&error, DbError::Internal(message) if message.contains("rollback failed")) {
                            Err(error)
                        } else {
                            Err(DbError::QueryTimeout { timeout_ms })
                        }
                    }
                    Err(error) => Err(DbError::Internal(format!(
                        "oneshot recv error after interrupt: {error}"
                    ))),
                }
            }
        }
    }

    pub async fn execute_transaction(
        &self,
        statements: Vec<String>,
        read_statements: Vec<bool>,
        max_rows: u64,
        timeout_ms: u64,
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        let (tx, mut rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteTransaction {
            statements,
            read_statements,
            max_rows,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| TransactionFailure {
            phase: TransactionFailurePhase::Validation,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: DbError::Internal(format!("spawn_blocking join error: {e}")),
        })?;
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), &mut rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(TransactionFailure {
                phase: TransactionFailurePhase::Validation,
                statement_index: 0,
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Internal(format!("oneshot recv error: {error}")),
            }),
            Err(_) => {
                self.interrupt_handle.interrupt();
                // Keep the receiver alive and wait for the actor to finish its
                // explicit rollback. Returning immediately would violate the
                // DbConnector transaction contract and could race the next command.
                match rx.await {
                    Ok(Ok(results)) => Ok(results),
                    Ok(Err(mut failure)) => {
                        if !matches!(&failure.error, DbError::Internal(message) if message.contains("rollback failed"))
                        {
                            failure.error = DbError::QueryTimeout { timeout_ms };
                        }
                        Err(failure)
                    }
                    Err(error) => Err(TransactionFailure {
                        phase: TransactionFailurePhase::Validation,
                        statement_index: 0,
                        outcome: TransactionFailureOutcome::Unknown,
                        results: Vec::new(),
                        error: DbError::Internal(format!("oneshot recv error after interrupt: {error}")),
                    }),
                }
            }
        }
    }

    pub async fn execute_parameterized_transaction(
        &self,
        statements: Vec<ParameterizedTransactionStatement>,
        timeout_ms: u64,
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        let (tx, mut rx) = oneshot::channel();
        let cmd = SqliteCommand::ExecuteParameterizedTransaction {
            statements,
            responder: tx,
        };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|error| TransactionFailure {
            phase: TransactionFailurePhase::Validation,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: DbError::Internal(format!("spawn_blocking join error: {error}")),
        })?;
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), &mut rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(TransactionFailure {
                phase: TransactionFailurePhase::Validation,
                statement_index: 0,
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Internal(format!("oneshot recv error: {error}")),
            }),
            Err(_) => {
                self.interrupt_handle.interrupt();
                match rx.await {
                    Ok(result) => result,
                    Err(error) => Err(TransactionFailure {
                        phase: TransactionFailurePhase::Validation,
                        statement_index: 0,
                        outcome: TransactionFailureOutcome::Unknown,
                        results: Vec::new(),
                        error: DbError::Internal(format!("oneshot recv error after interrupt: {error}")),
                    }),
                }
            }
        }
    }

    async fn await_result<T>(
        &self,
        receiver: oneshot::Receiver<Result<T, DbError>>,
        timeout_ms: u64,
    ) -> Result<T, DbError> {
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(DbError::Internal(format!("oneshot recv error: {error}"))),
            Err(_) => {
                self.interrupt_handle.interrupt();
                Err(DbError::QueryTimeout { timeout_ms })
            }
        }
    }

    /// Tell the actor thread to shut down.
    pub async fn shutdown(&self) {
        let sender = self.sender.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let _ = sender.send(SqliteCommand::Shutdown);
        })
        .await;
    }
}

// ---------------------------------------------------------------------------
// SqliteActor – owns the connection on a dedicated std::thread
// ---------------------------------------------------------------------------

pub struct SqliteActor {
    conn: rusqlite::Connection,
}

impl SqliteActor {
    /// Open (or create) the database at `db_path`, spawn the actor thread,
    /// and return a clonable handle for sending commands.
    pub fn spawn(db_path: &str) -> Result<SqliteHandle, DbError> {
        let conn = rusqlite::Connection::open(db_path).map_err(crate::error::from_rusqlite)?;
        let interrupt_handle = Arc::new(conn.get_interrupt_handle());

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            Self { conn }.run(receiver);
        });

        Ok(SqliteHandle {
            sender,
            interrupt_handle,
        })
    }

    // -- main loop ----------------------------------------------------------

    fn run(self, receiver: mpsc::Receiver<SqliteCommand>) {
        for cmd in receiver {
            match cmd {
                SqliteCommand::Execute {
                    sql,
                    params,
                    max_rows,
                    responder,
                } => {
                    let _ = responder.send(self.handle_execute(&sql, &params, max_rows));
                }
                SqliteCommand::Interrupt { responder } => {
                    let _ = responder.send(Ok(()));
                }
                SqliteCommand::Introspect { responder } => {
                    let _ = responder.send(self.handle_introspect());
                }
                SqliteCommand::Explain { sql, responder } => {
                    let _ = responder.send(self.handle_explain(&sql));
                }
                SqliteCommand::RawQuery { sql, params, responder } => {
                    let _ = responder.send(self.handle_raw_query(&sql, &params));
                }
                SqliteCommand::ExecuteStatement { sql, responder } => {
                    let _ = responder.send(self.handle_execute_statement(&sql));
                }
                SqliteCommand::ExecuteStatementParam { sql, params, responder } => {
                    let _ = responder.send(self.handle_execute_statement_param(&sql, &params));
                }
                SqliteCommand::ExecuteBatch { statements, responder } => {
                    let _ = responder.send(self.handle_execute_batch(&statements));
                }
                SqliteCommand::ExecuteTransaction {
                    statements,
                    read_statements,
                    max_rows,
                    responder,
                } => {
                    let _ = responder.send(self.handle_execute_transaction(&statements, &read_statements, max_rows));
                }
                SqliteCommand::ExecuteParameterizedTransaction { statements, responder } => {
                    let _ = responder.send(self.handle_execute_parameterized_transaction(&statements));
                }
                SqliteCommand::Shutdown => {
                    tracing::info!("sqlite actor received shutdown command");
                    break;
                }
            }
        }
        tracing::info!("sqlite actor thread exiting");
    }

    // -- handlers -----------------------------------------------------------

    #[allow(clippy::result_large_err)]
    fn handle_execute_parameterized_transaction(
        &self,
        statements: &[ParameterizedTransactionStatement],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        let tx = self.conn.unchecked_transaction().map_err(|error| TransactionFailure {
            phase: TransactionFailurePhase::Begin,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: crate::error::from_rusqlite(error),
        })?;
        let mut results = Vec::with_capacity(statements.len());
        for (index, statement) in statements.iter().enumerate() {
            let params = to_rusqlite_params(&statement.params);
            let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|param| param.as_ref()).collect();
            let result = tx
                .execute(&statement.sql, refs.as_slice())
                .map(|affected| (affected as u64, Instant::now()))
                .map_err(crate::error::from_rusqlite);
            let affected = match result {
                Ok((affected, _)) => affected,
                Err(error) => {
                    let (error, outcome) = match tx.rollback().err().map(crate::error::from_rusqlite) {
                        Some(rollback_error) => (
                            DbError::Internal(format!("statement failed: {error}; rollback failed: {rollback_error}")),
                            TransactionFailureOutcome::Unknown,
                        ),
                        None => (error, TransactionFailureOutcome::RolledBack),
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
                let (error, outcome) = match tx.rollback().err().map(crate::error::from_rusqlite) {
                    Some(rollback_error) => (
                        DbError::Internal(format!("mutation affected no rows; rollback failed: {rollback_error}")),
                        TransactionFailureOutcome::Unknown,
                    ),
                    None => (error, TransactionFailureOutcome::RolledBack),
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
                let (error, outcome) = match tx.rollback().err().map(crate::error::from_rusqlite) {
                    Some(rollback_error) => (
                        DbError::Internal(format!(
                            "mutation invariant failed: {error}; rollback failed: {rollback_error}"
                        )),
                        TransactionFailureOutcome::Unknown,
                    ),
                    None => (error, TransactionFailureOutcome::RolledBack),
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
        if let Err(error) = tx.commit() {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Commit,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::Unknown,
                results,
                error: crate::error::from_rusqlite(error),
            });
        }
        Ok(results)
    }

    #[allow(clippy::result_large_err)]
    fn handle_execute_transaction(
        &self,
        statements: &[String],
        read_statements: &[bool],
        max_rows: u64,
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        if statements.len() != read_statements.len() {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Validation,
                statement_index: 0,
                outcome: TransactionFailureOutcome::NotStarted,
                results: Vec::new(),
                error: DbError::Internal("transaction statement metadata length mismatch".into()),
            });
        }

        let tx = self.conn.unchecked_transaction().map_err(|error| TransactionFailure {
            phase: TransactionFailurePhase::Begin,
            statement_index: 0,
            outcome: TransactionFailureOutcome::NotStarted,
            results: Vec::new(),
            error: crate::error::from_rusqlite(error),
        })?;
        let mut results = Vec::with_capacity(statements.len());

        for (index, (statement, is_read)) in statements.iter().zip(read_statements).enumerate() {
            let started = Instant::now();
            let result = if *is_read {
                query_transaction(&tx, statement, max_rows, started)
            } else {
                tx.execute_batch(statement)
                    .map(|_| TransactionStatementResult::Affected {
                        row_count: tx.changes(),
                        duration_ms: started.elapsed().as_millis() as u64,
                    })
                    .map_err(crate::error::from_rusqlite)
            };

            match result {
                Ok(result) => results.push(result),
                Err(error) => {
                    let (error, outcome) = match tx.rollback().err().map(crate::error::from_rusqlite) {
                        Some(rollback_error) => (
                            DbError::Internal(format!("statement failed: {error}; rollback failed: {rollback_error}")),
                            TransactionFailureOutcome::Unknown,
                        ),
                        None => (error, TransactionFailureOutcome::RolledBack),
                    };
                    return Err(TransactionFailure {
                        phase: TransactionFailurePhase::Statement,
                        statement_index: index,
                        outcome,
                        results,
                        error,
                    });
                }
            }
        }

        if let Err(error) = tx.commit() {
            return Err(TransactionFailure {
                phase: TransactionFailurePhase::Commit,
                statement_index: statements.len(),
                outcome: TransactionFailureOutcome::Unknown,
                results,
                error: crate::error::from_rusqlite(error),
            });
        }
        Ok(results)
    }

    fn handle_execute(&self, sql: &str, params: &[QueryParam], max_rows: u64) -> Result<QueryResult, DbError> {
        let start = Instant::now();

        let mut stmt = self.conn.prepare(sql).map_err(crate::error::from_rusqlite)?;

        let columns = extract_columns(&stmt);

        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = rusqlite_params.iter().map(|p| p.as_ref()).collect();

        let mut rows = Vec::new();
        let mut raw_rows = stmt.query(param_refs.as_slice()).map_err(crate::error::from_rusqlite)?;

        while let Some(row) = raw_rows.next().map_err(crate::error::from_rusqlite)? {
            if rows.len() as u64 >= max_rows {
                break;
            }
            let cells = map_row_to_cells(row)?;
            rows.push(Row(cells));
        }

        let row_count = rows.len() as u64;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            columns,
            rows,
            row_count,
            duration_ms,
        })
    }

    fn handle_introspect(&self) -> Result<IntrospectResult, DbError> {
        super::introspect::run_introspection(&self.conn)
    }

    fn handle_explain(&self, sql: &str) -> Result<serde_json::Value, DbError> {
        let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
        let mut stmt = self.conn.prepare(&explain_sql).map_err(crate::error::from_rusqlite)?;

        let col_names: Vec<String> = stmt.column_names().iter().map(|n| n.to_string()).collect();

        let mut raw_rows = stmt.query([]).map_err(crate::error::from_rusqlite)?;

        let mut result = Vec::new();
        while let Some(row) = raw_rows.next().map_err(crate::error::from_rusqlite)? {
            let mut map = serde_json::Map::new();
            for (i, col_name) in col_names.iter().enumerate() {
                let value = row.get_ref(i).map_err(crate::error::from_rusqlite)?;
                let json_val = match value {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Real(v) => serde_json::json!(v),
                    rusqlite::types::ValueRef::Text(v) => {
                        serde_json::json!(String::from_utf8_lossy(v))
                    }
                    rusqlite::types::ValueRef::Blob(v) => {
                        serde_json::json!(v.iter().collect::<Vec<_>>())
                    }
                };
                map.insert(col_name.clone(), json_val);
            }
            result.push(serde_json::Value::Object(map));
        }

        Ok(serde_json::Value::Array(result))
    }

    fn handle_raw_query(&self, sql: &str, params: &[String]) -> Result<Vec<Vec<String>>, DbError> {
        let mut stmt = self.conn.prepare(sql).map_err(crate::error::from_rusqlite)?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

        let col_count = stmt.column_count();

        let mut raw_rows = stmt.query(param_refs.as_slice()).map_err(crate::error::from_rusqlite)?;
        let mut result = Vec::new();

        while let Some(row) = raw_rows.next().map_err(crate::error::from_rusqlite)? {
            let mut row_strings = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let value = row.get_ref(i).map_err(crate::error::from_rusqlite)?;
                let s = match value {
                    rusqlite::types::ValueRef::Null => String::new(),
                    rusqlite::types::ValueRef::Integer(v) => v.to_string(),
                    rusqlite::types::ValueRef::Real(v) => v.to_string(),
                    rusqlite::types::ValueRef::Text(v) => String::from_utf8_lossy(v).to_string(),
                    rusqlite::types::ValueRef::Blob(v) => {
                        format!("<blob {} bytes>", v.len())
                    }
                };
                row_strings.push(s);
            }
            result.push(row_strings);
        }

        Ok(result)
    }

    fn handle_execute_statement(&self, sql: &str) -> Result<usize, DbError> {
        self.conn.execute_batch(sql).map_err(crate::error::from_rusqlite)?;
        // Note: changes() reflects only the last statement in the batch.
        Ok(self.conn.changes() as usize)
    }

    fn handle_execute_statement_param(&self, sql: &str, params: &[QueryParam]) -> Result<usize, DbError> {
        let mut stmt = self.conn.prepare(sql).map_err(crate::error::from_rusqlite)?;
        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = rusqlite_params.iter().map(|p| p.as_ref()).collect();
        let affected = stmt
            .execute(param_refs.as_slice())
            .map_err(crate::error::from_rusqlite)?;
        Ok(affected)
    }

    fn handle_execute_batch(&self, statements: &[String]) -> Result<u64, DbError> {
        let tx = self.conn.unchecked_transaction().map_err(crate::error::from_rusqlite)?;
        let mut total: u64 = 0;
        for stmt_sql in statements {
            if let Err(error) = tx.execute_batch(stmt_sql).map_err(crate::error::from_rusqlite) {
                let rollback_error = tx.rollback().err().map(crate::error::from_rusqlite);
                return match rollback_error {
                    Some(rollback_error) => Err(DbError::Internal(format!(
                        "batch statement failed: {error}; rollback failed: {rollback_error}"
                    ))),
                    None => Err(error),
                };
            }
            total = match total.checked_add(tx.changes()) {
                Some(total) => total,
                None => {
                    let error = DbError::Internal("batch affected-row count overflow".into());
                    let rollback_error = tx.rollback().err().map(crate::error::from_rusqlite);
                    return match rollback_error {
                        Some(rollback_error) => {
                            Err(DbError::Internal(format!("{error}; rollback failed: {rollback_error}")))
                        }
                        None => Err(error),
                    };
                }
            };
        }
        tx.commit().map_err(crate::error::from_rusqlite)?;
        Ok(total)
    }
}

fn query_transaction(
    tx: &rusqlite::Transaction<'_>,
    sql: &str,
    max_rows: u64,
    started: Instant,
) -> Result<TransactionStatementResult, DbError> {
    let mut stmt = tx.prepare(sql).map_err(crate::error::from_rusqlite)?;
    let columns = extract_columns(&stmt);
    let mut rows = Vec::new();
    let mut raw_rows = stmt.query([]).map_err(crate::error::from_rusqlite)?;
    while (rows.len() as u64) < max_rows {
        let row = match raw_rows.next().map_err(crate::error::from_rusqlite)? {
            Some(row) => row,
            None => break,
        };
        rows.push(Row(map_row_to_cells(row)?));
    }
    Ok(TransactionStatementResult::Query(QueryResult {
        columns,
        row_count: rows.len() as u64,
        rows,
        duration_ms: started.elapsed().as_millis() as u64,
    }))
}
