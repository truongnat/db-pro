use crate::components::overlay::{ToastManager, ToastPosition};

/// User-visible feedback shared by feature slices and the shell status surfaces.
#[derive(Debug, Default)]
pub(crate) struct FeedbackState {
    pub(super) runtime_message: String,
    pub(super) copy_status: String,
    pub(super) toasts: ToastManager,
}

impl FeedbackState {
    pub(crate) fn set_runtime_message(&mut self, message: impl Into<String>) {
        self.runtime_message = message.into();
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
mod tests {
    use super::FeedbackState;

    #[test]
    fn default_feedback_is_quiet() {
        let state = FeedbackState::default();

        assert!(state.runtime_message.is_empty());
        assert!(state.copy_status.is_empty());
        assert!(state.toasts.is_empty());
    }
}
