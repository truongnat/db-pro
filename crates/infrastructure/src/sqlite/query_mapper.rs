use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam};
use rusqlite::types::{ToSql, ValueRef};

pub fn to_rusqlite_params(params: &[QueryParam]) -> Vec<Box<dyn ToSql>> {
    params
        .iter()
        .map(|p| -> Box<dyn ToSql> {
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
