use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam, Row};
use sqlx::postgres::PgArguments;
use sqlx::{Arguments, Column, Row as _, TypeInfo, ValueRef};

pub fn bind_params(params: &[QueryParam], args: &mut PgArguments) -> Result<(), DbError> {
    for param in params {
        let result = match param {
            QueryParam::Null => args.add(Option::<String>::None),
            QueryParam::Bool(v) => args.add(v),
            QueryParam::Int64(v) => args.add(v),
            QueryParam::Float64(v) => args.add(v),
            QueryParam::Text(v) => args.add(v.as_str()),
            QueryParam::Bytes(v) => args.add(v.as_slice()),
            QueryParam::Uuid(v) => {
                let uuid = uuid::Uuid::parse_str(v)
                    .map_err(|e| DbError::QueryFailed(format!("invalid UUID parameter: {e}")))?;
                args.add(uuid)
            }
            QueryParam::DateTime(v) => {
                let dt = chrono::DateTime::parse_from_rfc3339(v)
                    .map_err(|e| DbError::QueryFailed(format!("invalid datetime parameter: {e}")))?;
                args.add(dt.with_timezone(&chrono::Utc))
            }
            QueryParam::Json(v) => args.add(sqlx::types::Json(v)),
        };
        result.map_err(|e| DbError::QueryFailed(format!("failed to bind parameter: {e}")))?;
    }
    Ok(())
}

pub fn columns_from_describe(describe: &sqlx::Describe<sqlx::Postgres>) -> Vec<ColumnMeta> {
    describe
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| ColumnMeta {
            name: col.name().to_string(),
            data_type: col.type_info().name().to_string(),
            nullable: describe.nullable(i).unwrap_or(true),
        })
        .collect()
}

pub fn map_row(row: &sqlx::postgres::PgRow, columns: &[ColumnMeta]) -> Result<Row, DbError> {
    let mut cells = Vec::with_capacity(columns.len());
    for (i, col) in columns.iter().enumerate() {
        let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;
        let cell = if raw.is_null() {
            CellValue::Null
        } else {
            match col.data_type.as_str() {
                "BOOL" => CellValue::Bool(row.try_get(i).map_err(crate::error::from_sqlx)?),
                "INT2" | "INT4" | "INT8" => CellValue::Int64(row.try_get(i).map_err(crate::error::from_sqlx)?),
                "FLOAT4" | "FLOAT8" => CellValue::Float64(row.try_get(i).map_err(crate::error::from_sqlx)?),
                "UUID" => {
                    let v: uuid::Uuid = row.try_get(i).map_err(crate::error::from_sqlx)?;
                    CellValue::Uuid(v.to_string())
                }
                "TIMESTAMPTZ" => {
                    let v: chrono::DateTime<chrono::Utc> = row.try_get(i).map_err(crate::error::from_sqlx)?;
                    CellValue::DateTime(v.to_rfc3339())
                }
                "TIMESTAMP" => {
                    let v: chrono::NaiveDateTime = row.try_get(i).map_err(crate::error::from_sqlx)?;
                    CellValue::DateTime(v.and_utc().to_rfc3339())
                }
                "JSON" | "JSONB" => {
                    let v: serde_json::Value = row.try_get(i).map_err(crate::error::from_sqlx)?;
                    CellValue::Json(v)
                }
                "BYTEA" => {
                    let v: Vec<u8> = row.try_get(i).map_err(crate::error::from_sqlx)?;
                    CellValue::Bytes(v)
                }
                _ => {
                    let v: String = row.try_get(i).unwrap_or_default();
                    CellValue::Text(v)
                }
            }
        };
        cells.push(cell);
    }
    Ok(Row(cells))
}
