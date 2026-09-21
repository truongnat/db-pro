//! State transitions and command preparation for query execution.

use super::query_execution_state::{PendingDestructiveRun, PendingDestructiveRunMetadata};
use super::{
    FeedbackState, QueryEditorState, QueryExecutionPolicyState, QueryExecutionState, QuerySessionState, RequestId,
    UiCommand,
};
use std::time::Instant;

/// Coordinates query execution policy with the query document/session state.
///
/// This context deliberately stops before the runtime boundary: it prepares a
/// typed command and applies the local execution transition, while the app shell
/// remains responsible for sending that command through `TaskBridge`.
pub(crate) struct QueryExecutionContext<'a> {
    session: &'a mut QuerySessionState,
    editor: &'a mut QueryEditorState,
    policy: &'a mut QueryExecutionPolicyState,
    feedback: &'a mut FeedbackState,
    driver: String,
}

impl<'a> QueryExecutionContext<'a> {
    pub(crate) fn new(
        session: &'a mut QuerySessionState,
        editor: &'a mut QueryEditorState,
        policy: &'a mut QueryExecutionPolicyState,
        feedback: &'a mut FeedbackState,
        driver: impl Into<String>,
    ) -> Self {
        Self {
            session,
            editor,
            policy,
            feedback,
            driver: driver.into(),
        }
    }

    pub(crate) fn hold_destructive_run(
        &mut self,
        sql: &str,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) -> bool {
        if db_pro_core::domain::safety::classify_script_safety(sql)
            != Some(db_pro_core::domain::safety::StatementSafety::Destructive)
        {
            return false;
        }

        self.policy.set_pending_destructive_run(PendingDestructiveRun::new(
            sql.to_owned(),
            PendingDestructiveRunMetadata::new(execution_range, version, all_statements),
        ));
        self.feedback
            .set_runtime_message("Destructive statement held for confirmation — nothing was sent to the database");
        true
    }

    pub(crate) fn take_pending_destructive_run(&mut self) -> Option<PendingDestructiveRun> {
        self.policy.take_pending_destructive_run()
    }

    pub(crate) fn cancel_pending_destructive_run(&mut self) {
        if self.policy.take_pending_destructive_run().is_some() {
            self.feedback
                .set_runtime_message("Destructive statement cancelled — nothing was sent to the database");
        }
    }

    pub(crate) fn prepare_query_run(
        &mut self,
        request_id: RequestId,
        connection_id: String,
        sql: String,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) -> Option<UiCommand> {
        let discovered = crate::query::discover_sql_parameters(&sql);
        if all_statements && !discovered.is_empty() {
            self.feedback.set_runtime_message(
                "Parameterized scripts are not supported yet — run a single statement with bindings",
            );
            return None;
        }

        let style = if self.driver.eq_ignore_ascii_case("postgresql") || self.driver.eq_ignore_ascii_case("postgres") {
            crate::query::PlaceholderStyle::NumberedDollar
        } else {
            crate::query::PlaceholderStyle::QuestionMark
        };
        let values = self
            .session
            .active_document()
            .map(|document| document.parameter_values.clone())
            .unwrap_or_default();
        let (sql, params) = if discovered.is_empty() {
            (sql, Vec::new())
        } else {
            match crate::query::prepare_bound_sql(&sql, &values, style) {
                Ok(prepared) => (prepared.sql, prepared.values),
                Err(missing) => {
                    self.feedback
                        .set_runtime_message(format!("Fill parameter {missing} before running"));
                    return None;
                }
            }
        };

        self.record_query_history(&sql);
        self.mark_document_running(request_id, &sql, execution_range, version);
        self.feedback.set_runtime_message(if all_statements {
            "Sending full script to runtime…"
        } else {
            "Sending query to runtime…"
        });

        Some(if all_statements {
            UiCommand::RunQueryMulti {
                request_id,
                connection_id,
                sql,
            }
        } else {
            UiCommand::RunQuery {
                request_id,
                connection_id,
                sql,
                params,
            }
        })
    }

    fn record_query_history(&mut self, sql: &str) {
        if self.editor.query_history.iter().any(|query| query == sql) {
            return;
        }
        self.editor.query_history.push(sql.to_owned());
        if self.editor.query_history.len() > 20 {
            self.editor.query_history.remove(0);
        }
    }

    fn mark_document_running(
        &mut self,
        request_id: RequestId,
        sql: &str,
        execution_range: (usize, usize),
        version: u64,
    ) {
        let Some(document) = self.session.active_document_mut() else {
            return;
        };
        document.execution_state = QueryExecutionState::Running(request_id);
        document.execution_started_at = Some(Instant::now());
        document.execution_started_wall_time = Some(chrono::Utc::now().to_rfc3339());
        document.executing_range = Some(execution_range);
        document.executing_sql = Some(sql.to_owned());
        document.executing_version = Some(version);
        document.last_executed_range = Some(execution_range);
        document.execution_diagnostic = None;
        let document_id = document.id.clone();
        self.session.document_requests.insert(request_id, document_id);
    }
}

#[cfg(test)]
mod tests {
    use super::QueryExecutionPolicyState;
    use super::*;
    use crate::query::QueryDocument;

    fn context<'a>(
        session: &'a mut QuerySessionState,
        editor: &'a mut QueryEditorState,
        policy: &'a mut QueryExecutionPolicyState,
        feedback: &'a mut FeedbackState,
        driver: &str,
    ) -> QueryExecutionContext<'a> {
        QueryExecutionContext::new(session, editor, policy, feedback, driver)
    }

    #[test]
    fn destructive_sql_is_held_without_changing_document_execution_state() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "DROP TABLE users"));
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        assert!(context.hold_destructive_run("DROP TABLE users", (0, 16), 0, false));
        assert!(context.take_pending_destructive_run().is_some());
        assert!(session.active_running_request().is_none());
        assert!(feedback.runtime_message.contains("held for confirmation"));
    }

    #[test]
    fn preparing_a_parameterized_query_binds_values_and_tracks_document_request() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT * FROM users WHERE id = :id");
        document.parameter_values.insert(":id".to_owned(), "7".to_owned());
        session.add_document(document);
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        let command = context
            .prepare_query_run(
                RequestId(9),
                "conn-1".to_owned(),
                "SELECT * FROM users WHERE id = :id".to_owned(),
                (0, 36),
                2,
                false,
            )
            .expect("query should be prepared");

        assert_eq!(
            command,
            UiCommand::RunQuery {
                request_id: RequestId(9),
                connection_id: "conn-1".to_owned(),
                sql: "SELECT * FROM users WHERE id = $1".to_owned(),
                params: vec!["7".to_owned()],
            }
        );
        assert_eq!(session.active_running_request(), Some(RequestId(9)));
        assert_eq!(
            session.document_requests.get(&RequestId(9)).map(String::as_str),
            Some("doc-1")
        );
        assert_eq!(editor.query_history, vec!["SELECT * FROM users WHERE id = $1"]);
        assert!(feedback.runtime_message.contains("Sending query"));
    }

    #[test]
    fn parameterized_scripts_are_rejected_before_document_state_changes() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT :id"));
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        assert!(context
            .prepare_query_run(
                RequestId(9),
                "conn-1".to_owned(),
                "SELECT :id".to_owned(),
                (0, 10),
                1,
                true
            )
            .is_none());
        assert!(session.active_running_request().is_none());
        assert!(feedback.runtime_message.contains("Parameterized scripts"));
    }
}
