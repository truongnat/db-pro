//! The only UI-side adapter allowed to reach the runtime command bridge.
//!
//! Views may request a command or cancellation through this narrow port, but
//! they cannot depend on `TaskBridge` or manufacture request identities on
//! their own. The composition root still decides when to create the port.

use super::{FeedbackState, RequestId, TaskBridge, UiCommand};

pub(crate) struct RuntimeCommandDispatcher<'a> {
    bridge: &'a mut TaskBridge,
}

impl<'a> RuntimeCommandDispatcher<'a> {
    pub(crate) fn new(bridge: &'a mut TaskBridge) -> Self {
        Self { bridge }
    }

    pub(crate) fn next_request_id(&mut self) -> RequestId {
        self.bridge.next_request_id()
    }

    pub(crate) fn send_best_effort(&mut self, command: UiCommand) -> bool {
        self.bridge.send_best_effort(command)
    }

    pub(crate) fn dispatch(&mut self, command: UiCommand, feedback: &mut FeedbackState) -> bool {
        if self.send_best_effort(command) {
            return true;
        }
        let message = "Runtime worker unavailable";
        feedback.set_runtime_message(message);
        feedback.show_error_toast(message);
        false
    }
}
