use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread;

use db_pro_core::application::sql_builder::{FilterOp, SortClause, SortDir, TableFilter};
use db_pro_core::domain::query::CellValue;
use db_pro_runtime::{spawn_worker, DbProRuntime, RuntimeCommand, RuntimeEvent, RuntimeRequestId};
use db_pro_ui::{
    AgentMessage, AgentRole, DbProApp, DbProTheme, RequestId, TaskBridge, UiCell, UiColumn, UiCommand,
    UiConnectionDraft, UiConnectionSummary, UiDriver, UiEvent, UiFunctionSummary, UiQueryFolderSummary, UiQueryResult,
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
    init_tracing();
    let tokio_runtime = Builder::new_multi_thread().enable_all().build()?;
    let data_dir = resolve_data_dir();
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();

    let (runtime_tx, runtime_rx) = tokio_runtime.block_on(async {
        let runtime = DbProRuntime::new(data_dir).await?;
        seed_default_connection(&runtime).await;
        Ok::<_, Box<dyn Error>>(spawn_worker(runtime, 64))
    })?;

    let command_runtime_tx = runtime_tx.clone();
    let command_handle = tokio_runtime.handle().clone();
    let picker_event_tx = event_tx.clone();
    thread::spawn(move || {
        let send_picked = |request_id, kind: &str, path: Option<String>| {
            // Fire-and-forget: a picker result is only dropped once the UI has
            // gone away, which means the whole app is shutting down.
            let _ = picker_event_tx.send(UiEvent::FilePicked {
                request_id,
                kind: kind.to_owned(),
                path,
            });
        };
        while let Ok(command) = command_rx.recv() {
            match command {
                UiCommand::PickSqliteFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .add_filter("SQLite database", &["db", "sqlite", "sqlite3"])
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "sqlite", path);
                    continue;
                }
                UiCommand::PickBackupFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup output")
                        .save_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "backup", path);
                    continue;
                }
                UiCommand::PickRestoreFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup to restore")
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "restore", path);
                    continue;
                }
                UiCommand::PickSshPrivateKey { request_id } => {
                    let path = rfd::FileDialog::new()
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "ssh-key", path);
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

    spawn_event_pump(runtime_rx, event_tx, tokio_runtime.handle().clone());

    run_native_app(bridge)
}

fn init_tracing() {
    // Best-effort: a global tracing subscriber may already be installed when
    // the app is embedded in a host process, which is not a failure.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "db_pro_runtime=info".to_owned()))
        .try_init();
}

fn resolve_data_dir() -> std::path::PathBuf {
    std::env::var_os("DB_PRO_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("current directory is available")
                .join(".db-pro-data")
        })
}

/// Seeds the demo PostgreSQL connection the first time the app runs.
async fn seed_default_connection(runtime: &DbProRuntime) {
    let existing = runtime.connections().list().await.unwrap_or_default();
    if existing
        .iter()
        .any(|c| c.config.database == "fullstack_starter" && c.config.port == 5432)
    {
        return;
    }
    let config = db_pro_core::domain::connection::ConnectionConfig {
        name: "Xe Lạc Hồng (PostgreSQL)".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "fullstack_starter".to_owned(),
        username: "postgres".to_owned(),
        driver: db_pro_core::domain::connection::DriverType::Postgres,
        ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 30_000,
        max_rows: 500,
        color: Some("#6366f1".to_owned()),
        tags: vec!["docker".to_owned(), "xe-lac-hong".to_owned()],
        group: None,
        readonly: false,
    };
    if let Err(err) = runtime.connections().create(config, "postgres").await {
        tracing::warn!("failed to seed default Xe Lạc Hồng connection: {err}");
    }
}

/// Forwards translated runtime events to the UI, stopping when the UI is gone.
fn spawn_event_pump(
    mut runtime_rx: tokio::sync::mpsc::Receiver<RuntimeEvent>,
    event_tx: Sender<UiEvent>,
    event_handle: tokio::runtime::Handle,
) {
    event_handle.spawn(async move {
        while let Some(event) = runtime_rx.recv().await {
            if let Some(event) = translate_event(event) {
                if event_tx.send(event).is_err() {
                    break;
                }
            }
        }
    });
}

fn run_native_app(bridge: TaskBridge) -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DB Pro")
            .with_maximized(true)
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DB Pro",
        options,
        Box::new(|creation_context| {
            // Re-apply the product default after eframe restores its persisted window frame.
            creation_context
                .egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Maximized(true));
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
