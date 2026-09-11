use std::error::Error;
use std::thread;

use db_pro_core::application::sql_builder::{FilterOp, SortClause, SortDir, TableFilter};
use db_pro_core::domain::query::CellValue;
use db_pro_runtime::{spawn_worker, DbProRuntime, RuntimeCommand, RuntimeEvent, RuntimeRequestId};
use db_pro_ui::{
    AgentMessage, AgentRole, DbProApp, DbProTheme, TaskBridge, UiCell, UiColumn, UiCommand, UiConnectionDraft,
    UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary, UiQueryFolderSummary, UiQueryResult,
    UiSavedQuerySummary, UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiSslMode, UiTableColumn,
    UiTableDataFilter, UiTableDataSort, UiTableForeignKey, UiTableIndex, UiTableInfo, UiTableSummary, UiTriggerSummary,
    UiViewSummary,
};
use eframe::egui;
use tokio::runtime::Builder;

mod translate;

#[cfg(test)]
pub(crate) use translate::draft_to_domain;
use translate::{translate_command, translate_event};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "db_pro_runtime=info".to_owned()))
        .try_init();
    let tokio_runtime = Builder::new_multi_thread().enable_all().build()?;
    let data_dir = std::env::var_os("DB_PRO_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("current directory is available")
                .join(".db-pro-data")
        });
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();

    let (runtime_tx, mut runtime_rx) = tokio_runtime.block_on(async {
        let runtime = DbProRuntime::new(data_dir).await?;
        Ok::<_, Box<dyn Error>>(spawn_worker(runtime, 64))
    })?;

    let command_runtime_tx = runtime_tx.clone();
    let command_handle = tokio_runtime.handle().clone();
    let picker_event_tx = event_tx.clone();
    thread::spawn(move || {
        while let Ok(command) = command_rx.recv() {
            match command {
                UiCommand::PickSqliteFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .add_filter("SQLite database", &["db", "sqlite", "sqlite3"])
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    let _ = picker_event_tx.send(UiEvent::FilePicked {
                        request_id,
                        kind: "sqlite".to_owned(),
                        path,
                    });
                    continue;
                }
                UiCommand::PickBackupFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup output")
                        .save_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    let _ = picker_event_tx.send(UiEvent::FilePicked {
                        request_id,
                        kind: "backup".to_owned(),
                        path,
                    });
                    continue;
                }
                UiCommand::PickRestoreFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup to restore")
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    let _ = picker_event_tx.send(UiEvent::FilePicked {
                        request_id,
                        kind: "restore".to_owned(),
                        path,
                    });
                    continue;
                }
                UiCommand::PickSshPrivateKey { request_id } => {
                    let path = rfd::FileDialog::new()
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    let _ = picker_event_tx.send(UiEvent::FilePicked {
                        request_id,
                        kind: "ssh-key".to_owned(),
                        path,
                    });
                    continue;
                }
                command => {
                    let Some(command) = translate_command(command) else {
                        continue;
                    };
                    let send_result = command_handle.block_on(command_runtime_tx.send(command));
                    if send_result.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let event_handle = tokio_runtime.handle().clone();
    event_handle.spawn(async move {
        while let Some(event) = runtime_rx.recv().await {
            if let Some(event) = translate_event(event) {
                if event_tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DB Pro")
            .with_maximized(true)
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DB Pro",
        options,
        Box::new(|creation_context| {
            DbProTheme::install_fonts(&creation_context.egui_ctx);
            Ok(Box::new(DbProApp::with_task_bridge_and_storage(
                bridge,
                creation_context.storage,
            )))
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_draft_drops_postgres_only_credentials() {
        let draft = UiConnectionDraft {
            driver: UiDriver::Sqlite,
            database: "/tmp/app.sqlite".to_owned(),
            password: "should-not-cross-boundary".to_owned(),
            ssh_tunnel_enabled: true,
            ssh_host: "bastion".to_owned(),
            ..Default::default()
        };

        let (config, password) = draft_to_domain(draft).expect("valid SQLite draft");

        assert!(password.is_empty());
        assert!(config.ssh_tunnel.is_none());
    }
}
