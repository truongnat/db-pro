use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use db_pro_core::domain::query::QueryResult;
use tokio::sync::{mpsc, oneshot};

use crate::{ConnectionSummary, DbProRuntime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeRequestId(pub u64);

#[derive(Debug)]
pub enum RuntimeCommand {
    ListConnections { request_id: RuntimeRequestId },
    Connect {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    ExecuteQuery {
        request_id: RuntimeRequestId,
        connection_id: String,
        sql: String,
    },
    CancelQuery { request_id: RuntimeRequestId },
}

#[derive(Debug)]
pub enum RuntimeEvent {
    ConnectionsLoaded {
        request_id: RuntimeRequestId,
        connections: Vec<ConnectionSummary>,
    },
    Connected {
        request_id: RuntimeRequestId,
        connection_id: String,
    },
    QueryCompleted {
        request_id: RuntimeRequestId,
        result: QueryResult,
    },
    QueryCancelled {
        request_id: RuntimeRequestId,
    },
    Failed {
        request_id: RuntimeRequestId,
        message: String,
    },
}

type CancelMap = Arc<Mutex<HashMap<RuntimeRequestId, oneshot::Sender<()>>>>;

/// Spawn the async worker that translates native UI commands into application
/// service calls. The UI receives only typed events and never sees credentials
/// or infrastructure handles.
pub fn spawn_worker(
    runtime: Arc<DbProRuntime>,
    capacity: usize,
) -> (mpsc::Sender<RuntimeCommand>, mpsc::Receiver<RuntimeEvent>) {
    let (command_tx, mut command_rx) = mpsc::channel(capacity);
    let (event_tx, event_rx) = mpsc::channel(capacity);
    let cancellations: CancelMap = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(async move {
        while let Some(command) = command_rx.recv().await {
            match command {
                RuntimeCommand::ListConnections { request_id } => {
                    let event = match runtime.connection_api().list().await {
                        Ok(connections) => RuntimeEvent::ConnectionsLoaded { request_id, connections },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::Connect {
                    request_id,
                    connection_id,
                } => {
                    let event = match runtime.connection_api().connect(&connection_id).await {
                        Ok(()) => RuntimeEvent::Connected {
                            request_id,
                            connection_id,
                        },
                        Err(error) => RuntimeEvent::Failed {
                            request_id,
                            message: error.message,
                        },
                    };
                    let _ = event_tx.send(event).await;
                }
                RuntimeCommand::ExecuteQuery {
                    request_id,
                    connection_id,
                    sql,
                } => {
                    let (cancel_tx, cancel_rx) = oneshot::channel();
                    cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .insert(request_id, cancel_tx);
                    let query_api = runtime.query_api();
                    let event_tx = event_tx.clone();
                    let cancellations = Arc::clone(&cancellations);
                    tokio::spawn(async move {
                        let result = tokio::select! {
                            result = query_api.execute(&connection_id, &sql) => result,
                            _ = cancel_rx => Err(crate::DbErrorDto {
                                code: "QUERY_CANCELLED".to_owned(),
                                message: "Query cancelled".to_owned(),
                                message_id: "error.query.cancelled".to_owned(),
                                retryable: false,
                            }),
                        };
                        let event = match result {
                            Ok(result) => RuntimeEvent::QueryCompleted { request_id, result },
                            Err(error) if error.code == "QUERY_CANCELLED" => RuntimeEvent::QueryCancelled { request_id },
                            Err(error) => RuntimeEvent::Failed {
                                request_id,
                                message: error.message,
                            },
                        };
                        cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .remove(&request_id);
                        let _ = event_tx.send(event).await;
                    });
                }
                RuntimeCommand::CancelQuery { request_id } => {
                    if let Some(sender) = cancellations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .remove(&request_id)
                    {
                        let _ = sender.send(());
                    }
                }
            }
        }
    });

    (command_tx, event_rx)
}
