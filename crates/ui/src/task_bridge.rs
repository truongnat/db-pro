//! Bounded UI/runtime channel adapter.

use super::{RequestId, UiCommand, UiEvent};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};

/// Small typed boundary between the immediate-mode UI and asynchronous work.
/// The receiver is drained by the UI thread in bounded batches; the native
/// binary adapts these standard channels to the tokio runtime worker.
pub struct TaskBridge {
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<UiEvent>,
    next_request_id: u64,
}

/// Maximum number of runtime events the UI reducer applies in one frame.
///
/// Keeping this bound at the UI boundary prevents a burst of backend results
/// from monopolising an egui frame. The caller requests another repaint when
/// the batch reaches this limit.
pub(crate) const MAX_RUNTIME_EVENTS_PER_FRAME: usize = 64;

/// Capacity of the bounded native adapter queue between the worker and egui.
pub(crate) const UI_EVENT_CHANNEL_CAPACITY: usize = 256;

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
    pub fn with_channels() -> (Self, Receiver<UiCommand>, SyncSender<UiEvent>) {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::sync_channel(UI_EVENT_CHANNEL_CAPACITY);
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

    pub fn send(&self, command: UiCommand) -> Result<(), Box<mpsc::SendError<UiCommand>>> {
        self.command_tx.send(command).map_err(Box::new)
    }

    pub fn send_best_effort(&self, command: UiCommand) -> bool {
        if self.send(command).is_ok() {
            return true;
        }
        tracing::warn!("runtime command channel closed before command dispatch");
        false
    }

    pub fn drain_events(&self, limit: usize) -> impl Iterator<Item = UiEvent> + '_ {
        std::iter::from_fn(|| self.event_rx.try_recv().ok()).take(limit)
    }
}
