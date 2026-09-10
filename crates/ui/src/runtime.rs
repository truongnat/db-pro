use std::sync::mpsc::{self, Receiver, Sender};

/// Stable identity for an async UI operation. Real backend tasks will reuse
/// this identity for cancellation and stale-result protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDriver {
    Postgres,
    Sqlite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSslMode {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionDraft {
    pub name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub driver: UiDriver,
    pub ssl_mode: UiSslMode,
    pub readonly: bool,
    pub ssh_tunnel_enabled: bool,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub ssh_private_key: String,
}

impl Default for UiConnectionDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: "localhost".to_owned(),
            port: "5432".to_owned(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            driver: UiDriver::Postgres,
            ssl_mode: UiSslMode::Disable,
            readonly: false,
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSchemaSummary {
    pub tables: Vec<String>,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSavedQuerySummary {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionSummary {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    OpenQuery,
    ListConnections { request_id: RequestId },
    IntrospectSchema { request_id: RequestId, connection_id: String },
    ListSavedQueries {
        request_id: RequestId,
        connection_id: String,
    },
    SaveQuery {
        request_id: RequestId,
        connection_id: String,
        name: String,
        sql: String,
        folder: Option<String>,
    },
    CreateQueryFolder {
        request_id: RequestId,
        connection_id: String,
        name: String,
    },
    RenameSavedQuery { request_id: RequestId, id: String, name: String },
    DeleteSavedQuery { request_id: RequestId, id: String },
    DeleteQueryFolder { request_id: RequestId, id: String },
    Connect {
        request_id: RequestId,
        connection_id: String,
    },
    CreateConnection {
        request_id: RequestId,
        draft: UiConnectionDraft,
    },
    UpdateConnection {
        request_id: RequestId,
        connection_id: String,
        draft: UiConnectionDraft,
    },
    TestConnection {
        request_id: RequestId,
        draft: UiConnectionDraft,
    },
    PickSqliteFile { request_id: RequestId },
    PickSshPrivateKey { request_id: RequestId },
    DeleteConnection {
        request_id: RequestId,
        connection_id: String,
    },
    RunQuery {
        request_id: RequestId,
        connection_id: String,
        sql: String,
    },
    CancelQuery { request_id: RequestId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiCell {
    Null,
    Boolean(bool),
    Number(String),
    Text(String),
    Json(String),
    Bytes(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiQueryResult {
    pub columns: Vec<UiColumn>,
    pub rows: Vec<Vec<UiCell>>,
    pub row_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    ConnectionsLoaded {
        request_id: RequestId,
        connections: Vec<UiConnectionSummary>,
    },
    SavedQueriesLoaded {
        request_id: RequestId,
        queries: Vec<UiSavedQuerySummary>,
    },
    SchemaLoaded { request_id: RequestId, schema: UiSchemaSummary },
    QueryFoldersLoaded {
        request_id: RequestId,
        folders: Vec<String>,
    },
    FilePicked {
        request_id: RequestId,
        kind: String,
        path: Option<String>,
    },
    OperationProgress {
        request_id: RequestId,
        operation: String,
        status: String,
    },
    BackupCompleted {
        request_id: RequestId,
        output_path: String,
        size_bytes: u64,
    },
    OperationCompleted {
        request_id: RequestId,
        operation: String,
    },
    Connected {
        request_id: RequestId,
        connection_id: String,
    },
    QueryQueued { request_id: RequestId },
    QueryCompleted {
        request_id: RequestId,
        result: UiQueryResult,
    },
    QueryCancelled { request_id: RequestId },
    QueryFailed { request_id: RequestId, message: String },
}

/// Small typed boundary between the immediate-mode UI and asynchronous work.
/// The receiver is drained by the UI thread once per frame; the native binary
/// adapts these standard channels to the tokio runtime worker.
pub struct TaskBridge {
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<UiEvent>,
    next_request_id: u64,
}

impl Default for TaskBridge {
    fn default() -> Self {
        let (bridge, _command_rx, _event_tx) = Self::with_channels();
        bridge
    }
}

impl TaskBridge {
    /// Build the UI-side bridge and expose its endpoints to a runtime adapter.
    /// The adapter is responsible for translating these messages to its async
    /// channel implementation.
    pub fn with_channels() -> (Self, Receiver<UiCommand>, Sender<UiEvent>) {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        (
            Self {
                command_tx,
                event_rx,
                next_request_id: 1,
            },
            command_rx,
            event_tx,
        )
    }

    pub fn new(command_tx: Sender<UiCommand>, event_rx: Receiver<UiEvent>) -> Self {
        Self {
            command_tx,
            event_rx,
            next_request_id: 1,
        }
    }

    pub fn next_request_id(&mut self) -> RequestId {
        let id = RequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        id
    }

    pub fn send(&self, command: UiCommand) -> Result<(), mpsc::SendError<UiCommand>> {
        self.command_tx.send(command)
    }

    pub fn drain_events(&self) -> impl Iterator<Item = UiEvent> + '_ {
        std::iter::from_fn(|| self.event_rx.try_recv().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_are_monotonic() {
        let mut bridge = TaskBridge::default();
        assert_eq!(bridge.next_request_id(), RequestId(1));
        assert_eq!(bridge.next_request_id(), RequestId(2));
    }

    #[test]
    fn bridge_delivers_typed_commands() {
        let (command_tx, command_rx) = mpsc::channel();
        let (_event_tx, event_rx) = mpsc::channel();
        let bridge = TaskBridge::new(command_tx, event_rx);
        let command = UiCommand::OpenQuery;
        bridge.send(command.clone()).expect("receiver is alive");
        assert_eq!(command_rx.recv().expect("command expected"), command);
    }
}
