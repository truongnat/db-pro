use serde::Deserialize;

use tauri::State;

use db_pro_core::domain::user::PrivilegeObjectKind;
use db_pro_runtime::DbProRuntime;

use crate::dto::{CommandError, DatabaseUserDto, PrivilegeDto};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionIdRequest {
    pub connection_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub connection_id: String,
    pub name: String,
    pub login: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropRoleRequest {
    pub connection_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleNameRequest {
    pub connection_id: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantRequest {
    pub connection_id: String,
    pub role_name: String,
    #[serde(default)]
    pub object_kind: Option<String>,
    pub schema: String,
    #[serde(alias = "table")]
    pub object_name: String,
    pub privilege: String,
}

fn parse_object_kind(raw: Option<&str>) -> PrivilegeObjectKind {
    match raw.map(str::to_ascii_lowercase).as_deref() {
        Some("schema") => PrivilegeObjectKind::Schema,
        Some("database") => PrivilegeObjectKind::Database,
        Some("sequence") => PrivilegeObjectKind::Sequence,
        _ => PrivilegeObjectKind::Table,
    }
}

#[tauri::command]
pub async fn list_users(
    req: ConnectionIdRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<Vec<DatabaseUserDto>, CommandError> {
    let users = runtime.user_api().list_users(&req.connection_id).await?;
    Ok(users.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn create_role(
    req: CreateRoleRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<(), CommandError> {
    runtime
        .user_api()
        .create_role(&req.connection_id, &req.name, req.login)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn drop_role(
    req: DropRoleRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<(), CommandError> {
    runtime.user_api().drop_role(&req.connection_id, &req.name).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_privileges(
    req: RoleNameRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<Vec<PrivilegeDto>, CommandError> {
    let privs = runtime
        .user_api()
        .list_privileges(&req.connection_id, &req.role_name)
        .await?;
    Ok(privs.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn grant_privilege(
    req: GrantRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<(), CommandError> {
    runtime
        .user_api()
        .grant_privilege(
            &req.connection_id,
            &req.role_name,
            parse_object_kind(req.object_kind.as_deref()),
            &req.schema,
            &req.object_name,
            &req.privilege,
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn revoke_privilege(
    req: GrantRequest,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<(), CommandError> {
    runtime
        .user_api()
        .revoke_privilege(
            &req.connection_id,
            &req.role_name,
            parse_object_kind(req.object_kind.as_deref()),
            &req.schema,
            &req.object_name,
            &req.privilege,
        )
        .await?;
    Ok(())
}
