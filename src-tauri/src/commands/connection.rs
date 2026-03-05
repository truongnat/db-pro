use std::io::ErrorKind;
use std::path::Path;

use uuid::Uuid;

use crate::models::{ConnectionInput, ConnectionListItem, ConnectionProfile, DbEngine};
use crate::sample;
use crate::secrets;
use crate::state::AppState;

use super::shared::{normalize_sqlite_profile_path, persist_profiles, required};

pub fn list_connections(state: &AppState) -> Vec<ConnectionListItem> {
    let guard = match state.connections.read() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    let mut items: Vec<ConnectionListItem> = guard
        .values()
        .map(ConnectionListItem::from_profile)
        .collect();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    items
}

pub fn get_connection(connection_id: &str, state: &AppState) -> Result<ConnectionInput, String> {
    let guard = state
        .connections
        .read()
        .map_err(|_| "Failed to access app state".to_string())?;

    let profile = guard
        .get(connection_id)
        .cloned()
        .ok_or_else(|| "Connection not found".to_string())?;

    Ok(to_connection_input(&profile))
}

pub fn upsert_connection(
    input: ConnectionInput,
    state: &AppState,
) -> Result<ConnectionListItem, String> {
    let existing_profile = if let Some(id) = input.id.as_deref() {
        let guard = state
            .connections
            .read()
            .map_err(|_| "Failed to access app state".to_string())?;
        guard.get(id).cloned()
    } else {
        None
    };

    let mut profile = validate_and_build_connection(input, existing_profile.as_ref())?;

    if matches!(profile.engine, DbEngine::Sqlite) {
        let base_dir = state
            .storage_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        normalize_sqlite_profile_path(&mut profile, base_dir);
    }

    if matches!(profile.engine, DbEngine::Postgres | DbEngine::Mysql) {
        let password = required(profile.password.as_deref(), "Password")?;
        secrets::save_password(&profile.id, password)?;
        profile.password = None;
    }

    let item = ConnectionListItem::from_profile(&profile);
    let mut guard = state
        .connections
        .write()
        .map_err(|_| "Failed to update app state".to_string())?;
    guard.insert(profile.id.clone(), profile);
    let snapshot = guard.clone();
    drop(guard);

    persist_profiles(state, &snapshot)?;

    Ok(item)
}

pub fn delete_connection(connection_id: &str, state: &AppState) -> Result<(), String> {
    if connection_id == sample::SAMPLE_CONNECTION_ID {
        return Err(
            "Sample SQLite connection is protected. Use Reset Data to restore defaults."
                .to_string(),
        );
    }

    let mut guard = state
        .connections
        .write()
        .map_err(|_| "Failed to update app state".to_string())?;

    if guard.remove(connection_id).is_some() {
        let snapshot = guard.clone();
        drop(guard);

        secrets::delete_password(connection_id)?;
        persist_profiles(state, &snapshot)?;

        return Ok(());
    }

    Err("Connection not found".to_string())
}

pub async fn reset_connections(state: &AppState) -> Result<Vec<ConnectionListItem>, String> {
    let app_data_dir = state
        .storage_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let sample_sqlite_path = app_data_dir.join("db-pro-sample.sqlite");

    if let Err(err) = std::fs::remove_file(&sample_sqlite_path) {
        if err.kind() != ErrorKind::NotFound {
            return Err(format!(
                "Failed to remove sample SQLite database '{}': {err}",
                sample_sqlite_path.display()
            ));
        }
    }

    crate::sample::ensure_sample_sqlite(&sample_sqlite_path).await?;
    let sample_connection = sample::sample_connection_profile(&sample_sqlite_path);

    let removed_connection_ids = {
        let mut guard = state
            .connections
            .write()
            .map_err(|_| "Failed to update app state".to_string())?;

        let removed = guard
            .keys()
            .filter(|id| *id != sample::SAMPLE_CONNECTION_ID)
            .cloned()
            .collect::<Vec<_>>();

        guard.clear();
        guard.insert(sample_connection.id.clone(), sample_connection);

        let snapshot = guard.clone();
        drop(guard);
        persist_profiles(state, &snapshot)?;

        removed
    };

    for connection_id in removed_connection_ids {
        if let Err(err) = secrets::delete_password(&connection_id) {
            eprintln!(
                "Failed to clear password for connection '{}': {err}",
                connection_id
            );
        }
    }

    Ok(list_connections(state))
}

fn validate_and_build_connection(
    input: ConnectionInput,
    existing_profile: Option<&ConnectionProfile>,
) -> Result<ConnectionProfile, String> {
    let ConnectionInput {
        id,
        name,
        engine,
        path,
        host,
        port,
        database,
        username,
        password,
    } = input;

    let name = name.trim();
    if name.is_empty() {
        return Err("Connection name is required".to_string());
    }

    match engine {
        DbEngine::Sqlite => {
            let path = required(path.as_deref(), "SQLite path")?;
            let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());

            Ok(ConnectionProfile {
                id,
                name: name.to_string(),
                engine: DbEngine::Sqlite,
                path: Some(path.to_string()),
                host: None,
                port: None,
                database: None,
                username: None,
                password: None,
            })
        }
        DbEngine::Postgres => {
            let host = required(host.as_deref(), "Host")?.to_string();
            let database = required(database.as_deref(), "Database")?.to_string();
            let username = required(username.as_deref(), "Username")?.to_string();
            let password = resolve_network_password(password, existing_profile)?;
            let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());

            Ok(ConnectionProfile {
                id,
                name: name.to_string(),
                engine: DbEngine::Postgres,
                path: None,
                host: Some(host),
                port: Some(port.unwrap_or(5432)),
                database: Some(database),
                username: Some(username),
                password: Some(password),
            })
        }
        DbEngine::Mysql => {
            let host = required(host.as_deref(), "Host")?.to_string();
            let database = required(database.as_deref(), "Database")?.to_string();
            let username = required(username.as_deref(), "Username")?.to_string();
            let password = resolve_network_password(password, existing_profile)?;
            let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());

            Ok(ConnectionProfile {
                id,
                name: name.to_string(),
                engine: DbEngine::Mysql,
                path: None,
                host: Some(host),
                port: Some(port.unwrap_or(3306)),
                database: Some(database),
                username: Some(username),
                password: Some(password),
            })
        }
    }
}

fn resolve_network_password(
    candidate: Option<String>,
    existing_profile: Option<&ConnectionProfile>,
) -> Result<String, String> {
    if let Some(password) = candidate {
        if !password.trim().is_empty() {
            return Ok(password);
        }
    }

    if let Some(profile) = existing_profile {
        if matches!(profile.engine, DbEngine::Postgres | DbEngine::Mysql) {
            return match secrets::load_password(&profile.id) {
                Ok(password) => Ok(password),
                Err(err) if err.contains("No saved password") => {
                    Err("Password is required".to_string())
                }
                Err(err) => Err(err),
            };
        }
    }

    Err("Password is required".to_string())
}

fn to_connection_input(profile: &ConnectionProfile) -> ConnectionInput {
    ConnectionInput {
        id: Some(profile.id.clone()),
        name: profile.name.clone(),
        engine: profile.engine.clone(),
        path: profile.path.clone(),
        host: profile.host.clone(),
        port: profile.port,
        database: profile.database.clone(),
        username: profile.username.clone(),
        password: None,
    }
}
