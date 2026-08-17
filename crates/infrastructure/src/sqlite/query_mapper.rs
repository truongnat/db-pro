use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam};
use rusqlite::types::{ToSql, ValueRef};

pub fn to_rusqlite_params(params: &[QueryParam]) -> Result<Vec<Box<dyn ToSql>>, DbError> {
    params
        .iter()
        .map(|p| -> Result<Box<dyn ToSql>, DbError> {
            match p {
                QueryParam::Null => Ok(Box::new(rusqlite::types::Null)),
                QueryParam::Bool(v) => Ok(Box::new(*v)),
                QueryParam::Int64(v) => Ok(Box::new(*v)),
                QueryParam::Float64(v) => Ok(Box::new(*v)),
                QueryParam::Decimal(_) => Err(DbError::Unsupported(
                    "SQLite decimal parameter binding is not enabled yet".into(),
                )),
                QueryParam::Text(v) => Ok(Box::new(v.clone())),
                QueryParam::Bytes(v) => Ok(Box::new(v.clone())),
                QueryParam::Uuid(v) => Ok(Box::new(v.clone())),
                QueryParam::DateTime(v) => Ok(Box::new(v.clone())),
                QueryParam::Json(v) => Ok(Box::new(v.to_string())),
            }
        })
        .collect()
}

pub fn map_row_to_cells(row: &rusqlite::Row) -> Result<Vec<CellValue>, DbError> {
    let mut cells = Vec::new();
    for i in 0..row.as_ref().column_count() {
        let value = row.get_ref(i).map_err(crate::error::from_rusqlite)?;
        let cell = match value {
            ValueRef::Null => CellValue::Null,
            ValueRef::Integer(v) => CellValue::Int64(v),
            ValueRef::Real(v) => CellValue::Float64(v),
            ValueRef::Text(v) => CellValue::Text(String::from_utf8_lossy(v).into_owned()),
            ValueRef::Blob(v) => CellValue::Bytes(v.to_vec()),
        };
        cells.push(cell);
    }
    Ok(cells)
}

pub fn extract_columns(stmt: &rusqlite::Statement) -> Vec<ColumnMeta> {
    let columns = stmt.columns();
    (0..stmt.column_count())
        .map(|i| ColumnMeta {
            name: columns[i].name().to_string(),
            data_type: columns[i].decl_type().unwrap_or("TEXT").to_string(),
            nullable: true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_parameter_is_rejected_before_sqlite_binding() {
        let params = vec![QueryParam::Decimal("1234567890.1234500".into())];
        assert!(matches!(to_rusqlite_params(&params), Err(DbError::Unsupported(_))));
    }
}
