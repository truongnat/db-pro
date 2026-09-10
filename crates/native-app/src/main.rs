use std::error::Error;
use std::thread;

use db_pro_runtime::{spawn_worker, DbProRuntime, RuntimeCommand, RuntimeEvent, RuntimeRequestId};
use db_pro_ui::{
    DbProApp, TaskBridge, UiCell, UiColumn, UiCommand, UiConnectionDraft, UiConnectionSummary, UiDriver, UiSslMode,
    UiEvent, UiQueryResult, UiSavedQuerySummary,
};
use eframe::egui;
use tokio::runtime::Builder;

fn main() -> Result<(), Box<dyn Error>> {
    let tokio_runtime = Builder::new_multi_thread().enable_all().build()?;
    let data_dir = std::env::var_os("DB_PRO_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("current directory is available").join(".db-pro-data"));
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
                UiCommand::PickSshPrivateKey { request_id } => {
                    let path = rfd::FileDialog::new().pick_file().map(|path| path.to_string_lossy().into_owned());
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
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DB Pro",
        options,
        Box::new(|creation_context| {
            Ok(Box::new(DbProApp::with_task_bridge_and_storage(
                bridge,
                creation_context.storage,
            )))
        }),
    )?;
    Ok(())
}

fn draft_to_domain(draft: UiConnectionDraft) -> Option<(db_pro_core::domain::connection::ConnectionConfig, String)> {
    let port = draft.port.parse::<u16>().ok()?;
    let driver = match draft.driver {
        UiDriver::Postgres => db_pro_core::domain::connection::DriverType::Postgres,
        UiDriver::Sqlite => db_pro_core::domain::connection::DriverType::SQLite,
    };
    let ssl_mode = match draft.ssl_mode {
        UiSslMode::Disable => db_pro_core::domain::connection::SslMode::Disable,
        UiSslMode::Require => db_pro_core::domain::connection::SslMode::Require,
        UiSslMode::VerifyCa => db_pro_core::domain::connection::SslMode::VerifyCa,
        UiSslMode::VerifyFull => db_pro_core::domain::connection::SslMode::VerifyFull,
    };
    let ssh_tunnel = if draft.ssh_tunnel_enabled {
        Some(db_pro_core::domain::connection::SshTunnelConfig {
            host: draft.ssh_host.clone(),
            port: draft.ssh_port.parse::<u16>().ok()?,
            user: draft.ssh_user.clone(),
            private_key_path: draft.ssh_private_key.clone(),
            password: None,
        })
    } else {
        None
    };
    Some((
        db_pro_core::domain::connection::ConnectionConfig {
            name: draft.name,
            host: draft.host,
            port,
            database: draft.database,
            username: draft.username,
            driver,
            ssl_mode,
            ssh_tunnel,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: Vec::new(),
            group: None,
            readonly: draft.readonly,
        },
        draft.password,
    ))
}

fn translate_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::OpenQuery | UiCommand::PickSqliteFile { .. } | UiCommand::PickSshPrivateKey { .. } => None,
        UiCommand::ListSavedQueries { request_id, connection_id } => Some(RuntimeCommand::ListSavedQueries {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::SaveQuery { request_id, connection_id, name, sql, folder } => Some(RuntimeCommand::SaveQuery {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            name,
            sql,
            folder,
        }),
        UiCommand::CreateQueryFolder { request_id, connection_id, name } => Some(RuntimeCommand::CreateQueryFolder {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            name,
        }),
        UiCommand::ListConnections { request_id } => Some(RuntimeCommand::ListConnections {
            request_id: RuntimeRequestId(request_id.0),
        }),
        UiCommand::CreateConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::CreateConnection {
                request_id: RuntimeRequestId(request_id.0),
                config,
                password,
            })
        }
        UiCommand::UpdateConnection { request_id, connection_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::UpdateConnection {
                request_id: RuntimeRequestId(request_id.0),
                connection_id,
                config,
                password: Some(password),
            })
        }
        UiCommand::TestConnection { request_id, draft } => {
            let (config, password) = draft_to_domain(draft)?;
            Some(RuntimeCommand::TestConnection {
                request_id: RuntimeRequestId(request_id.0),
                config,
                password,
            })
        }
        UiCommand::DeleteConnection { request_id, connection_id } => Some(RuntimeCommand::DeleteConnection {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::Connect {
            request_id,
            connection_id,
        } => Some(RuntimeCommand::Connect {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
        }),
        UiCommand::RunQuery {
            request_id,
            connection_id,
            sql,
        } => Some(RuntimeCommand::ExecuteQuery {
            request_id: RuntimeRequestId(request_id.0),
            connection_id,
            sql,
        }),
        UiCommand::CancelQuery { request_id } => Some(RuntimeCommand::CancelQuery {
            request_id: RuntimeRequestId(request_id.0),
        }),
    }
}

