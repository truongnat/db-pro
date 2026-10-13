//! Validation and state transitions for query explain requests.

use super::{
    CapabilityLookup, FeedbackState, OutputTab, QueryExecutionPolicyState, QueryOutputState, QuerySessionState,
    RequestId,
};

pub(crate) struct PreparedExplainRequest {
    pub(crate) connection_id: String,
    pub(crate) sql: String,
    pub(crate) params: Vec<String>,
    pub(crate) analyze: bool,
    /// True when this driver's EXPLAIN ANALYZE actually runs the statement.
    pub(crate) analyze_executes: bool,
    pub(crate) document_index: usize,
}

/// PostgreSQL `EXPLAIN ANALYZE` and MySQL `EXPLAIN ANALYZE` run the statement.
/// SQLite and SQL Server ignore the analyze flag.
pub(crate) fn explain_analyze_executes(driver: &str) -> bool {
    driver.eq_ignore_ascii_case("postgresql")
        || driver.eq_ignore_ascii_case("postgres")
        || driver.eq_ignore_ascii_case("mysql")
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
        driver: &str,
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
        let (sql, _) = self
            .session
            .active_document()
            .map(|document| document.resolve_executable_sql())
            .unwrap_or_default();
        let sql = sql.trim().to_owned();
        if sql.is_empty() {
            self.feedback.set_runtime_message("Enter a query before explaining it");
            return None;
        }
        let style = crate::query::placeholder_style_for_driver(driver);
        let values = self
            .session
            .active_document()
            .map(|document| document.parameter_values.clone())
            .unwrap_or_default();
        let prepared = match crate::query::prepare_bound_sql(&sql, &values, style) {
            Ok(prepared) => prepared,
            Err(missing) => {
                self.feedback
                    .set_runtime_message(format!("Fill parameter {missing} before running"));
                return None;
            }
        };
        let analyze_executes = explain_analyze_executes(driver);
        if analyze && analyze_executes && !self.execution.explain_analyze_confirmed {
            self.execution.pending_explain_analyze = true;
            self.activate_explain_output();
            self.feedback
                .set_runtime_message("EXPLAIN ANALYZE executes the statement — confirm in the Explain pane");
            return None;
        }

        Some(PreparedExplainRequest {
            connection_id,
            sql: prepared.sql,
            params: prepared.values,
            analyze,
            analyze_executes,
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
        self.feedback.set_runtime_message(if request.analyze && request.analyze_executes {
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
            .prepare(Some("conn-1".to_owned()), &capabilities, true, "PostgreSQL")
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
            .prepare(Some("conn-1".to_owned()), &capabilities, true, "PostgreSQL")
            .expect("confirmed analyze should prepare");

        context.commit_dispatched(RequestId(4), &request);

        assert_eq!(session.active_explain_request(), Some(RequestId(4)));
        assert!(!execution.explain_analyze_confirmed);
        assert!(!execution.pending_explain_analyze);
        assert!(feedback.runtime_message.contains("running"));
    }

    #[test]
    fn caret_after_a_semicolon_explains_that_statement() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT 1;\n\nSELECT 2;");
        document.cursor.set_offset(&document.buffer, 10);
        session.add_document(document);
        let mut execution = QueryExecutionPolicyState::default();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("PostgreSQL");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);

        let request = context
            .prepare(Some("conn-1".to_owned()), &capabilities, false, "PostgreSQL")
            .expect("caret statement");

        assert_eq!(request.sql, "SELECT 1;");
        assert!(request.params.is_empty());
    }

    #[test]
    fn selection_wins_over_the_caret_statement() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT 1;\n\nSELECT 2;");
        document.cursor.set_offset(&document.buffer, 10);
        document.selection = crate::editor::SelectionRange::new(11, 19);
        session.add_document(document);
        let mut execution = QueryExecutionPolicyState::default();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("SQLite");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);

        let request = context
            .prepare(Some("conn-1".to_owned()), &capabilities, false, "SQLite")
            .expect("selection");

        assert_eq!(request.sql, "SELECT 2");
    }

    #[test]
    fn sqlite_analyze_does_not_ask_for_execution_confirmation() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT 1"));
        let mut execution = QueryExecutionPolicyState::default();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("SQLite");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);

        let request = context
            .prepare(Some("conn-1".to_owned()), &capabilities, true, "SQLite")
            .expect("sqlite analyze does not execute");

        assert!(request.analyze);
        assert!(!request.analyze_executes);
        assert!(!execution.pending_explain_analyze);
    }

    #[test]
    fn missing_parameter_fails_closed() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT $1"));
        let mut execution = QueryExecutionPolicyState::default();
        let mut output = QueryOutputState::default();
        let mut feedback = FeedbackState::default();
        let capabilities = CapabilityLookup::for_driver_label("PostgreSQL");
        let mut context = context(&mut session, &mut execution, &mut output, &mut feedback);

        assert!(context
            .prepare(Some("conn-1".to_owned()), &capabilities, false, "PostgreSQL")
            .is_none());
        assert!(feedback.runtime_message.contains("Fill parameter"));
    }
}
