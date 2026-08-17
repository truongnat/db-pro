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
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    args.add(dt.with_timezone(&chrono::Utc))
                } else {
                    let date = chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").map_err(|e| {
                        DbError::QueryFailed(format!("invalid date/datetime parameter: {e}"))
                    })?;
                    args.add(date)
                }
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
            decode_cell(row, i, &col.data_type)
        };
        cells.push(cell);
    }
    Ok(Row(cells))
}

fn decode_cell(row: &sqlx::postgres::PgRow, i: usize, data_type: &str) -> CellValue {
    let dt_upper = data_type.to_uppercase();
    let res = match dt_upper.as_str() {
        "BOOL" => row.try_get::<bool, _>(i).map(CellValue::Bool),
        "INT2" => row.try_get::<i16, _>(i).map(|v| CellValue::Int64(v as i64)),
        "INT4" => row.try_get::<i32, _>(i).map(|v| CellValue::Int64(v as i64)),
        "INT8" => row.try_get::<i64, _>(i).map(CellValue::Int64),
        "OID" => row
            .try_get::<i32, _>(i)
            .map(|v| CellValue::Int64(v as i64))
            .or_else(|_| row.try_get::<i64, _>(i).map(CellValue::Int64)),
        "FLOAT4" => row.try_get::<f32, _>(i).map(|v| CellValue::Float64(v as f64)),
        "FLOAT8" => row.try_get::<f64, _>(i).map(CellValue::Float64),
        "UUID" => row
            .try_get::<uuid::Uuid, _>(i)
            .map(|v| CellValue::Uuid(v.to_string())),
        "TIMESTAMPTZ" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(i)
            .map(|v| CellValue::DateTime(v.to_rfc3339())),
        "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, _>(i)
            .map(|v| CellValue::DateTime(v.and_utc().to_rfc3339())),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(i)
            .map(|v| CellValue::DateTime(v.to_string())),
        "TIME" => row
            .try_get::<chrono::NaiveTime, _>(i)
            .map(|v| CellValue::Text(v.to_string())),
        "JSON" | "JSONB" => row
            .try_get::<serde_json::Value, _>(i)
            .map(CellValue::Json),
        "BYTEA" => row.try_get::<Vec<u8>, _>(i).map(CellValue::Bytes),
        _ => row.try_get::<String, _>(i).map(CellValue::Text),
    };

    res.unwrap_or_else(|_| {
        row.try_get::<String, _>(i)
            .map(CellValue::Text)
            .unwrap_or_else(|_| CellValue::Text(format!("<unsupported value: {}>", data_type)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_params_handles_supported_types() {
        let mut args = PgArguments::default();
        let params = vec![
            QueryParam::Null,
            QueryParam::Bool(true),
            QueryParam::Int64(42),
            QueryParam::Float64(1.234),
            QueryParam::Text("hello".into()),
            QueryParam::Bytes(vec![1, 2, 3]),
            QueryParam::Uuid("550e8400-e29b-41d4-a716-446655440000".into()),
            QueryParam::DateTime("2026-01-01T00:00:00Z".into()),
            QueryParam::Json(serde_json::json!({"key": "value"})),
        ];
        assert!(bind_params(&params, &mut args).is_ok());
    }

    #[test]
    fn bind_params_accepts_date_only_value() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::DateTime("2026-08-17".into())];
        assert!(bind_params(&params, &mut args).is_ok());
    }

    #[test]
    fn bind_params_fails_on_invalid_uuid() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::Uuid("invalid-uuid".into())];
        assert!(bind_params(&params, &mut args).is_err());
    }

    #[test]
    fn bind_params_fails_on_invalid_datetime() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::DateTime("invalid-date".into())];
        assert!(bind_params(&params, &mut args).is_err());
    }
}
