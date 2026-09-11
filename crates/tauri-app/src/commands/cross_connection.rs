use std::sync::Arc;
use tauri::State;

use crate::dto::{CommandError, DataDiffDto, ObjectDependencyDto, PartitionInfoDto, SchemaDiffDto, TablespaceInfoDto};
use db_pro_runtime::DbProRuntime;

#[tauri::command]
pub async fn diff_schemas(
    runtime: State<'_, Arc<DbProRuntime>>,
    source_id: String,
    target_id: String,
) -> Result<SchemaDiffDto, CommandError> {
    let diff = runtime.schema_api().diff_schemas(&source_id, &target_id).await?;
    Ok(diff.into())
}

#[tauri::command]
pub async fn diff_table_data(
    runtime: State<'_, Arc<DbProRuntime>>,
    source_id: String,
    target_id: String,
    schema: String,
    table: String,
) -> Result<DataDiffDto, CommandError> {
    let diff = runtime
        .data_diff_api()
        .diff_table_data(&source_id, &target_id, &schema, &table)
        .await?;
    Ok(diff.into())
}

#[tauri::command]
pub async fn get_object_dependencies(
    runtime: State<'_, Arc<DbProRuntime>>,
    connection_id: String,
    schema: String,
    object_name: String,
) -> Result<Vec<ObjectDependencyDto>, CommandError> {
    let deps = runtime
        .postgres_api()
        .object_dependencies(&connection_id, &schema, &object_name)
        .await?;
    Ok(deps.into_iter().map(ObjectDependencyDto::from).collect())
}

#[tauri::command]
pub async fn list_partitions(
    runtime: State<'_, Arc<DbProRuntime>>,
    connection_id: String,
) -> Result<Vec<PartitionInfoDto>, CommandError> {
    let partitions = runtime.postgres_api().partitions(&connection_id).await?;
    Ok(partitions.into_iter().map(PartitionInfoDto::from).collect())
}

#[tauri::command]
pub async fn list_tablespaces(
    runtime: State<'_, Arc<DbProRuntime>>,
    connection_id: String,
) -> Result<Vec<TablespaceInfoDto>, CommandError> {
    let tablespaces = runtime.postgres_api().tablespaces(&connection_id).await?;
    Ok(tablespaces.into_iter().map(TablespaceInfoDto::from).collect())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn rename_schema_object(
    runtime: State<'_, Arc<DbProRuntime>>,
    connection_id: String,
    object_type: String,
    schema: String,
    old_name: String,
    new_name: String,
) -> Result<(), CommandError> {
    runtime
        .postgres_api()
        .rename_schema_object(&connection_id, &object_type, &schema, &old_name, &new_name)
        .await?;
    Ok(())
}
