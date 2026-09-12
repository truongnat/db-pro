use std::collections::HashSet;
use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::ports::{ConnectionRepository, DbConnector};

use super::registry::ConnectionRegistry;
use super::sql_policy::reject_multi_statement;

#[derive(Debug)]
pub struct ExportResult {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
    pub row_count: u64,
}

pub struct ExportService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl ExportService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            registry,
            connections,
        }
    }

    async fn execute_for_export(&self, connection_id: &ConnectionId, sql: &str) -> Result<QueryResult, DbError> {
        reject_multi_statement(sql)?;

        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        let policy = if config.readonly {
            ConnectionSafetyPolicy::read_only()
        } else {
            ConnectionSafetyPolicy::full_access()
        };
        validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let result = self.connector.query(&handle, sql, &[]).await?;
        result.validate().map_err(DbError::QueryFailed)?;
        Ok(result)
    }

    pub async fn export_csv(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        let mut writer = csv::Writer::from_writer(Vec::new());

        let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
        writer
            .write_record(&headers)
            .map_err(|e| DbError::Internal(format!("csv header write failed: {e}")))?;

        for row in &result.rows {
            let fields: Vec<String> = row.0.iter().map(cell_to_csv_string).collect();
            let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
            writer
                .write_record(&refs)
                .map_err(|e| DbError::Internal(format!("csv row write failed: {e}")))?;
        }

        let content = writer
            .into_inner()
            .map_err(|e| DbError::Internal(format!("csv flush failed: {e}")))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.csv"),
            mime_type: "text/csv".into(),
            row_count: result.row_count,
        })
    }

    pub async fn export_json(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        validate_json_column_names(&result)?;

        let rows: Vec<serde_json::Map<String, serde_json::Value>> = result
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (col, cell) in result.columns.iter().zip(row.0.iter()) {
                    map.insert(col.name.clone(), cell_to_json(cell)?);
                }
                Ok(map)
            })
            .collect::<Result<_, DbError>>()?;

        let content = serde_json::to_vec_pretty(&rows)
            .map_err(|e| DbError::Internal(format!("json serialization failed: {e}")))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.json"),
            mime_type: "application/json".into(),
            row_count: result.row_count,
        })
    }

    pub async fn export_excel(&self, connection_id: &ConnectionId, sql: &str) -> Result<ExportResult, DbError> {
        let result = self.execute_for_export(connection_id, sql).await?;
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = rust_xlsxwriter::Format::new().set_bold();

        for (col_idx, col) in result.columns.iter().enumerate() {
            let col_idx = excel_column_index(col_idx)?;
            worksheet
                .write_string_with_format(0, col_idx, &col.name, &header_format)
                .map_err(|e| DbError::Internal(format!("excel header write failed: {e}")))?;
        }

        for (row_idx, row) in result.rows.iter().enumerate() {
            let row_idx = excel_row_index(row_idx)?;
            for (col_idx, cell) in row.0.iter().enumerate() {
                let col_idx = excel_column_index(col_idx)?;
                write_excel_cell(worksheet, row_idx, col_idx, cell)?;
            }
        }

        let content = workbook
            .save_to_buffer()
            .map_err(|e| DbError::Internal(format!("excel save failed: {e}")))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Ok(ExportResult {
            content,
            filename: format!("export_{timestamp}.xlsx"),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
            row_count: result.row_count,
        })
    }
}

fn excel_column_index(index: usize) -> Result<u16, DbError> {
    u16::try_from(index).map_err(|_| DbError::Validation("Excel export has too many columns".into()))
}

fn excel_row_index(index: usize) -> Result<u32, DbError> {
    index
        .checked_add(1)
        .and_then(|index| u32::try_from(index).ok())
        .ok_or_else(|| DbError::Validation("Excel export has too many rows".into()))
}

fn validate_json_column_names(result: &QueryResult) -> Result<(), DbError> {
    let mut names = HashSet::with_capacity(result.columns.len());
    for column in &result.columns {
        if !names.insert(column.name.as_str()) {
            return Err(DbError::Validation(format!(
                "JSON export requires unique column names; duplicate column: {}",
                column.name
            )));
        }
    }
    Ok(())
}

const MAX_EXACT_EXCEL_INTEGER: i64 = 1_i64 << 53;

fn excel_integer_is_exact(value: i64) -> bool {
    (-MAX_EXACT_EXCEL_INTEGER..=MAX_EXACT_EXCEL_INTEGER).contains(&value)
}

fn cell_to_csv_string(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(b) => b.to_string(),
        CellValue::Int64(i) => i.to_string(),
        CellValue::Float64(f) => f.to_string(),
        CellValue::Decimal(s) => s.clone(),
        CellValue::Text(s) => s.clone(),
        CellValue::Bytes(_) => "[binary]".into(),
        CellValue::Uuid(s) => s.clone(),
        CellValue::DateTime(s) => s.clone(),
        CellValue::Date(s) => s.clone(),
        CellValue::Time(s) => s.clone(),
        CellValue::Interval(s) => s.clone(),
        CellValue::Inet(s) => s.clone(),
        CellValue::Json(v) => v.to_string(),
    }
}

