use super::*;
use std::collections::HashMap;

use db_pro_core::domain::transaction_control::{transaction_control, TransactionControl};

/// Which dispatch surface held the statement — confirmation routes the run back to
/// the matching channel (`send_query_run` vs `send_table_data_query_run`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingDestructiveRunOrigin {
    /// The query editor's Ctrl+Enter / run-all path.
    QueryEditor,
    /// The table-data tab's inline SQL bar.
    InlineQuery,
    /// An EXPLAIN ANALYZE whose inner statement is destructive: confirming must
    /// dispatch `ExplainQuery`, not a plain run.
    ExplainAnalyze,
}

/// A statement (or script) the classifier rates `Destructive`, held until the user
/// confirms the exact text the prompt displayed.
#[derive(Debug, Clone)]
pub(crate) struct PendingDestructiveRun {
    sql: String,
    origin: PendingDestructiveRunOrigin,
    /// Pre-bound typed arguments from the routine workbench (#252). `None` means
    /// the editor's own parameter discovery re-binds at confirm time; `Some`
    /// keeps the exact typed values the preview showed so a confirmed `CALL`
    /// executes the arguments the user saw, not whatever the form holds later.
    bound_params: Option<Vec<db_pro_core::domain::query::QueryParam>>,
    /// Connection/schema the dialog must show — resolved when the run is held, not
    /// when it is confirmed.
    target_label: String,
    metadata: PendingDestructiveRunMetadata,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingDestructiveRunMetadata {
    execution_range: (usize, usize),
    version: u64,
    all_statements: bool,
    /// Row cap applied before this held run (UAT-4); carried through confirmation
    /// so completion can disclose the applied limit in the result status.
    row_limit: Option<u64>,
}

impl PendingDestructiveRunMetadata {
    pub(crate) fn new(execution_range: (usize, usize), version: u64, all_statements: bool) -> Self {
        Self {
            execution_range,
            version,
            all_statements,
            row_limit: None,
        }
    }

    /// Marks the held run as carrying an injected row cap.
    pub(crate) fn with_row_limit(mut self, row_limit: Option<u64>) -> Self {
        self.row_limit = row_limit;
        self
    }
}

impl PendingDestructiveRun {
    pub(crate) fn new(
        sql: String,
        origin: PendingDestructiveRunOrigin,
        target_label: String,
        metadata: PendingDestructiveRunMetadata,
        bound_params: Option<Vec<db_pro_core::domain::query::QueryParam>>,
    ) -> Self {
        Self {
            sql,
            origin,
            bound_params,
            target_label,
            metadata,
        }
    }

    pub(crate) fn sql(&self) -> &str {
        &self.sql
    }

    /// Pre-bound typed arguments to carry through confirmation (#252).
    pub(crate) fn bound_params(&self) -> Option<&[db_pro_core::domain::query::QueryParam]> {
        self.bound_params.as_deref()
    }

    pub(crate) fn origin(&self) -> PendingDestructiveRunOrigin {
        self.origin
    }

    pub(crate) fn target_label(&self) -> &str {
        &self.target_label
    }

    pub(crate) fn execution_range(&self) -> (usize, usize) {
        self.metadata.execution_range
    }

    pub(crate) fn version(&self) -> u64 {
        self.metadata.version
    }

    pub(crate) fn all_statements(&self) -> bool {
        self.metadata.all_statements
    }

    /// Row cap injected before the hold — `None` for unbounded or held writes.
    pub(crate) fn row_limit(&self) -> Option<u64> {
        self.metadata.row_limit
    }
}

#[derive(Clone, Copy)]
struct DocumentTransaction {
    auto_commit: bool,
    in_transaction: bool,
    pending: usize,
}

impl Default for DocumentTransaction {
    fn default() -> Self {
        Self {
            auto_commit: true,
            in_transaction: false,
            pending: 0,
        }
    }
}

/// Execution-policy state for the query workspace.
///
/// Editor content belongs to `QuerySessionState`/`QueryEditorState`; this
/// aggregate owns execution safety, transaction presentation and explain mode.
/// Transaction flags follow the query tab. Two tabs on one connection do not
/// share an open transaction.
pub(crate) struct QueryExecutionPolicyState {
    pub(super) pending_explain_analyze: bool,
    pub(super) explain_analyze_confirmed: bool,
    pub(super) explain_show_raw_json: bool,
    documents: HashMap<String, DocumentTransaction>,
    /// BEGIN/COMMIT/ROLLBACK requests whose server result has not arrived.
    /// The document id is the tab that sent the statement, not whichever tab is active now.
    pending_transactions: HashMap<crate::RequestId, (String, TransactionControl)>,
    pub(super) disconnect_txn_guard: bool,
    pub(super) query_txn_bar_open: bool,
    /// Row cap applied to dispatched reads (`None` = no limit) — toolbar picker.
    pub(super) query_row_limit: Option<u64>,
    pending_destructive_run: Option<PendingDestructiveRun>,
}

impl Default for QueryExecutionPolicyState {
    fn default() -> Self {
        Self {
            pending_explain_analyze: false,
            explain_analyze_confirmed: false,
            explain_show_raw_json: false,
            documents: HashMap::new(),
            pending_transactions: HashMap::new(),
            disconnect_txn_guard: false,
            query_txn_bar_open: false,
            query_row_limit: Some(500),
            pending_destructive_run: None,
        }
    }
}

impl QueryExecutionPolicyState {
    #[cfg(test)]
    pub(crate) fn confirm_explain_analyze(&mut self) {
        self.explain_analyze_confirmed = true;
    }

