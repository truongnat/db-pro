use tauri::State;

use crate::dto::{CommandError, DdlResultDto, IntrospectResultDto, TableInfoDto};
use db_pro_runtime::DbProRuntime;

#[tauri::command]
pub async fn introspect(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    force_refresh: Option<bool>,
) -> Result<IntrospectResultDto, CommandError> {
    let result = runtime
        .schema_api()
        .introspect(&connection_id, force_refresh.unwrap_or(false))
        .await?;
    Ok(result.into())
}

#[tauri::command]
pub async fn get_table_info(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    schema: String,
    table: String,
) -> Result<TableInfoDto, CommandError> {
    let info = runtime.schema_api().table_info(&connection_id, &schema, &table).await?;
    Ok(info.into())
}

#[tauri::command]
pub async fn get_table_ddl(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    schema: String,
    table: String,
) -> Result<String, CommandError> {
    Ok(runtime.schema_api().table_ddl(&connection_id, &schema, &table).await?)
}

#[tauri::command]
pub async fn execute_ddl(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime.schema_api().execute_ddl(&connection_id, &sql).await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn execute_ddl_batch(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    statements: Vec<String>,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime
        .schema_api()
        .execute_ddl_batch(&connection_id, &statements)
        .await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn create_index(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime.schema_api().execute_ddl(&connection_id, &sql).await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn drop_index(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime.schema_api().execute_ddl(&connection_id, &sql).await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn create_trigger(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime.schema_api().execute_ddl(&connection_id, &sql).await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn drop_trigger(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
    sql: String,
) -> Result<DdlResultDto, CommandError> {
    let affected = runtime.schema_api().execute_ddl(&connection_id, &sql).await?;
    Ok(DdlResultDto {
        affected_rows: affected,
    })
}

#[tauri::command]
pub async fn invalidate_cache(
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
    connection_id: String,
) -> Result<(), CommandError> {
    runtime.schema_api().invalidate_cache(&connection_id).await?;
    Ok(())
}
