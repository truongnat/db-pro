mod commands;
mod dto;

use std::sync::Arc;

use db_pro_core::application::{ConnectionRegistry, ConnectionService, ExportService, QueryService, SchemaService};
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;
use db_pro_infrastructure::secret::keyring_vault::KeyringVault;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();

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
                    let vault = KeyringVault::new("com.dbpro.app", secrets_dir);
                    #[cfg(debug_assertions)]
                    let vault = vault.with_fallback();
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
                    Arc::clone(&registry),
                );

                let schema_service = SchemaService::new(
                    Box::new(Arc::clone(&connector)),
                    Box::new(meta_store.clone()),
                    Arc::clone(&registry),
                );

                let export_service = ExportService::new(Box::new(Arc::clone(&connector)), Arc::clone(&registry));

                handle.manage(conn_service);
                handle.manage(query_service);
                handle.manage(schema_service);
                handle.manage(export_service);
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
            commands::execute_query,
            commands::explain_query,
            commands::get_query_history,
            commands::save_query,
            commands::list_saved_queries,
            commands::delete_saved_query,
            commands::introspect,
            commands::get_table_info,
            commands::get_table_ddl,
            commands::invalidate_cache,
            commands::export_csv,
            commands::export_json,
            commands::export_excel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
