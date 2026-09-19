use super::*;

/// Execution-policy state for the query workspace.
///
/// Editor content belongs to `QuerySessionState`/`QueryEditorState`; this
/// aggregate owns execution safety, transaction presentation and explain mode.
pub(crate) struct QueryExecutionPolicyState {
    pub(super) pending_explain_analyze: bool,
    pub(super) explain_analyze_confirmed: bool,
    pub(super) explain_show_raw_json: bool,
    pub(super) query_auto_commit: bool,
    pub(super) query_in_transaction: bool,
    pub(super) query_txn_pending: usize,
    pub(super) disconnect_txn_guard: bool,
    pub(super) query_txn_bar_open: bool,
    pub(super) pending_destructive_run: Option<events::PendingDestructiveRun>,
}

impl Default for QueryExecutionPolicyState {
    fn default() -> Self {
        Self {
            pending_explain_analyze: false,
            explain_analyze_confirmed: false,
            explain_show_raw_json: false,
            query_auto_commit: true,
            query_in_transaction: false,
            query_txn_pending: 0,
            disconnect_txn_guard: false,
            query_txn_bar_open: false,
            pending_destructive_run: None,
        }
    }
}

impl QueryExecutionPolicyState {
    pub(super) fn apply_transaction_action(
        &mut self,
        action: crate::components::TransactionAction,
        feedback: &mut FeedbackState,
    ) -> Option<&'static str> {
        match action {
            crate::components::TransactionAction::ToggleAutoCommit(value) => {
                if self.query_in_transaction && value {
                    feedback.set_runtime_message("Commit or rollback the open transaction before enabling auto-commit");
                    return None;
                }
                self.query_auto_commit = value;
                if value {
                    self.query_in_transaction = false;
                    self.query_txn_pending = 0;
                }
                None
            }
            crate::components::TransactionAction::Begin => {
                self.query_auto_commit = false;
                self.query_in_transaction = true;
                self.query_txn_pending = 0;
                Some("BEGIN")
            }
            crate::components::TransactionAction::Commit => {
                self.query_in_transaction = false;
                self.query_txn_pending = 0;
                Some("COMMIT")
            }
            crate::components::TransactionAction::Rollback => {
                self.query_in_transaction = false;
                self.query_txn_pending = 0;
                Some("ROLLBACK")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::FeedbackState;

    use super::QueryExecutionPolicyState;

    #[test]
    fn default_execution_state_is_safe_and_autocommit() {
        let state = QueryExecutionPolicyState::default();

        assert!(state.query_auto_commit);
        assert!(!state.query_in_transaction);
        assert_eq!(state.query_txn_pending, 0);
        assert!(state.pending_destructive_run.is_none());
        assert!(!state.pending_explain_analyze);
    }

    #[test]
    fn transaction_actions_return_sql_effects_and_update_policy_state() {
        let mut state = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();

        assert_eq!(
            state.apply_transaction_action(crate::components::TransactionAction::Begin, &mut feedback),
            Some("BEGIN")
        );
        assert!(!state.query_auto_commit);
        assert!(state.query_in_transaction);

        assert_eq!(
            state.apply_transaction_action(crate::components::TransactionAction::Commit, &mut feedback),
            Some("COMMIT")
        );
        assert!(!state.query_in_transaction);
        assert_eq!(state.query_txn_pending, 0);
    }

    #[test]
    fn transaction_cannot_enable_autocommit_while_open() {
        let mut state = QueryExecutionPolicyState {
            query_auto_commit: false,
            query_in_transaction: true,
            ..Default::default()
        };
        let mut feedback = FeedbackState::default();

        assert_eq!(
            state.apply_transaction_action(
                crate::components::TransactionAction::ToggleAutoCommit(true),
                &mut feedback,
            ),
            None
        );
        assert!(!state.query_auto_commit);
        assert!(feedback.runtime_message.contains("Commit or rollback"));
    }
}
