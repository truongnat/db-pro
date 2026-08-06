use tauri::State;

use crate::dto::{CommandError, ConnectionConfigDto, ConnectionDto};
use db_pro_core::application::ConnectionService;
use db_pro_core::domain::connection::ConnectionId;

#[tauri::command]
pub async fn list_connections(service: State<'_, ConnectionService>) -> Result<Vec<ConnectionDto>, CommandError> {
    let connections = service.list().await?;
    Ok(connections.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn get_connection(
    service: State<'_, ConnectionService>,
    id: String,
) -> Result<Option<ConnectionDto>, CommandError> {
    let conn_id = ConnectionId::parse(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    let connection = service.get(&conn_id).await?;
    Ok(connection.map(Into::into))
}

#[tauri::command]
pub async fn create_connection(
    service: State<'_, ConnectionService>,
    config: ConnectionConfigDto,
    password: String,
) -> Result<ConnectionDto, CommandError> {
    let domain_config = config.to_domain();
    let connection = service.create(domain_config, &password).await?;
    Ok(connection.into())
}

#[tauri::command]
pub async fn update_connection(
    service: State<'_, ConnectionService>,
    id: String,
    config: ConnectionConfigDto,
    password: Option<String>,
) -> Result<(), CommandError> {
    let conn_id = ConnectionId::parse(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    let domain_config = config.to_domain();
    service.update(&conn_id, domain_config, password.as_deref()).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_connection(service: State<'_, ConnectionService>, id: String) -> Result<(), CommandError> {
    let conn_id = ConnectionId::parse(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    service.delete(&conn_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    service: State<'_, ConnectionService>,
    config: ConnectionConfigDto,
    password: String,
) -> Result<(), CommandError> {
    let domain_config = config.to_domain();
    service.test_connectivity(&domain_config, &password).await?;
    Ok(())
}

#[tauri::command]
pub async fn connect(service: State<'_, ConnectionService>, id: String) -> Result<(), CommandError> {
    let conn_id = ConnectionId::parse(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    service.connect(&conn_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn disconnect(service: State<'_, ConnectionService>, id: String) -> Result<(), CommandError> {
    let conn_id = ConnectionId::parse(&id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })?;
    service.disconnect(&conn_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn test_ssh_tunnel(
    connector: tauri::State<'_, db_pro_infrastructure::connector::CompositeConnector>,
    config: crate::dto::SshTunnelConfigDto,
) -> Result<(), CommandError> {
    let domain_config = db_pro_core::domain::connection::SshTunnelConfig {
        host: config.host,
        port: config.port,
        user: config.user,
        private_key_path: config.private_key_path,
        password: config.password,
    };
    connector.test_ssh_tunnel(&domain_config).await?;
    Ok(())
}
