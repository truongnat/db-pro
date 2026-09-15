use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
use sqlx::{Column, Row as SqlxRow, TypeInfo};

pub struct MySqlQueryMapper;

impl MySqlQueryMapper {
    pub fn map_rows(rows: Vec<sqlx::mysql::MySqlRow>) -> Result<QueryResult, DbError> {
        let mut result_rows = Vec::new();
        let mut columns: Vec<ColumnMeta> = Vec::new();

        for row in &rows {
            let cols = row.columns();
            if columns.is_empty() {
                columns = cols
                    .iter()
                    .map(|c| ColumnMeta {
                        name: c.name().to_string(),
                        data_type: c.type_info().name().to_string(),
                        nullable: true,
                    })
                    .collect();
            }
            let mut cells = Vec::new();
            for (i, _col) in cols.iter().enumerate() {
                let value = Self::extract_cell(row, i)?;
                cells.push(value);
            }
            result_rows.push(Row(cells));
        }

        let row_count = result_rows.len() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
            duration_ms: 0,
        })
    }

    fn extract_cell(row: &sqlx::mysql::MySqlRow, index: usize) -> Result<CellValue, DbError> {
        if let Ok(Some(val)) = row.try_get::<Option<bool>, _>(index) {
            return Ok(CellValue::Bool(val));
        }
        if let Ok(Some(val)) = row.try_get::<Option<i64>, _>(index) {
            return Ok(CellValue::Int64(val));
        }
        if let Ok(Some(val)) = row.try_get::<Option<f64>, _>(index) {
            return Ok(CellValue::Float64(val));
        }
        if let Ok(Some(val)) = row.try_get::<Option<String>, _>(index) {
            return Ok(CellValue::Text(val));
        }
        if let Ok(Some(val)) = row.try_get::<Option<Vec<u8>>, _>(index) {
            return Ok(CellValue::Bytes(val));
        }
        if let Ok(None) = row.try_get::<Option<String>, _>(index) {
            return Ok(CellValue::Null);
        }
        Ok(CellValue::Text("unsupported".to_string()))
    }
}
