mod cancel;
mod commands;
mod dto;

use db_pro_runtime::DbProRuntime;
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
                let runtime = DbProRuntime::new(&data_dir)
                    .await
                    .expect("failed to initialize shared DB Pro runtime");

                handle.manage(runtime.connections());
                handle.manage(runtime.queries());
                handle.manage(runtime.schema());
                handle.manage(runtime.export());
                handle.manage(runtime.table_data());
                handle.manage(runtime.users());
                handle.manage(runtime.backup());
                handle.manage(runtime.data_diff());
                handle.manage(ExecutionRegistry::new());
                handle.manage(runtime.connector());
                handle.manage(runtime.registry());
                handle.manage(runtime.meta_store());
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
            commands::rename_saved_query,
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
            commands::reveal_backup_path,
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
