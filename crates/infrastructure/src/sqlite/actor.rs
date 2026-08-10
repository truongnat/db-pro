use std::sync::mpsc;
use std::time::Instant;

use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam, QueryResult, Row};
use db_pro_core::domain::schema::IntrospectResult;
use tokio::sync::oneshot;
use tracing;

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
    Shutdown,
}

// ---------------------------------------------------------------------------
// SqliteHandle – cheap-to-clone, async-facing handle
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SqliteHandle {
    sender: mpsc::Sender<SqliteCommand>,
}

impl SqliteHandle {
    /// Execute a parameterised query, returning a full `QueryResult`.
    pub async fn execute(&self, sql: String, params: Vec<QueryParam>, max_rows: u64) -> Result<QueryResult, DbError> {
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
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
    }

    /// Run full schema introspection.
    pub async fn introspect(&self) -> Result<IntrospectResult, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Introspect { responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
    }

    /// Return an `EXPLAIN QUERY PLAN` result as JSON.
    pub async fn explain(&self, sql: String) -> Result<serde_json::Value, DbError> {
        let (tx, rx) = oneshot::channel();
        let cmd = SqliteCommand::Explain { sql, responder: tx };
        let sender = self.sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(cmd);
        })
        .await
        .map_err(|e| DbError::Internal(format!("spawn_blocking join error: {e}")))?;
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
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

    /// Execute a parameterized statement and return rows affected.
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

    /// Execute multiple statements atomically inside a transaction.
    pub async fn execute_batch(&self, statements: Vec<String>) -> Result<u64, DbError> {
        let (tx, rx) = oneshot::channel();
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
        rx.await
            .map_err(|e| DbError::Internal(format!("oneshot recv error: {e}")))?
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

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            Self { conn }.run(receiver);
        });

        Ok(SqliteHandle { sender })
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
                SqliteCommand::Shutdown => {
                    tracing::info!("sqlite actor received shutdown command");
                    break;
                }
            }
        }
        tracing::info!("sqlite actor thread exiting");
    }

    // -- handlers -----------------------------------------------------------

    fn handle_execute(&self, sql: &str, params: &[QueryParam], max_rows: u64) -> Result<QueryResult, DbError> {
        let start = Instant::now();

        let mut stmt = self.conn.prepare(sql).map_err(crate::error::from_rusqlite)?;

        let col_count = stmt.column_count();
        let columns: Vec<ColumnMeta> = (0..col_count)
            .map(|i| {
                let name = stmt.column_name(i).unwrap_or("?").to_string();
                let data_type = stmt.column_names().get(i).map(|_| "TEXT").unwrap_or("TEXT").to_string();
                ColumnMeta {
                    name,
                    data_type,
                    nullable: true,
                }
            })
            .collect();

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
            tx.execute_batch(stmt_sql).map_err(crate::error::from_rusqlite)?;
            total += tx.changes();
        }
        tx.commit().map_err(crate::error::from_rusqlite)?;
        Ok(total)
    }
}

// ---------------------------------------------------------------------------
// Inline helpers (will be replaced by query_mapper module once available)
// ---------------------------------------------------------------------------

/// Convert domain `QueryParam` values into boxed `ToSql` trait objects
/// suitable for binding to a `rusqlite::Statement`.
fn to_rusqlite_params(params: &[QueryParam]) -> Vec<Box<dyn rusqlite::types::ToSql>> {
    params
        .iter()
        .map(|p| -> Box<dyn rusqlite::types::ToSql> {
            match p {
                QueryParam::Null => Box::new(rusqlite::types::Null),
                QueryParam::Bool(v) => Box::new(*v),
                QueryParam::Int64(v) => Box::new(*v),
                QueryParam::Float64(v) => Box::new(*v),
                QueryParam::Text(v) => Box::new(v.clone()),
                QueryParam::Bytes(v) => Box::new(v.clone()),
                QueryParam::Uuid(v) => Box::new(v.clone()),
                QueryParam::DateTime(v) => Box::new(v.clone()),
                QueryParam::Json(v) => Box::new(v.to_string()),
            }
        })
        .collect()
}

/// Map a single `rusqlite::Row` to a `Vec<CellValue>` by iterating columns
/// and inspecting the runtime type of each value.
fn map_row_to_cells(row: &rusqlite::Row) -> Result<Vec<CellValue>, DbError> {
    let mut cells = Vec::new();
    for i in 0..row.as_ref().column_count() {
        let value = row.get_ref(i).map_err(crate::error::from_rusqlite)?;
        let cell = match value {
            rusqlite::types::ValueRef::Null => CellValue::Null,
            rusqlite::types::ValueRef::Integer(v) => CellValue::Int64(v),
            rusqlite::types::ValueRef::Real(v) => CellValue::Float64(v),
            rusqlite::types::ValueRef::Text(v) => CellValue::Text(String::from_utf8_lossy(v).to_string()),
            rusqlite::types::ValueRef::Blob(v) => CellValue::Bytes(v.to_vec()),
        };
        cells.push(cell);
    }
    Ok(cells)
}
