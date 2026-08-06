use tauri::State;

use crate::dto::{CommandError, ExportResultDto};
use db_pro_core::application::ExportService;

#[tauri::command]
pub async fn export_csv(
    service: State<'_, ExportService>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.export_csv(&conn_id, &sql).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn export_json(
    service: State<'_, ExportService>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.export_json(&conn_id, &sql).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn export_excel(
    service: State<'_, ExportService>,
    connection_id: String,
    sql: String,
) -> Result<ExportResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.export_excel(&conn_id, &sql).await?;
    Ok(result.into())
}

fn parse_connection_id(id: &str) -> Result<db_pro_core::domain::connection::ConnectionId, CommandError> {
    db_pro_core::domain::connection::ConnectionId::parse(id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })
}
