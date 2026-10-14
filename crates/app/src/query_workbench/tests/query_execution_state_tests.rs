//! Tests for `../query_execution_state.rs` (moved out of the source file).
    use crate::app::FeedbackState;

    use super::QueryExecutionPolicyState;

    #[test]
    fn default_execution_state_is_safe_and_autocommit() {
        let state = QueryExecutionPolicyState::default();

        assert!(state.auto_commit(Some("doc")));
        assert!(!state.in_transaction(Some("doc")));
        assert_eq!(state.txn_pending(Some("doc")), 0);
        assert!(state.pending_destructive_run().is_none());
        assert!(!state.pending_explain_analyze);
    }

    #[test]
    fn transaction_actions_return_sql_effects_and_update_policy_state() {
        let mut state = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();

        assert_eq!(
            state.apply_transaction_action("doc", crate::components::TransactionAction::Begin, &mut feedback),
            Some("BEGIN")
        );
        assert!(state.auto_commit(Some("doc")));
        assert!(!state.in_transaction(Some("doc")));

        state.note_transaction_dispatch("doc", crate::RequestId(1), "BEGIN");
        state.settle_transaction_request(crate::RequestId(1), true);
        assert!(!state.auto_commit(Some("doc")));
        assert!(state.in_transaction(Some("doc")));

        state.note_transaction_dispatch("doc", crate::RequestId(2), "SELECT 1");
        assert_eq!(state.txn_pending(Some("doc")), 1);

        assert_eq!(
            state.apply_transaction_action("doc", crate::components::TransactionAction::Commit, &mut feedback),
            Some("COMMIT")
        );
        assert!(state.in_transaction(Some("doc")));
        state.note_transaction_dispatch("doc", crate::RequestId(3), "COMMIT");
        state.settle_transaction_request(crate::RequestId(3), true);
        assert!(!state.in_transaction(Some("doc")));
        assert_eq!(state.txn_pending(Some("doc")), 0);
    }

    #[test]
    fn failed_begin_does_not_open_the_transaction() {
        let mut state = QueryExecutionPolicyState::default();
        state.note_transaction_dispatch("doc", crate::RequestId(4), "BEGIN");
        state.settle_transaction_request(crate::RequestId(4), false);
        assert!(!state.in_transaction(Some("doc")));
        assert!(state.auto_commit(Some("doc")));
    }

    #[test]
    fn transaction_cannot_enable_autocommit_while_open() {
        let mut state = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        state.note_transaction_dispatch("doc", crate::RequestId(1), "BEGIN");
        state.settle_transaction_request(crate::RequestId(1), true);

        assert_eq!(
            state.apply_transaction_action(
                "doc",
                crate::components::TransactionAction::ToggleAutoCommit(true),
                &mut feedback,
            ),
            None
        );
        assert!(!state.auto_commit(Some("doc")));
        assert!(feedback.runtime_message.contains("Commit or rollback"));
    }

    #[test]
    fn two_query_tabs_do_not_share_a_transaction() {
        let mut state = QueryExecutionPolicyState::default();
        state.note_transaction_dispatch("tab-a", crate::RequestId(1), "BEGIN");
        state.settle_transaction_request(crate::RequestId(1), true);
        state.note_transaction_dispatch("tab-a", crate::RequestId(2), "SELECT 1");

        assert!(state.in_transaction(Some("tab-a")));
        assert!(!state.in_transaction(Some("tab-b")));
        assert_eq!(state.txn_pending(Some("tab-a")), 1);
        assert_eq!(state.txn_pending(Some("tab-b")), 0);
        assert!(state.auto_commit(Some("tab-b")));
        assert!(state.any_transaction_busy());
        assert!(state.document_is_busy("tab-a"));
        assert!(!state.document_is_busy("tab-b"));
    }