    pub(crate) fn pending_destructive_run(&self) -> Option<&PendingDestructiveRun> {
        self.pending_destructive_run.as_ref()
    }

    pub(crate) fn set_pending_destructive_run(&mut self, pending: PendingDestructiveRun) {
        self.pending_destructive_run = Some(pending);
    }

    pub(crate) fn take_pending_destructive_run(&mut self) -> Option<PendingDestructiveRun> {
        self.pending_destructive_run.take()
    }

    pub(super) fn in_transaction(&self, document_id: Option<&str>) -> bool {
        document_id
            .and_then(|id| self.documents.get(id))
            .is_some_and(|document| document.in_transaction)
    }

    pub(super) fn txn_pending(&self, document_id: Option<&str>) -> usize {
        document_id
            .and_then(|id| self.documents.get(id))
            .map(|document| document.pending)
            .unwrap_or(0)
    }

    pub(super) fn auto_commit(&self, document_id: Option<&str>) -> bool {
        document_id
            .and_then(|id| self.documents.get(id))
            .is_none_or(|document| document.auto_commit)
    }

    pub(super) fn any_transaction_busy(&self) -> bool {
        self.documents.values().any(|document| document.in_transaction) || !self.pending_transactions.is_empty()
    }

    pub(super) fn document_is_busy(&self, document_id: &str) -> bool {
        self.in_transaction(Some(document_id))
            || self
                .pending_transactions
                .values()
                .any(|(id, _)| id == document_id)
    }

    /// ID of the document holding the open transaction, if any. Server-side a
    /// transaction is per-connection, so a second tab on the same connection
    /// silently shares it — callers must gate cross-tab runs (#423).
    pub(super) fn transaction_owner(&self) -> Option<&str> {
        self.documents
            .iter()
            .find(|(_, document)| document.in_transaction)
            .map(|(id, _)| id.as_str())
    }

    pub(super) fn forget_document(&mut self, document_id: &str) {
        self.documents.remove(document_id);
    }

    fn document_mut(&mut self, document_id: &str) -> &mut DocumentTransaction {
        self.documents.entry(document_id.to_owned()).or_default()
    }

    pub(super) fn apply_transaction_action(
        &mut self,
        document_id: &str,
        action: crate::components::TransactionAction,
        feedback: &mut FeedbackState,
    ) -> Option<&'static str> {
        match action {
            crate::components::TransactionAction::ToggleAutoCommit(value) => {
                let document = self.document_mut(document_id);
                if document.in_transaction && value {
                    feedback.set_runtime_message("Commit or rollback the open transaction before enabling auto-commit");
                    return None;
                }
                document.auto_commit = value;
                if value {
                    document.in_transaction = false;
                    document.pending = 0;
                }
                None
            }
            crate::components::TransactionAction::Begin => Some("BEGIN"),
            crate::components::TransactionAction::Commit => Some("COMMIT"),
            crate::components::TransactionAction::Rollback => Some("ROLLBACK"),
        }
    }

    /// Record a dispatched statement for one query tab. Control statements wait
    /// for the server before that tab's flag changes. Other statements count
    /// toward the pending total only while that tab's transaction is already open.
    pub(super) fn note_transaction_dispatch(&mut self, document_id: &str, request_id: crate::RequestId, sql: &str) {
        if let Some(kind) = transaction_control(sql) {
            self.pending_transactions
                .insert(request_id, (document_id.to_owned(), kind));
            return;
        }
        if self.in_transaction(Some(document_id)) {
            let document = self.document_mut(document_id);
            document.pending = document.pending.saturating_add(1);
        }
    }

    pub(super) fn settle_transaction_request(&mut self, request_id: crate::RequestId, success: bool) {
        let Some((document_id, kind)) = self.pending_transactions.remove(&request_id) else {
            return;
        };
        if !success {
            return;
        }
        let document = self.document_mut(&document_id);
        match kind {
            TransactionControl::Begin => {
                document.auto_commit = false;
                document.in_transaction = true;
                document.pending = 0;
            }
            TransactionControl::Commit | TransactionControl::Rollback => {
                document.in_transaction = false;
                document.pending = 0;
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/query_execution_state_tests.rs"]
mod tests;

