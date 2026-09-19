use crate::components::overlay::ToastManager;

/// User-visible feedback shared by feature slices and the shell status surfaces.
#[derive(Debug, Default)]
pub(crate) struct FeedbackState {
    pub(super) runtime_message: String,
    pub(super) copy_status: String,
    pub(super) toasts: ToastManager,
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
