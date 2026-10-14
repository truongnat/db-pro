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
#[path = "tests/query_explain_actions_tests.rs"]
mod tests;

