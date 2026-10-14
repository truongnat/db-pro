//! Tests for `../query_explain_actions.rs` (moved out of the source file).
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