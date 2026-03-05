mod connection;
mod navigator;
mod query;
mod shared;

use tauri::State;

use crate::models::{
    ConnectionInput, ConnectionListItem, NavigatorTree, QueryRequest, QueryResult,
};
use crate::state::AppState;

#[tauri::command]
pub fn list_connections(state: State<'_, AppState>) -> Vec<ConnectionListItem> {
    connection::list_connections(state.inner())
}

#[tauri::command]
pub fn get_connection(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<ConnectionInput, String> {
    connection::get_connection(&connection_id, state.inner())
}

#[tauri::command]
pub fn upsert_connection(
    input: ConnectionInput,
    state: State<'_, AppState>,
) -> Result<ConnectionListItem, String> {
    connection::upsert_connection(input, state.inner())
}

#[tauri::command]
pub fn delete_connection(connection_id: String, state: State<'_, AppState>) -> Result<(), String> {
    connection::delete_connection(&connection_id, state.inner())
}

#[tauri::command]
pub async fn reset_connections(
    state: State<'_, AppState>,
) -> Result<Vec<ConnectionListItem>, String> {
    connection::reset_connections(state.inner()).await
}

#[tauri::command]
pub async fn test_connection(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    query::test_connection(&connection_id, state.inner()).await
}

#[tauri::command]
pub async fn execute_query(
    request: QueryRequest,
    state: State<'_, AppState>,
) -> Result<QueryResult, String> {
    query::execute_query(request, state.inner()).await
}

#[tauri::command]
pub fn cancel_query(connection_id: String, state: State<'_, AppState>) -> Result<String, String> {
    query::cancel_query(&connection_id, state.inner())
}

#[tauri::command]
pub async fn load_navigator(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<NavigatorTree, String> {
    navigator::load_navigator(&connection_id, state.inner()).await
}
