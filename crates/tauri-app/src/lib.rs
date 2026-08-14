mod cancel;
mod commands;
mod dto;

use std::sync::Arc;

use db_pro_core::application::{
    BackupService, ConnectionRegistry, ConnectionService, DataDiffService, ExportService, QueryService, SchemaService,
    TableDataService, UserService,
};
use db_pro_infrastructure::backup::pg_dump::PgDumpEngine;
use db_pro_infrastructure::backup::sqlite_backup::SqliteBackupEngine;
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;
use db_pro_infrastructure::postgres::user_manager::PostgresUserManager;
use db_pro_infrastructure::secret::keyring_vault::KeyringVault;
use tauri::Manager;

use crate::cancel::ExecutionRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Safety net: never let the main window stay hidden forever. If the
            // frontend never invokes `finish_startup` (e.g. bootstrap throws),
            // force the handoff after a short grace period.
            let timeout_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(6)).await;
                let _ = crate::commands::finish_startup_inner(&timeout_handle);
            });

            tauri::async_runtime::block_on(async move {
                let data_dir = handle.path().app_data_dir().expect("failed to get app data dir");
                std::fs::create_dir_all(&data_dir).expect("failed to create data dir");

                let meta_path = data_dir.join("meta.db");
                let secrets_dir = data_dir.join("secrets");
                std::fs::create_dir_all(&secrets_dir).expect("failed to create secrets dir");

                let meta_store = SQLiteMetaStore::new(&meta_path.to_string_lossy())
                    .await
                    .expect("failed to initialize meta store");

                let secret_store = {
                    // Production: fail-closed when OS keyring is unavailable.
                    // Do NOT call .with_fallback() — the encrypted-file fallback
                    // derives its key from the service name (not a secret) and is
                    // intended for development/CI only. See #142.
                    let vault = KeyringVault::new("com.dbpro.app", secrets_dir);
                    Arc::new(vault)
                };

                let registry = Arc::new(ConnectionRegistry::new());
                let connector: Arc<CompositeConnector> = Arc::new(CompositeConnector::new());

                let conn_service = ConnectionService::new(
                    Box::new(Arc::clone(&connector)),
                    Box::new(meta_store.clone()),
                    Box::new(secret_store.clone()),
                    Arc::clone(&registry),
                );

                let query_service = QueryService::new(
                    Box::new(Arc::clone(&connector)),
                    Box::new(meta_store.clone()),
                    Box::new(meta_store.clone()),
                    Box::new(meta_store.clone()),
                    Arc::clone(&registry),
                    Box::new(meta_store.clone()),
                );

                let schema_service = SchemaService::new(
                    Box::new(Arc::clone(&connector)),
                    Box::new(meta_store.clone()),
                    Arc::clone(&registry),
                    Box::new(meta_store.clone()),
                );

                let export_service = ExportService::new(Box::new(Arc::clone(&connector)), Arc::clone(&registry));

                let table_data_service = TableDataService::new(
                    Box::new(Arc::clone(&connector)),
                    Arc::clone(&registry),
                    Box::new(meta_store.clone()),
                );

                let pg_connector = connector.postgres_connector();
                let user_manager = PostgresUserManager::new(pg_connector);
                let user_service = UserService::new(
                    Box::new(user_manager),
                    Arc::clone(&registry),
                    Box::new(meta_store.clone()),
                );

                let backup_service = BackupService::new(
                    Box::new(meta_store.clone()),
                    Box::new(Arc::clone(&secret_store)),
                    Box::new(|host: &str, port: u16, database: &str, username: &str| {
                        let config = db_pro_core::domain::connection::ConnectionConfig {
                            name: String::new(),
                            host: host.to_string(),
                            port,
                            database: database.to_string(),
                            username: username.to_string(),
                            driver: db_pro_core::domain::connection::DriverType::Postgres,
                            ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
                            ssh_tunnel: None,
                            query_timeout_ms: 30_000,
                            max_rows: 500,
                            color: None,
                            tags: vec![],
                            group: None,
                            readonly: false,
                        };
                        Box::new(PgDumpEngine::new(config)) as Box<dyn db_pro_core::ports::BackupEngine>
                    }),
                    Box::new(|database: &str| {
                        let config = db_pro_core::domain::connection::ConnectionConfig {
                            name: String::new(),
                            host: String::new(),
                            port: 0,
                            database: database.to_string(),
                            username: String::new(),
                            driver: db_pro_core::domain::connection::DriverType::SQLite,
                            ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
                            ssh_tunnel: None,
                            query_timeout_ms: 30_000,
                            max_rows: 500,
                            color: None,
                            tags: vec![],
                            group: None,
                            readonly: false,
                        };
                        Box::new(SqliteBackupEngine::new(config)) as Box<dyn db_pro_core::ports::BackupEngine>
                    }),
                );

                let data_diff_service = DataDiffService::new(Box::new(Arc::clone(&connector)), Arc::clone(&registry));

                handle.manage(conn_service);
                handle.manage(query_service);
                handle.manage(schema_service);
                handle.manage(export_service);
                handle.manage(table_data_service);
                handle.manage(user_service);
                handle.manage(backup_service);
                handle.manage(data_diff_service);
                handle.manage(ExecutionRegistry::new());
                handle.manage(Arc::clone(&connector) as Arc<CompositeConnector>);
                handle.manage(Arc::clone(&registry) as Arc<ConnectionRegistry>);
                handle.manage(meta_store);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_connections,
            commands::get_connection,
            commands::create_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::connect,
            commands::disconnect,
            commands::test_ssh_tunnel,
            commands::execute_query,
            commands::cancel_query,
            commands::execute_query_multi,
            commands::explain_query,
            commands::get_query_history,
            commands::save_query,
            commands::list_saved_queries,
            commands::delete_saved_query,
            commands::create_folder,
            commands::list_folders,
            commands::delete_folder,
            commands::save_run_config,
            commands::list_run_configs,
            commands::delete_run_config,
            commands::introspect,
            commands::get_table_info,
            commands::get_table_ddl,
            commands::execute_ddl,
            commands::execute_ddl_batch,
            commands::create_index,
            commands::drop_index,
            commands::create_trigger,
            commands::drop_trigger,
            commands::invalidate_cache,
            commands::export_csv,
            commands::export_json,
            commands::export_excel,
            commands::fetch_table_rows,
            commands::insert_table_row,
            commands::update_table_row,
            commands::delete_table_row,
            commands::list_users,
            commands::create_role,
            commands::drop_role,
            commands::list_privileges,
            commands::grant_privilege,
            commands::revoke_privilege,
            commands::backup_database,
            commands::restore_database,
            commands::diff_schemas,
            commands::diff_table_data,
            commands::get_object_dependencies,
            commands::list_partitions,
            commands::list_tablespaces,
            commands::rename_schema_object,
            commands::finish_startup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
