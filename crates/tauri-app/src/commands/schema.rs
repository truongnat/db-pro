use tauri::State;

use crate::dto::{CommandError, IntrospectResultDto, TableInfoDto};
use db_pro_core::application::SchemaService;
use db_pro_core::domain::connection::ConnectionId;

#[tauri::command]
pub async fn introspect(
    service: State<'_, SchemaService>,
    connection_id: String,
    force_refresh: Option<bool>,
) -> Result<IntrospectResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.introspect(&conn_id, force_refresh.unwrap_or(false)).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn get_table_info(
    service: State<'_, SchemaService>,
    connection_id: String,
    schema: String,
    table: String,
) -> Result<TableInfoDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let info = service.get_table_info(&conn_id, &schema, &table).await?;
    Ok(info.into())
}

#[tauri::command]
pub async fn get_table_ddl(
    service: State<'_, SchemaService>,
    connection_id: String,
    schema: String,
    table: String,
) -> Result<String, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    Ok(service.get_table_ddl(&conn_id, &schema, &table).await?)
}

#[tauri::command]
pub async fn invalidate_cache(service: State<'_, SchemaService>, connection_id: String) -> Result<(), CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    service.invalidate_cache(&conn_id).await?;
    Ok(())
}

fn parse_connection_id(id: &str) -> Result<ConnectionId, CommandError> {
    ConnectionId::parse(id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })
}
