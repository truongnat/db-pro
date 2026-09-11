use tauri::State;

use crate::dto::{CommandError, ExportResultDto};
use db_pro_runtime::DbProRuntime;

#[tauri::command]
pub async fn export_csv(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let result = runtime.export_api().csv(&connection_id, &sql).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn export_json(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let result = runtime.export_api().json(&connection_id, &sql).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn export_excel(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let result = runtime.export_api().excel(&connection_id, &sql).await?;
    Ok(result.into())
}
