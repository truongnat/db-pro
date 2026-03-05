use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::models::{ConnectionProfile, DbEngine};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedConnections {
    #[serde(default = "default_version")]
    version: u8,
    #[serde(default)]
    connections: Vec<ConnectionProfile>,
}

fn default_version() -> u8 {
    1
}

pub fn load_connections(path: &Path) -> Result<HashMap<String, ConnectionProfile>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let raw = fs::read_to_string(path).map_err(|err| {
        format!(
            "Failed to read connection store '{}': {err}",
            path.display()
        )
    })?;

    let mut payload: PersistedConnections = serde_json::from_str(&raw)
        .map_err(|err| format!("Invalid connection store JSON: {err}"))?;

    for profile in &mut payload.connections {
        if matches!(profile.engine, DbEngine::Postgres | DbEngine::Mysql) {
            profile.password = None;
        }
    }

    Ok(payload
        .connections
        .into_iter()
        .map(|profile| (profile.id.clone(), profile))
        .collect())
}

pub fn save_connections(
    path: &Path,
    profiles: &HashMap<String, ConnectionProfile>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Failed to create connection store directory '{}': {err}",
                parent.display()
            )
        })?;
    }

    let mut list = profiles.values().cloned().collect::<Vec<_>>();
    for profile in &mut list {
        profile.password = None;
    }
    list.sort_by(|left, right| left.name.cmp(&right.name));

    let payload = PersistedConnections {
        version: 1,
        connections: list,
    };

    let raw = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to encode connection store JSON: {err}"))?;

    fs::write(path, raw).map_err(|err| {
        format!(
            "Failed to write connection store '{}': {err}",
            path.display()
        )
    })
}

pub fn quarantine_corrupted_store(path: &Path) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let source_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("connections.json");
    let backup_name = format!("{source_name}.corrupted-{timestamp}.bak");
    let backup_path = path.with_file_name(backup_name);

    fs::rename(path, &backup_path).map_err(|err| {
        format!(
            "Failed to move corrupted connection store '{}' to '{}': {err}",
            path.display(),
            backup_path.display()
        )
    })?;

    Ok(Some(backup_path))
}
