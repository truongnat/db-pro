use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::SqliteConnectOptions;
use url::Url;

use crate::models::{ConnectionProfile, DbEngine};
use crate::secrets;
use crate::state::AppState;
use crate::storage;

pub const DEFAULT_QUERY_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_NAVIGATOR_TIMEOUT_MS: u64 = 12_000;
pub const DEFAULT_CONNECTION_TEST_TIMEOUT_MS: u64 = 8_000;
pub const MIN_QUERY_TIMEOUT_MS: u64 = 1_000;
pub const MAX_QUERY_TIMEOUT_MS: u64 = 300_000;

pub fn required<'a>(value: Option<&'a str>, field_name: &str) -> Result<&'a str, String> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(format!("{field_name} is required"));
    }

    Ok(value)
}

pub fn get_profile(connection_id: &str, state: &AppState) -> Result<ConnectionProfile, String> {
    let guard = state
        .connections
        .read()
        .map_err(|_| "Failed to access app state".to_string())?;

    let mut profile = guard
        .get(connection_id)
        .cloned()
        .ok_or_else(|| "Connection not found".to_string())?;
    drop(guard);

    if matches!(profile.engine, DbEngine::Postgres | DbEngine::Mysql)
        && profile.password.as_deref().unwrap_or_default().is_empty()
    {
        profile.password = Some(secrets::load_password(&profile.id)?);
    }

    Ok(profile)
}

pub fn to_connection_url(profile: &ConnectionProfile) -> Result<String, String> {
    match profile.engine {
        DbEngine::Sqlite => {
            let path = required(profile.path.as_deref(), "SQLite path")?;
            if path == ":memory:" {
                return Ok("sqlite::memory:".to_string());
            }

            let sqlite_path = Path::new(path);
            if sqlite_path.is_absolute() {
                let file_url = Url::from_file_path(sqlite_path)
                    .map_err(|_| "Invalid absolute SQLite path".to_string())?;
                return Ok(format!("sqlite://{}", file_url.path()));
            }

            Ok(format!("sqlite://{}", path.replace(' ', "%20")))
        }
        DbEngine::Postgres => to_network_url("postgres", profile, 5432),
        DbEngine::Mysql => to_network_url("mysql", profile, 3306),
    }
}

fn to_network_url(
    scheme: &str,
    profile: &ConnectionProfile,
    default_port: u16,
) -> Result<String, String> {
    let host = required(profile.host.as_deref(), "Host")?;
    let username = required(profile.username.as_deref(), "Username")?;
    let password = required(profile.password.as_deref(), "Password")?;
    let database = required(profile.database.as_deref(), "Database")?;

    let mut url = Url::parse(&format!("{scheme}://{host}"))
        .map_err(|err| format!("Failed to build connection URL: {err}"))?;

    url.set_username(username)
        .map_err(|_| "Invalid username".to_string())?;
    url.set_password(Some(password))
        .map_err(|_| "Invalid password".to_string())?;
    url.set_port(Some(profile.port.unwrap_or(default_port)))
        .map_err(|_| "Invalid port".to_string())?;
    url.set_path(&format!("/{database}"));

    Ok(url.to_string())
}

pub fn normalize_sqlite_profile_path(profile: &mut ConnectionProfile, base_dir: &Path) {
    if !matches!(profile.engine, DbEngine::Sqlite) {
        return;
    }

    let Some(raw_path) = profile.path.as_deref() else {
        return;
    };

    if raw_path == ":memory:" {
        return;
    }

    let path = Path::new(raw_path);
    if path.is_absolute() {
        return;
    }

    let normalized = base_dir.join(path);
    profile.path = Some(normalized.to_string_lossy().to_string());
}

pub fn sqlite_connect_options(profile: &ConnectionProfile) -> Result<SqliteConnectOptions, String> {
    let sqlite_path = required(profile.path.as_deref(), "SQLite path")?;
    if sqlite_path == ":memory:" {
        return Ok(SqliteConnectOptions::new()
            .in_memory(true)
            .foreign_keys(true));
    }

    Ok(SqliteConnectOptions::new()
        .filename(Path::new(sqlite_path))
        .create_if_missing(true)
        .foreign_keys(true))
}

pub async fn with_timeout<T, F>(timeout_ms: u64, task: F, operation: &str) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    match tokio::time::timeout(Duration::from_millis(timeout_ms), task).await {
        Ok(result) => result,
        Err(_) => Err(format!("{operation} timed out after {timeout_ms} ms")),
    }
}

pub fn persist_profiles(
    state: &AppState,
    snapshot: &HashMap<String, ConnectionProfile>,
) -> Result<(), String> {
    storage::save_connections(&state.storage_path, snapshot)
}
