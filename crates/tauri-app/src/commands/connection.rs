use tauri::State;

use crate::dto::{CommandError, ConnectionConfigDto, ConnectionDto};
use db_pro_runtime::DbProRuntime;

#[tauri::command]
pub async fn list_connections(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<Vec<ConnectionDto>, CommandError> {
    let connections = runtime.connection_api().list_details().await?;
    Ok(connections.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn get_connection(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    id: String,
) -> Result<Option<ConnectionDto>, CommandError> {
    let connection = runtime.connection_api().get(&id).await?;
    Ok(connection.map(Into::into))
}

#[tauri::command]
pub async fn create_connection(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    config: ConnectionConfigDto,
    password: String,
) -> Result<ConnectionDto, CommandError> {
    let connection = runtime
        .connection_api()
        .create_detail(config.to_domain(), &password)
        .await?;
    Ok(connection.into())
}

#[tauri::command]
pub async fn update_connection(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    id: String,
    config: ConnectionConfigDto,
    password: Option<String>,
) -> Result<(), CommandError> {
    runtime
        .connection_api()
        .update(&id, config.to_domain(), password.as_deref())
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_connection(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    id: String,
) -> Result<(), CommandError> {
    runtime.connection_api().delete(&id).await?;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    config: ConnectionConfigDto,
    password: String,
    connection_id: Option<String>,
) -> Result<(), CommandError> {
    let domain_config = config.to_domain();
    match connection_id {
        Some(id) => {
            runtime
                .connection_api()
                .test_with_secret(&id, &domain_config, &password)
                .await?;
        }
        None => runtime.connection_api().test(&domain_config, &password).await?,
    }
    Ok(())
}

#[tauri::command]
pub async fn connect(runtime: State<'_, std::sync::Arc<DbProRuntime>>, id: String) -> Result<(), CommandError> {
    runtime.connection_api().connect(&id).await?;
    Ok(())
}

#[tauri::command]
pub async fn disconnect(runtime: State<'_, std::sync::Arc<DbProRuntime>>, id: String) -> Result<(), CommandError> {
    runtime.connection_api().disconnect(&id).await?;
    Ok(())
}

#[tauri::command]
pub async fn test_ssh_tunnel(
    runtime: tauri::State<'_, std::sync::Arc<db_pro_runtime::DbProRuntime>>,
    config: crate::dto::SshTunnelConfigDto,
) -> Result<(), CommandError> {
    let domain_config = db_pro_core::domain::connection::SshTunnelConfig {
        host: config.host,
        port: config.port,
        user: config.user,
        private_key_path: config.private_key_path,
        password: config.password,
    };
    runtime.postgres_api().test_ssh_tunnel(&domain_config).await?;
    Ok(())
}
