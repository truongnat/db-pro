use serde::Deserialize;
use tauri::State;

use db_pro_core::application::UserService;
use db_pro_core::domain::connection::ConnectionId;

use crate::dto::{CommandError, DatabaseUserDto, PrivilegeDto};

fn parse_connection_id(id: &str) -> Result<ConnectionId, CommandError> {
    ConnectionId::parse(id).map_err(|e| CommandError {
        error: "VALIDATION".into(),
        message: format!("invalid connection id: {e}"),
        message_id: "error.validation".into(),
        details: None,
    })
}

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
    pub schema: String,
    pub table: String,
    pub privilege: String,
}

#[tauri::command]
pub async fn list_users(req: ConnectionIdRequest, service: State<'_, UserService>) -> Result<Vec<DatabaseUserDto>, CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    let users = service.list_users(&conn_id).await.map_err(CommandError::from)?;
    Ok(users.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn create_role(req: CreateRoleRequest, service: State<'_, UserService>) -> Result<(), CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    service.create_role(&conn_id, &req.name, req.login).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn drop_role(req: DropRoleRequest, service: State<'_, UserService>) -> Result<(), CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    service.drop_role(&conn_id, &req.name).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn list_privileges(req: RoleNameRequest, service: State<'_, UserService>) -> Result<Vec<PrivilegeDto>, CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    let privs = service.list_privileges(&conn_id, &req.role_name).await.map_err(CommandError::from)?;
    Ok(privs.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn grant_privilege(req: GrantRequest, service: State<'_, UserService>) -> Result<(), CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    service.grant_privilege(&conn_id, &req.role_name, &req.schema, &req.table, &req.privilege).await.map_err(CommandError::from)
}

#[tauri::command]
pub async fn revoke_privilege(req: GrantRequest, service: State<'_, UserService>) -> Result<(), CommandError> {
    let conn_id = parse_connection_id(&req.connection_id)?;
    service.revoke_privilege(&conn_id, &req.role_name, &req.schema, &req.table, &req.privilege).await.map_err(CommandError::from)
}
