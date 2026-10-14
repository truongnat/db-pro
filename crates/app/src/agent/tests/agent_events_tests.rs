//! Tests for `../agent_events.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn configuration_failure_is_scoped_to_the_pending_request() {
        let mut agent = AgentState {
            configure_request: Some(RequestId(4)),
            ..Default::default()
        };
        let mut feedback = FeedbackState::default();

        assert!(!handle_agent_request_failure(
            &mut agent,
            &mut feedback,
            RequestId(5),
            "stale"
        ));
        assert_eq!(agent.configure_request, Some(RequestId(4)));
        assert!(handle_agent_request_failure(
            &mut agent,
            &mut feedback,
            RequestId(4),
            "offline"
        ));
        assert!(agent.configure_request.is_none());
        assert_eq!(feedback.runtime_message, "Agent key operation failed · offline");
        assert_eq!(feedback.toasts.len(), 1);
    }

    #[test]
    fn provider_ready_updates_only_agent_provider_state() {
        let mut agent = AgentState::default();

        on_agent_provider_ready(&mut agent, "OpenAI".to_owned(), "configured".to_owned());

        assert_eq!(agent.provider_label, "OpenAI");
        assert_eq!(agent.provider_detail, "configured");
    }