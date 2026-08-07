use tauri::State;

use crate::cancel::CancelRegistry;
use crate::dto::{CommandError, MultiQueryResultDto, QueryHistoryDto, QueryResultDto, RunConfigDto, SavedQueryDto, SavedQueryFolderDto};
use db_pro_core::application::QueryService;
use db_pro_core::domain::connection::ConnectionId;

#[tauri::command]
pub async fn execute_query(
    service: State<'_, QueryService>,
    cancel_registry: State<'_, CancelRegistry>,
    connection_id: String,
    sql: String,
) -> Result<QueryResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let cancel_rx = cancel_registry.register(&connection_id);

    let query_future = service.execute(&conn_id, &sql, &[]);
    tokio::pin!(query_future);

    let result = tokio::select! {
        res = &mut query_future => res,
        _ = cancel_rx => {
            cancel_registry.unregister(&connection_id);
            return Err(CommandError {
                error: "QUERY_CANCELLED".into(),
                message: "Query was cancelled by user".into(),
                message_id: "error.query.cancelled".into(),
                details: None,
                retryable: false,
            });
        }
    };

    cancel_registry.unregister(&connection_id);
    let result = result?;
    Ok(result.into())
}

#[tauri::command]
pub async fn cancel_query(
    cancel_registry: State<'_, CancelRegistry>,
    connection_id: String,
) -> Result<(), CommandError> {
    cancel_registry.cancel(&connection_id);
    Ok(())
}

#[tauri::command]
pub async fn execute_query_multi(
    service: State<'_, QueryService>,
    connection_id: String,
    sql: String,
) -> Result<MultiQueryResultDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let result = service.execute_multi(&conn_id, &sql).await?;
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
        retryable: false,
    })?;
    service.delete_saved_query(&uuid).await?;
    Ok(())
}

#[tauri::command]
pub async fn create_folder(
    service: State<'_, QueryService>,
    connection_id: String,
    name: String,
) -> Result<SavedQueryFolderDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let folder = service.create_folder(&conn_id, &name).await?;
    Ok(folder.into())
}

#[tauri::command]
pub async fn list_folders(
    service: State<'_, QueryService>,
    connection_id: String,
) -> Result<Vec<SavedQueryFolderDto>, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let folders = service.list_folders(&conn_id).await?;
    Ok(folders.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn delete_folder(service: State<'_, QueryService>, id: String) -> Result<(), CommandError> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid folder id: {e}"),
        message_id: "error.validation".into(),
        details: None,
        retryable: false,
    })?;
    service.delete_folder(&uuid).await?;
    Ok(())
}

#[tauri::command]
pub async fn save_run_config(
    service: State<'_, QueryService>,
    connection_id: String,
    name: String,
    sql: String,
    timeout_ms: u64,
    max_rows: u64,
) -> Result<RunConfigDto, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let config = service
        .save_run_config(&conn_id, &name, &sql, timeout_ms, max_rows)
        .await?;
    Ok(config.into())
}

#[tauri::command]
pub async fn list_run_configs(
    service: State<'_, QueryService>,
    connection_id: String,
) -> Result<Vec<RunConfigDto>, CommandError> {
    let conn_id = parse_connection_id(&connection_id)?;
    let configs = service.list_run_configs(&conn_id).await?;
    Ok(configs.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn delete_run_config(service: State<'_, QueryService>, id: String) -> Result<(), CommandError> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid run config id: {e}"),
        message_id: "error.validation".into(),
        details: None,
        retryable: false,
    })?;
    service.delete_run_config(&uuid).await?;
    Ok(())
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
