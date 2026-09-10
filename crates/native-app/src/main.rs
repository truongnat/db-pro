use std::error::Error;
use std::thread;

use db_pro_runtime::{spawn_worker, DbProRuntime, RuntimeCommand, RuntimeEvent, RuntimeRequestId};
use db_pro_ui::{DbProApp, TaskBridge, UiCommand, UiConnectionSummary, UiEvent};
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
    thread::spawn(move || {
        while let Ok(command) = command_rx.recv() {
            let Some(command) = translate_command(command) else {
                continue;
            };
            let send_result = command_handle.block_on(command_runtime_tx.send(command));
            if send_result.is_err() {
                break;
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
        Box::new(|_creation_context| Ok(Box::new(DbProApp::with_task_bridge(bridge)))),
    )?;
    Ok(())
}

fn translate_command(command: UiCommand) -> Option<RuntimeCommand> {
    match command {
        UiCommand::OpenQuery => None,
        UiCommand::ListConnections { request_id } => Some(RuntimeCommand::ListConnections {
            request_id: RuntimeRequestId(request_id.0),
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

fn translate_event(event: RuntimeEvent) -> Option<UiEvent> {
    match event {
        RuntimeEvent::ConnectionsLoaded { request_id, connections } => Some(UiEvent::ConnectionsLoaded {
            request_id: db_pro_ui::RequestId(request_id.0),
            connections: connections
                .into_iter()
                .map(|connection| UiConnectionSummary {
                    id: connection.id,
                    name: connection.name,
                    driver: connection.driver,
                    readonly: connection.readonly,
                })
                .collect(),
        }),
        RuntimeEvent::Connected {
            request_id,
            connection_id,
        } => Some(UiEvent::Connected {
            request_id: db_pro_ui::RequestId(request_id.0),
            connection_id,
        }),
        RuntimeEvent::QueryCompleted { request_id, result } => Some(UiEvent::QueryCompleted {
            request_id: db_pro_ui::RequestId(request_id.0),
            row_count: result.row_count,
        }),
        RuntimeEvent::QueryCancelled { request_id } => Some(UiEvent::QueryFailed {
            request_id: db_pro_ui::RequestId(request_id.0),
            message: "Query cancelled".to_owned(),
        }),
        RuntimeEvent::Failed { request_id, message } => Some(UiEvent::QueryFailed {
            request_id: db_pro_ui::RequestId(request_id.0),
            message,
        }),
    }
}
