mod commands;
mod models;
mod sample;
mod secrets;
mod state;
mod storage;

use std::collections::HashMap;
use std::path::Path;

use models::{ConnectionProfile, DbEngine};
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let storage_path = app_data_dir.join("connections.json");
            let mut connections = match storage::load_connections(&storage_path) {
                Ok(connections) => connections,
                Err(load_error) => {
                    let backup_path = storage::quarantine_corrupted_store(&storage_path)
                        .map_err(std::io::Error::other)?;
                    if let Some(path) = backup_path {
                        eprintln!(
                            "Connection store is invalid: {load_error}. Backed up to '{}'. Starting with empty store.",
                            path.display()
                        );
                    } else {
                        eprintln!(
                            "Connection store is invalid: {load_error}. Starting with empty store."
                        );
                    }
                    HashMap::new()
                }
            };

            let mut changed = normalize_sqlite_paths_in_store(&mut connections, &app_data_dir);

            let sample_sqlite_path = app_data_dir.join("db-pro-sample.sqlite");
            tauri::async_runtime::block_on(sample::ensure_sample_sqlite(&sample_sqlite_path))
                .map_err(std::io::Error::other)?;

            let sample_connection = sample::sample_connection_profile(&sample_sqlite_path);

            if !connections.contains_key(&sample_connection.id) {
                connections.insert(sample_connection.id.clone(), sample_connection);
                changed = true;
            }

            if changed {
                storage::save_connections(&storage_path, &connections)
                    .map_err(std::io::Error::other)?;
            }

            app.manage(AppState::new(storage_path, connections));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_connections,
            commands::get_connection,
            commands::upsert_connection,
            commands::delete_connection,
            commands::reset_connections,
            commands::test_connection,
            commands::execute_query,
            commands::cancel_query,
            commands::load_navigator,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn normalize_sqlite_paths_in_store(
    connections: &mut HashMap<String, ConnectionProfile>,
    app_data_dir: &Path,
) -> bool {
    let mut changed = false;

    for profile in connections.values_mut() {
        if !matches!(profile.engine, DbEngine::Sqlite) {
            continue;
        }

        let Some(raw_path) = profile.path.clone() else {
            continue;
        };

        if raw_path == ":memory:" {
            continue;
        }

        let candidate = Path::new(&raw_path);
        if candidate.is_absolute() {
            continue;
        }

        profile.path = Some(app_data_dir.join(candidate).to_string_lossy().to_string());
        changed = true;
    }

    changed
}
