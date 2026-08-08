use std::sync::Arc;
use tauri::State;

use db_pro_core::application::{ConnectionRegistry, DataDiffService, SchemaService};
use db_pro_core::domain::connection::ConnectionId;
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;
use db_pro_core::ports::ConnectionRepository;

use crate::dto::{
    CommandError, DataDiffDto, ObjectDependencyDto, PartitionInfoDto, SchemaDiffDto, TablespaceInfoDto,
};

#[tauri::command]
pub async fn diff_schemas(
    service: State<'_, SchemaService>,
    source_id: String,
    target_id: String,
) -> Result<SchemaDiffDto, CommandError> {
    let src = parse_connection_id(&source_id)?;
    let tgt = parse_connection_id(&target_id)?;
    let diff = service.diff_schemas(&src, &tgt).await?;
    Ok(diff.into())
}

#[tauri::command]
pub async fn diff_table_data(
    service: State<'_, DataDiffService>,
    source_id: String,
    target_id: String,
    schema: String,
    table: String,
) -> Result<DataDiffDto, CommandError> {
    let src = parse_connection_id(&source_id)?;
    let tgt = parse_connection_id(&target_id)?;
    let diff = service.diff_table_data(&src, &tgt, &schema, &table).await?;
    Ok(diff.into())
}

#[tauri::command]
pub async fn get_object_dependencies(
    connector: State<'_, Arc<CompositeConnector>>,
    registry: State<'_, Arc<ConnectionRegistry>>,
    connection_id: String,
    schema: String,
    object_name: String,
) -> Result<Vec<ObjectDependencyDto>, CommandError> {
    let pg = connector.postgres_connector();
    let inner = resolve_postgres_handle(&connector, &registry, &connection_id)?;
    let deps = pg.get_object_dependencies(&inner, &schema, &object_name).await.map_err(CommandError::from)?;
    Ok(deps.into_iter().map(ObjectDependencyDto::from).collect())
}

#[tauri::command]
pub async fn list_partitions(
    connector: State<'_, Arc<CompositeConnector>>,
    registry: State<'_, Arc<ConnectionRegistry>>,
    connection_id: String,
) -> Result<Vec<PartitionInfoDto>, CommandError> {
    let pg = connector.postgres_connector();
    let inner = resolve_postgres_handle(&connector, &registry, &connection_id)?;
    let partitions = pg.list_partitions(&inner).await.map_err(CommandError::from)?;
    Ok(partitions.into_iter().map(PartitionInfoDto::from).collect())
}

#[tauri::command]
pub async fn list_tablespaces(
    connector: State<'_, Arc<CompositeConnector>>,
    registry: State<'_, Arc<ConnectionRegistry>>,
    connection_id: String,
) -> Result<Vec<TablespaceInfoDto>, CommandError> {
    let pg = connector.postgres_connector();
    let inner = resolve_postgres_handle(&connector, &registry, &connection_id)?;
    let tablespaces = pg.list_tablespaces(&inner).await.map_err(CommandError::from)?;
    Ok(tablespaces.into_iter().map(TablespaceInfoDto::from).collect())
}

#[tauri::command]
pub async fn rename_schema_object(
    connector: State<'_, Arc<CompositeConnector>>,
    registry: State<'_, Arc<ConnectionRegistry>>,
    meta_store: State<'_, SQLiteMetaStore>,
    connection_id: String,
    object_type: String,
    schema: String,
    old_name: String,
    new_name: String,
) -> Result<(), CommandError> {
    // Enforce readonly policy before mutating schema.
    let conn_id = parse_connection_id(&connection_id)?;
    if let Some(config) = meta_store.get_config(&conn_id).await? {
        if config.readonly {
            return Err(CommandError {
                error: "SAFETY".into(),
                message: "connection is read-only — schema rename is not allowed".into(),
                message_id: "error.safety.readonly".into(),
                details: None,
                retryable: false,
            });
        }
    }

    let pg = connector.postgres_connector();
    let inner = resolve_postgres_handle(&connector, &registry, &connection_id)?;
    pg.rename_schema_object(&inner, &object_type, &schema, &old_name, &new_name)
        .await
        .map_err(CommandError::from)?;
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

fn resolve_postgres_handle(
    connector: &CompositeConnector,
    registry: &ConnectionRegistry,
    connection_id: &str,
) -> Result<db_pro_core::domain::connection::ConnectionHandle, CommandError> {
    let conn_id = parse_connection_id(connection_id)?;
    let composite_handle = registry.get(&conn_id).ok_or_else(|| CommandError {
        error: "NOT_CONNECTED".into(),
        message: "connection is not active".into(),
        message_id: "error.connection.failed".into(),
        details: None,
        retryable: false,
    })?;
    connector
        .inner_postgres_handle(&composite_handle)
        .map_err(CommandError::from)
}
