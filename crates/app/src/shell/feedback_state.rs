use crate::components::overlay::{ToastManager, ToastPosition};
use super::app_types::Activity;

/// User-visible feedback shared by feature slices and the shell status surfaces.
#[derive(Debug, Default)]
pub(crate) struct FeedbackState {
    pub(super) runtime_message: String,
    /// When set, the status bar shows `runtime_message` only on this activity.
    pub(super) runtime_message_activity: Option<Activity>,
    pub(super) copy_status: String,
    pub(super) toasts: ToastManager,
}

impl FeedbackState {
    pub(crate) fn set_runtime_message(&mut self, message: impl Into<String>) {
        self.runtime_message_activity = None;
        self.runtime_message = message.into();
    }

    pub(crate) fn set_activity_runtime_message(&mut self, activity: Activity, message: impl Into<String>) {
        self.runtime_message_activity = Some(activity);
        self.runtime_message = message.into();
    }

    pub(crate) fn runtime_message_for_display(&self, current: Activity) -> &str {
        match self.runtime_message_activity {
            Some(scope) if scope != current => "",
            _ => self.runtime_message.as_str(),
        }
    }

    pub(crate) fn show_error_toast(&mut self, message: impl Into<String>) {
        self.toasts.error(message, ToastPosition::BottomRight);
    }

    pub(crate) fn show_success_toast(&mut self, message: impl Into<String>) {
        self.toasts.success(message, ToastPosition::BottomRight);
    }

    pub(crate) fn show_info_toast(&mut self, message: impl Into<String>) {
        self.toasts.info(message, ToastPosition::BottomRight);
    }
}

#[cfg(test)]
#[path = "tests/feedback_state_tests.rs"]
mod tests;

