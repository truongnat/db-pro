use tauri::State;

use crate::dto::{CommandError, FetchRowsRequest, FetchRowsResultDto, MutateRowRequest, MutateRowResultDto};
use db_pro_core::application::TableDataService;
use db_pro_core::domain::connection::ConnectionId;

#[tauri::command]
pub async fn fetch_table_rows(
    service: State<'_, TableDataService>,
    connection_id: String,
    request: FetchRowsRequest,
) -> Result<FetchRowsResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let filters = request
        .filters
        .iter()
        .map(|f| f.to_domain())
        .collect::<Result<Vec<_>, _>>()?;
    let sorts: Vec<_> = request.sorts.iter().map(|s| s.to_domain()).collect();
    let offset = request.page.saturating_sub(1) * request.page_size;

    let (result, total_count) = service
        .fetch_rows(
            &conn_id,
            &request.schema,
            &request.table,
            &filters,
            &sorts,
            request.page_size,
            offset,
        )
        .await?;

    Ok(FetchRowsResultDto {
        columns: result.columns.into_iter().map(Into::into).collect(),
        rows: result.rows.into_iter().map(Into::into).collect(),
        total_count,
        duration_ms: result.duration_ms,
    })
}

#[tauri::command]
pub async fn insert_table_row(
    service: State<'_, TableDataService>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let values: Vec<_> = request.values.into_iter().map(Into::into).collect();
    let affected = service
        .insert_row(&conn_id, &request.schema, &request.table, &request.columns, &values)
        .await?;
    Ok(MutateRowResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn update_table_row(
    service: State<'_, TableDataService>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let values: Vec<_> = request.values.into_iter().map(Into::into).collect();
    let pk_columns = request.pk_columns.unwrap_or_default();
    let pk_values: Vec<_> = request
        .pk_values
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect();

    if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
        return Err(CommandError {
            error: "VALIDATION".into(),
            message: "pk_columns and pk_values are required for update".into(),
            message_id: "error.validation".into(),
            details: None,
            retryable: false,
        });
    }

    let affected = service
        .update_row(
            &conn_id,
            &request.schema,
            &request.table,
            &request.columns,
            &values,
            &pk_columns,
            &pk_values,
        )
        .await?;
    Ok(MutateRowResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn delete_table_row(
    service: State<'_, TableDataService>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let pk_columns = request.pk_columns.unwrap_or_default();
    let pk_values: Vec<_> = request
        .pk_values
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect();

    if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
        return Err(CommandError {
            error: "VALIDATION".into(),
            message: "pk_columns and pk_values are required for delete".into(),
            message_id: "error.validation".into(),
            details: None,
            retryable: false,
        });
    }

    let affected = service
        .delete_row(&conn_id, &request.schema, &request.table, &pk_columns, &pk_values)
        .await?;
    Ok(MutateRowResultDto {
        affected_rows: affected,
    })
}

fn parse_connection_id(id: &str) -> Result<ConnectionId, CommandError> {
    ConnectionId::parse(id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
        retryable: false,
    })
}