fn map_cell(cell: db_pro_core::domain::query::CellValue) -> UiCell {
    use db_pro_core::domain::query::CellValue;

    match cell {
        CellValue::Null => UiCell::Null,
        CellValue::Bool(value) => UiCell::Boolean(value),
        CellValue::Int64(value) => UiCell::Number(value.to_string()),
        CellValue::Float64(value) => UiCell::Number(value.to_string()),
        CellValue::Text(value)
        | CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => UiCell::Text(value),
        CellValue::Bytes(value) => UiCell::Bytes(format!("\\x{}", hex_encode(&value))),
        CellValue::Json(value) => UiCell::Json(value.to_string()),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn translate_event(event: RuntimeEvent) -> Option<UiEvent> {
    match event {
        RuntimeEvent::ConnectionsLoaded { request_id, connections } => Some(UiEvent::ConnectionsLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            connections: connections
                .into_iter()
                .map(|connection| UiConnectionSummary {
                    id: connection.id,
                    name: connection.name,
                    host: connection.host,
                    port: connection.port,
                    database: connection.database,
                    username: connection.username,
                    driver: connection.driver,
                    readonly: connection.readonly,
                })
                .collect(),
        }),
        RuntimeEvent::SavedQueriesLoaded { request_id, queries } => Some(UiEvent::SavedQueriesLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            queries: queries.into_iter().map(|query| UiSavedQuerySummary {
                id: query.id,
                name: query.name,
                sql: query.sql,
                folder: query.folder,
            }).collect(),
        }),
        RuntimeEvent::QueryFoldersLoaded { request_id, folders } => Some(UiEvent::QueryFoldersLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            folders,
        }),
        RuntimeEvent::OperationProgress { request_id, operation, status } => Some(UiEvent::OperationProgress {
            request_id: db_pro_ui::RequestId(request_id.0),
            operation: operation.to_owned(),
            status: status.to_owned(),
        }),
        RuntimeEvent::BackupCompleted { request_id, output_path, size_bytes } => Some(UiEvent::BackupCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            output_path,
            size_bytes,
        }),
        RuntimeEvent::OperationCompleted { request_id, operation } => Some(UiEvent::OperationCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            operation: operation.to_owned(),
        }),
        RuntimeEvent::Connected {
            request_id,
            connection_id,
        } => Some(UiEvent::Connected {
            request_id: db_pro_ui::RequestId(request_id.0),
            connection_id,
        }),
        RuntimeEvent::QueryCompleted { request_id, result } => {
            let row_count = result.row_count;
            let duration_ms = result.duration_ms;
            let columns = result
                .columns
                .into_iter()
                .map(|column| UiColumn {
                    name: column.name,
                    data_type: column.data_type,
                    nullable: column.nullable,
                })
                .collect();
            let rows = result
                .rows
                .into_iter()
                .map(|row| row.0.into_iter().map(map_cell).collect())
                .collect();
            Some(UiEvent::QueryCompleted {
                request_id: db_pro_ui::RequestId(request_id.0),
                result: UiQueryResult {
                    columns,
                    rows,
                    row_count,
                    duration_ms,
                },
            })
        },
        RuntimeEvent::QueryCancelled { request_id } => Some(UiEvent::QueryCancelled {
            request_id: db_pro_ui::RequestId(request_id.0),
        }),
        RuntimeEvent::Failed { request_id, message } => Some(UiEvent::QueryFailed {
            request_id: db_pro_ui::RequestId(request_id.0),
            message,
        }),
    }
}