fn cell_to_json(cell: &CellValue) -> Result<serde_json::Value, DbError> {
    match cell {
        CellValue::Null => Ok(serde_json::Value::Null),
        CellValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        CellValue::Int64(i) => Ok(serde_json::Value::Number((*i).into())),
        CellValue::Float64(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| DbError::Validation("JSON export cannot represent a non-finite float".into())),
        // Keep decimal text exact instead of converting through f64.
        CellValue::Decimal(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Text(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Bytes(b) => Ok(serde_json::Value::String(format!("[{} bytes]", b.len()))),
        CellValue::Uuid(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::DateTime(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Date(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Time(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Interval(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Inet(s) => Ok(serde_json::Value::String(s.clone())),
        CellValue::Json(v) => Ok(v.clone()),
    }
}

fn write_excel_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    cell: &CellValue,
) -> Result<(), DbError> {
    match cell {
        CellValue::Null => Ok(()),
        CellValue::Bool(b) => worksheet
            .write_boolean(row, col, *b)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Int64(i) if excel_integer_is_exact(*i) => worksheet
            .write_number(row, col, *i as f64)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Int64(i) => worksheet
            .write_string(row, col, i.to_string())
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Float64(f) => worksheet
            .write_number(row, col, *f)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Decimal(s) => worksheet
            .write_string(row, col, s)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Text(s) | CellValue::Uuid(s) | CellValue::DateTime(s) => worksheet
            .write_string(row, col, s)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Date(s) | CellValue::Time(s) | CellValue::Interval(s) | CellValue::Inet(s) => worksheet
            .write_string(row, col, s)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Bytes(b) => worksheet
            .write_string(row, col, format!("[{} bytes]", b.len()))
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Json(v) => worksheet
            .write_string(row, col, v.to_string())
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType, SslMode};
    use crate::domain::query::{ColumnMeta, Row};
    use crate::ports::{MockConnectionRepository, MockDbConnector};

    fn sample_result() -> QueryResult {
        QueryResult {
            columns: vec![
                ColumnMeta {
                    name: "id".into(),
                    data_type: "INT".into(),
                    nullable: false,
                },
                ColumnMeta {
                    name: "name".into(),
                    data_type: "TEXT".into(),
                    nullable: true,
                },
            ],
            rows: vec![
                Row(vec![CellValue::Int64(1), CellValue::Text("Alice".into())]),
                Row(vec![CellValue::Int64(2), CellValue::Null]),
            ],
            row_count: 2,
            duration_ms: 0,
        }
    }

    #[test]
    fn excel_indexes_reject_provider_type_overflow() {
        assert!(matches!(excel_column_index(u16::MAX as usize), Ok(u16::MAX)));
        assert!(matches!(
            excel_column_index(u16::MAX as usize + 1),
            Err(DbError::Validation(message)) if message.contains("columns")
        ));
        assert!(matches!(excel_row_index(u32::MAX as usize - 1), Ok(u32::MAX)));
        assert!(matches!(
            excel_row_index(u32::MAX as usize),
            Err(DbError::Validation(message)) if message.contains("rows")
        ));
    }

    #[test]
    fn excel_integer_precision_switches_to_text_outside_exact_f64_range() {
        assert!(excel_integer_is_exact(MAX_EXACT_EXCEL_INTEGER));
        assert!(excel_integer_is_exact(-MAX_EXACT_EXCEL_INTEGER));
        assert!(!excel_integer_is_exact(MAX_EXACT_EXCEL_INTEGER + 1));
        assert!(!excel_integer_is_exact(i64::MAX));
        assert!(!excel_integer_is_exact(i64::MIN));
    }

    fn build_service(connector: MockDbConnector, registry: Arc<ConnectionRegistry>) -> ExportService {
        let mut connections = MockConnectionRepository::new();
        connections.expect_get_config().returning(|_| {
            Ok(Some(ConnectionConfig {
                name: "test".into(),
                host: "localhost".into(),
                port: 5432,
                database: "testdb".into(),
                username: "user".into(),
                driver: DriverType::Postgres,
                ssl_mode: SslMode::Disable,
                ssh_tunnel: None,
                query_timeout_ms: 30_000,
                max_rows: 500,
                color: None,
                tags: vec![],
                group: None,
                readonly: false,
            }))
        });
        ExportService::new(Box::new(connector), registry, Box::new(connections))
    }

    #[tokio::test]
    async fn export_csv_basic() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(sample_result()));

        let svc = build_service(connector, Arc::clone(&registry));
        let result = svc.export_csv(&conn_id, "SELECT * FROM users").await.unwrap();

        let content = String::from_utf8(result.content).unwrap();
        assert!(content.contains("id,name"));
        assert!(content.contains("1,Alice"));
        assert!(content.contains("2,"));
        assert_eq!(result.mime_type, "text/csv");
        assert_eq!(result.row_count, 2);
    }

    #[tokio::test]
    async fn export_json_basic() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(sample_result()));

        let svc = build_service(connector, Arc::clone(&registry));
        let result = svc.export_json(&conn_id, "SELECT * FROM users").await.unwrap();

        let parsed: Vec<serde_json::Value> = serde_json::from_slice(&result.content).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[0]["name"], "Alice");
        assert_eq!(parsed[1]["name"], serde_json::Value::Null);
        assert_eq!(result.mime_type, "application/json");
    }

    #[tokio::test]
    async fn export_json_rejects_duplicate_column_names() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let duplicate_columns = QueryResult {
            columns: vec![
                ColumnMeta {
                    name: "value".into(),
                    data_type: "INT".into(),
                    nullable: false,
                },
                ColumnMeta {
                    name: "value".into(),
                    data_type: "INT".into(),
                    nullable: false,
                },
            ],
            rows: vec![Row(vec![CellValue::Int64(1), CellValue::Int64(2)])],
            row_count: 1,
            duration_ms: 0,
        };
        let mut connector = MockDbConnector::new();
        connector
            .expect_query()
            .returning(move |_, _, _| Ok(duplicate_columns.clone()));

        let error = build_service(connector, registry)
            .export_json(&conn_id, "SELECT 1 AS value, 2 AS value")
            .await
            .expect_err("JSON export must not drop a duplicate column");

        assert!(matches!(error, DbError::Validation(message)
            if message.contains("duplicate column") && message.contains("value")));
    }

    #[tokio::test]
    async fn export_json_rejects_non_finite_float() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let non_finite = QueryResult {
            columns: vec![ColumnMeta {
                name: "value".into(),
                data_type: "FLOAT8".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Float64(f64::NAN)])],
            row_count: 1,
            duration_ms: 0,
        };
        let mut connector = MockDbConnector::new();
        connector
            .expect_query()
            .returning(move |_, _, _| Ok(non_finite.clone()));

        let error = build_service(connector, registry)
            .export_json(&conn_id, "SELECT 'NaN'::float8 AS value")
            .await
            .expect_err("JSON export must not silently turn NaN into null");

        assert!(matches!(error, DbError::Validation(message) if message.contains("non-finite")));
    }

    #[tokio::test]
    async fn export_excel_basic() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(|_, _, _| Ok(sample_result()));

        let svc = build_service(connector, Arc::clone(&registry));
        let result = svc.export_excel(&conn_id, "SELECT * FROM users").await.unwrap();

        assert!(result.content.len() > 4);
        assert_eq!(&result.content[0..4], b"PK\x03\x04");
        assert_eq!(result.row_count, 2);
    }

    #[tokio::test]
    async fn export_not_active() {
        let svc = build_service(MockDbConnector::new(), Arc::new(ConnectionRegistry::new()));
        let result = svc.export_csv(&ConnectionId::new(), "SELECT 1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn export_multi_statement_rejected() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let svc = build_service(MockDbConnector::new(), Arc::clone(&registry));
        let result = svc.export_csv(&conn_id, "SELECT 1; SELECT 2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn export_rejects_mutating_query_on_readonly_connection() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut connections = MockConnectionRepository::new();
        connections.expect_get_config().returning(|_| {
            Ok(Some(ConnectionConfig {
                name: "readonly".into(),
                host: "localhost".into(),
                port: 5432,
                database: "testdb".into(),
                username: "user".into(),
                driver: DriverType::Postgres,
                ssl_mode: SslMode::Disable,
                ssh_tunnel: None,
                query_timeout_ms: 30_000,
                max_rows: 500,
                color: None,
                tags: vec![],
                group: None,
                readonly: true,
            }))
        });

        let svc = ExportService::new(Box::new(MockDbConnector::new()), registry, Box::new(connections));
        let error = svc
            .export_csv(&conn_id, "DELETE FROM users RETURNING id")
            .await
            .expect_err("readonly export must not execute a mutation");
        assert!(matches!(error, DbError::QueryFailed(message) if message.contains("read-only")));
    }

    #[tokio::test]
    async fn export_rejects_malformed_query_result_shape() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let malformed = QueryResult {
            columns: vec![ColumnMeta {
                name: "id".into(),
                data_type: "INT".into(),
                nullable: false,
            }],
            rows: vec![Row(Vec::new())],
            row_count: 1,
            duration_ms: 0,
        };
        let mut connector = MockDbConnector::new();
        connector.expect_query().returning(move |_, _, _| Ok(malformed.clone()));

        let error = build_service(connector, registry)
            .export_json(&conn_id, "SELECT id FROM users")
            .await
            .expect_err("export must reject an invalid provider result");

        assert!(matches!(error, DbError::QueryFailed(message) if message.contains("expected 1")));
    }
}
