//! Validation and state transitions for query explain requests.

use super::{
    CapabilityLookup, FeedbackState, OutputTab, QueryExecutionPolicyState, QueryOutputState, QuerySessionState,
    RequestId,
};

pub(crate) struct PreparedExplainRequest {
    pub(crate) connection_id: String,
    pub(crate) sql: String,
    pub(crate) analyze: bool,
    pub(crate) document_index: usize,
}

pub(crate) struct QueryExplainContext<'a> {
    session: &'a mut QuerySessionState,
    execution: &'a mut QueryExecutionPolicyState,
    output: &'a mut QueryOutputState,
    feedback: &'a mut FeedbackState,
}

impl<'a> QueryExplainContext<'a> {
    pub(crate) fn new(
        session: &'a mut QuerySessionState,
        execution: &'a mut QueryExecutionPolicyState,
        output: &'a mut QueryOutputState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            session,
            execution,
            output,
            feedback,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        connection_id: Option<String>,
        capabilities: &CapabilityLookup,
        analyze: bool,
    ) -> Option<PreparedExplainRequest> {
        if self.session.active_explain_request().is_some() {
            return None;
        }
        if let Some(reason) =
            capabilities.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Explain)
        {
            self.feedback
                .set_runtime_message(format!("Explain is unavailable: {reason}"));
            return None;
        }
        let Some(connection_id) = connection_id else {
            self.feedback
                .set_runtime_message("Connect to a database before explaining a query");
            return None;
        };
        let sql = if self.session.selected_text.trim().is_empty() {
            self.session.active_text().trim().to_owned()
        } else {
            self.session.selected_text.trim().to_owned()
        };
        if sql.is_empty() {
            self.feedback.set_runtime_message("Enter a query before explaining it");
            return None;
        }
        if analyze && !self.execution.explain_analyze_confirmed {
            self.execution.pending_explain_analyze = true;
            self.activate_explain_output();
            self.feedback
                .set_runtime_message("EXPLAIN ANALYZE executes the statement — confirm in the Explain pane");
            return None;
        }

        Some(PreparedExplainRequest {
            connection_id,
            sql,
            analyze,
            document_index: self.session.active_document_index,
        })
    }

    pub(crate) fn commit_dispatched(&mut self, request_id: RequestId, request: &PreparedExplainRequest) {
        if let Some(document) = self.session.documents.get_mut(request.document_index) {
            document.explain_request = Some(request_id);
            document.explain_plan = None;
        }
        self.execution.pending_explain_analyze = false;
        self.execution.explain_analyze_confirmed = false;
        self.activate_explain_output();
        self.feedback.set_runtime_message(if request.analyze {
            "EXPLAIN ANALYZE running (query executes)…"
        } else {
            "Explaining query…"
        });
    }

    fn activate_explain_output(&mut self) {
        self.output.set_active_for_optional_document(
            self.session.active_document().map(|document| document.id.as_str()),
            OutputTab::Explain,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryDocument;

    fn context<'a>(
        session: &'a mut QuerySessionState,
        execution: &'a mut QueryExecutionPolicyState,
        output: &'a mut QueryOutputState,
        feedback: &'a mut FeedbackState,
    ) -> QueryExplainContext<'a> {
        QueryExplainContext::new(session, execution, output, feedback)
    }

    #[test]
    fn analyze_requires_confirmation_without_creating_a_runtime_request() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT 1"));
        let mut execution = QueryExecutionPolicyState::default();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("PostgreSQL");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);

        assert!(context
            .prepare(Some("conn-1".to_owned()), &capabilities, true)
            .is_none());
        assert!(execution.pending_explain_analyze);
        assert_eq!(output.active_tab, OutputTab::Explain);
        assert!(feedback.runtime_message.contains("confirm"));
    }

    #[test]
    fn dispatched_explain_clears_confirmation_and_tracks_request_on_document() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT 1"));
        let mut execution = QueryExecutionPolicyState::default();
        execution.confirm_explain_analyze();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("PostgreSQL");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);
        let request = context
            .prepare(Some("conn-1".to_owned()), &capabilities, true)
            .expect("confirmed analyze should prepare");

        context.commit_dispatched(RequestId(4), &request);

        assert_eq!(session.active_explain_request(), Some(RequestId(4)));
        assert!(!execution.explain_analyze_confirmed);
        assert!(!execution.pending_explain_analyze);
        assert!(feedback.runtime_message.contains("running"));
    }
}
