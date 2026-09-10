use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::ports::DbConnector;

use super::registry::ConnectionRegistry;
use super::sql_policy::reject_multi_statement;

pub struct ExportResult {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
    pub row_count: u64,
}

pub struct ExportService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
}

impl ExportService {
    pub fn new(connector: Box<dyn DbConnector>, registry: Arc<ConnectionRegistry>) -> Self {
        Self { connector, registry }
    }

    async fn execute_for_export(&self, connection_id: &ConnectionId, sql: &str) -> Result<QueryResult, DbError> {
        reject_multi_statement(sql)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        self.connector.query(&handle, sql, &[]).await
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

        let rows: Vec<serde_json::Map<String, serde_json::Value>> = result
            .rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (col, cell) in result.columns.iter().zip(row.0.iter()) {
                    map.insert(col.name.clone(), cell_to_json(cell));
                }
                map
            })
            .collect();

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
            let col_idx = col_idx as u16;
            worksheet
                .write_string_with_format(0, col_idx, &col.name, &header_format)
                .map_err(|e| DbError::Internal(format!("excel header write failed: {e}")))?;
        }

        for (row_idx, row) in result.rows.iter().enumerate() {
            let row_idx = (row_idx + 1) as u32;
            for (col_idx, cell) in row.0.iter().enumerate() {
                let col_idx = col_idx as u16;
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

fn cell_to_csv_string(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(b) => b.to_string(),
        CellValue::Int64(i) => i.to_string(),
        CellValue::Float64(f) => f.to_string(),
        CellValue::Text(s) => s.clone(),
        CellValue::Bytes(_) => "[binary]".into(),
        CellValue::Uuid(s) => s.clone(),
        CellValue::DateTime(s) => s.clone(),
        CellValue::Json(v) => v.to_string(),
    }
}

fn cell_to_json(cell: &CellValue) -> serde_json::Value {
    match cell {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Bool(b) => serde_json::Value::Bool(*b),
        CellValue::Int64(i) => serde_json::Value::Number((*i).into()),
        CellValue::Float64(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        CellValue::Text(s) => serde_json::Value::String(s.clone()),
        CellValue::Bytes(b) => serde_json::Value::String(format!("[{} bytes]", b.len())),
        CellValue::Uuid(s) => serde_json::Value::String(s.clone()),
        CellValue::DateTime(s) => serde_json::Value::String(s.clone()),
        CellValue::Json(v) => v.clone(),
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
        CellValue::Int64(i) => worksheet
            .write_number(row, col, *i as f64)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Float64(f) => worksheet
            .write_number(row, col, *f)
            .map(|_| ())
            .map_err(|e| DbError::Internal(format!("excel write failed: {e}"))),
        CellValue::Text(s) | CellValue::Uuid(s) | CellValue::DateTime(s) => worksheet
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
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::{ColumnMeta, Row};
    use crate::ports::MockDbConnector;

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

    fn build_service(connector: MockDbConnector, registry: Arc<ConnectionRegistry>) -> ExportService {
        ExportService::new(Box::new(connector), registry)
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
}
