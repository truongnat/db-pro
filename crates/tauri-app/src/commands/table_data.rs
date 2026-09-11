use tauri::State;

use crate::dto::{CommandError, FetchRowsRequest, FetchRowsResultDto, MutateRowRequest, MutateRowResultDto};
use db_pro_runtime::DbProRuntime;

#[tauri::command]
pub async fn fetch_table_rows(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    request: FetchRowsRequest,
) -> Result<FetchRowsResultDto, CommandError> {
    let filters = request
        .filters
        .iter()
        .map(|f| f.to_domain())
        .collect::<Result<Vec<_>, _>>()?;
    let sorts: Vec<_> = request.sorts.iter().map(|s| s.to_domain()).collect();
    let offset = request.page.saturating_sub(1) * request.page_size;

    let (result, total_count) = runtime
        .table_data_api()
        .fetch_rows(
            &connection_id,
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
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
    let values: Vec<_> = request.values.into_iter().map(Into::into).collect();
    let affected = runtime
        .table_data_api()
        .insert_row(
            &connection_id,
            &request.schema,
            &request.table,
            &request.columns,
            &values,
        )
        .await?;
    Ok(MutateRowResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn update_table_row(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
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

    let affected = runtime
        .table_data_api()
        .update_row(
            &connection_id,
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
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    request: MutateRowRequest,
) -> Result<MutateRowResultDto, CommandError> {
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

    let affected = runtime
        .table_data_api()
        .delete_row(&connection_id, &request.schema, &request.table, &pk_columns, &pk_values)
        .await?;
    Ok(MutateRowResultDto {
        affected_rows: affected,
    })
}
