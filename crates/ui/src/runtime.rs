use std::sync::mpsc::{self, Receiver, Sender};

/// Stable identity for an async UI operation. Real backend tasks will reuse
/// this identity for cancellation and stale-result protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    OpenQuery,
    RunQuery { request_id: RequestId, sql: String },
    CancelQuery { request_id: RequestId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    QueryQueued { request_id: RequestId },
    QueryCompleted { request_id: RequestId, row_count: u64 },
    QueryFailed { request_id: RequestId, message: String },
}

/// Small typed boundary between the immediate-mode UI and asynchronous work.
/// The receiver is drained by the UI thread once per frame; workers will be
/// added when the backend facade is extracted from the Tauri adapter.
pub struct TaskBridge {
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<UiEvent>,
    next_request_id: u64,
}

impl Default for TaskBridge {
    fn default() -> Self {
        let (command_tx, _command_rx) = mpsc::channel();
        let (_event_tx, event_rx) = mpsc::channel();
        Self {
            command_tx,
            event_rx,
            next_request_id: 1,
        }
    }
}

impl TaskBridge {
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
