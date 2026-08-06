use tauri::State;

use crate::dto::{CommandError, QueryHistoryDto, QueryResultDto, SavedQueryDto};
use db_pro_core::application::QueryService;
use db_pro_core::domain::connection::ConnectionId;

#[tauri::command]
pub async fn execute_query(
    service: State<'_, QueryService>,
    connection_id: String,
    sql: String,
) -> Result<QueryResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.execute(&conn_id, &sql, &[]).await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn explain_query(
    service: State<'_, QueryService>,
    connection_id: String,
    sql: String,
) -> Result<serde_json::Value, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    Ok(service.explain(&conn_id, &sql).await?)
}

#[tauri::command]
pub async fn get_query_history(
    service: State<'_, QueryService>,
    connection_id: String,
    limit: Option<u32>,
) -> Result<Vec<QueryHistoryDto>, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let history = service.get_history(&conn_id, limit.unwrap_or(100)).await?;
    Ok(history.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn save_query(
    service: State<'_, QueryService>,
    connection_id: String,
    name: String,
    sql: String,
    folder: Option<String>,
) -> Result<SavedQueryDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let saved = service.save_query(&conn_id, &name, &sql, folder.as_deref()).await?;
    Ok(saved.into())
}

#[tauri::command]
pub async fn list_saved_queries(
    service: State<'_, QueryService>,
    connection_id: String,
) -> Result<Vec<SavedQueryDto>, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let queries = service.list_saved_queries(&conn_id).await?;
    Ok(queries.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn delete_saved_query(service: State<'_, QueryService>, id: String) -> Result<(), CommandError> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid query id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    service.delete_saved_query(&uuid).await?;
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
