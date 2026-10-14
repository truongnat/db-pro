//! Tests for `../connection_events.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn request_failure_classification_uses_typed_pending_operation() {
        let mut lifecycle = ConnectionLifecycleState::default();
        lifecycle.set_pending_request(Some(RequestId(7)));
        lifecycle.set_pending_operation(Some(PendingConnectionOperation::Delete));
        lifecycle.set_pending_connection_id(Some("conn-1".to_owned()));
        let mut dialog = ConnectionDialogState::default();
        let mut schema_explorer = SchemaExplorerState::default();
        let mut feedback = FeedbackState::default();
        feedback.set_runtime_message("Connecting to Local…");

        assert!(handle_connection_request_failure(
            &mut lifecycle,
            &mut dialog,
            &mut schema_explorer,
            &mut feedback,
            RequestId(7),
            "permission denied",
        ));

        assert_eq!(feedback.runtime_message, "Delete failed · permission denied");
        assert!(lifecycle.pending_operation().is_none());
    }