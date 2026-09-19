use super::*;

/// Execution-policy state for the query workspace.
///
/// Editor content belongs to `QuerySessionState`/`QueryEditorState`; this
/// aggregate owns execution safety, transaction presentation and explain mode.
pub(crate) struct QueryExecutionPolicyState {
    pub(crate) pending_explain_analyze: bool,
    pub(crate) explain_analyze_confirmed: bool,
    pub(crate) explain_show_raw_json: bool,
    pub(crate) query_auto_commit: bool,
    pub(crate) query_in_transaction: bool,
    pub(crate) query_txn_pending: usize,
    pub(crate) disconnect_txn_guard: bool,
    pub(crate) query_txn_bar_open: bool,
    pub(crate) pending_destructive_run: Option<events::PendingDestructiveRun>,
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

#[cfg(test)]
mod tests {
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
}
