//! Tests for `../query_execution_actions.rs` (moved out of the source file).
    use super::QueryExecutionPolicyState;
    use super::*;
    use crate::query::QueryDocument;
    use db_pro_core::domain::query::QueryParam;

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

        assert!(context.hold_destructive_run(
            "DROP TABLE users",
            PendingDestructiveRunOrigin::QueryEditor,
            "conn · public".to_owned(),
            PendingDestructiveRunMetadata::new((0, 16), 0, false),
            None
        ));
        let pending = context.take_pending_destructive_run().expect("held run");
        assert_eq!(pending.origin(), PendingDestructiveRunOrigin::QueryEditor);
        assert_eq!(pending.target_label(), "conn · public");
        assert!(session.active_running_request().is_none());
        assert!(feedback.runtime_message.contains("held for confirmation"));
    }

    #[test]
    fn a_second_destructive_run_never_overwrites_the_held_one() {
        let mut session = QuerySessionState::default();
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        assert!(context.hold_destructive_run(
            "DROP TABLE first",
            PendingDestructiveRunOrigin::QueryEditor,
            "conn".to_owned(),
            PendingDestructiveRunMetadata::new((0, 16), 0, false),
            None
        ));
        // A second destructive dispatch while the prompt is open must not replace it.
        assert!(context.hold_destructive_run(
            "DROP TABLE second",
            PendingDestructiveRunOrigin::InlineQuery,
            "conn".to_owned(),
            PendingDestructiveRunMetadata::new((0, 17), 0, false),
            None
        ));
        let pending = context.take_pending_destructive_run().expect("held run");
        assert_eq!(pending.sql(), "DROP TABLE first");
        assert_eq!(pending.origin(), PendingDestructiveRunOrigin::QueryEditor);
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

        let prepared = context
            .prepare_query_run(
                RequestId(9),
                "conn-1".to_owned(),
                "SELECT * FROM users WHERE id = :id".to_owned(),
                false,
            )
            .expect("query should be prepared");
        context.commit_dispatched(&prepared, (0, 36), 2);

        assert_eq!(
            prepared,
            PreparedQueryRun {
                request_id: RequestId(9),
                connection_id: "conn-1".to_owned(),
                sql: "SELECT * FROM users WHERE id = $1".to_owned(),
                params: vec![QueryParam::Text("7".to_owned())],
                all_statements: false,
                schema: None,
                session_id: None,
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
            .prepare_query_run(RequestId(9), "conn-1".to_owned(), "SELECT :id".to_owned(), true)
            .is_none());
        assert!(session.active_running_request().is_none());
        assert!(feedback.runtime_message.contains("Parameterized scripts"));
    }

    #[test]
    fn postgres_jsonb_question_mark_script_is_not_rejected_or_rewritten() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT data ? 'x'; SELECT 1"));
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        let prepared = context
            .prepare_query_run(
                RequestId(9),
                "conn-1".to_owned(),
                "SELECT data ? 'x'; SELECT 1".to_owned(),
                true,
            )
            .expect("jsonb ? is not a placeholder");
        assert_eq!(prepared.sql, "SELECT data ? 'x'; SELECT 1");
        assert!(prepared.params.is_empty());
        assert!(prepared.all_statements);
    }

    #[test]
    fn sql_server_rewrites_placeholders_to_at_params_in_order() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT :id, $1");
        document.parameter_values.insert(":id".to_owned(), "7".to_owned());
        document.parameter_values.insert("$1".to_owned(), "8".to_owned());
        session.add_document(document);
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "sql server");

        let prepared = context
            .prepare_query_run(RequestId(4), "conn-1".to_owned(), "SELECT :id, $1".to_owned(), false)
            .expect("sql server query should be prepared");
        assert_eq!(prepared.sql, "SELECT @p1, @p2");
        assert_eq!(
            prepared.params,
            vec![QueryParam::Text("7".to_owned()), QueryParam::Text("8".to_owned())]
        );
    }

    #[test]
    fn mysql_question_mark_still_binds() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT ?");
        document.parameter_values.insert("?1".to_owned(), "9".to_owned());
        session.add_document(document);
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "MySQL");

        let prepared = context
            .prepare_query_run(RequestId(5), "conn-1".to_owned(), "SELECT ?".to_owned(), false)
            .expect("mysql query should be prepared");
        assert_eq!(prepared.sql, "SELECT ?");
        assert_eq!(prepared.params, vec![QueryParam::Text("9".to_owned())]);
    }

    #[test]
    fn sqlite_question_mark_still_binds() {
        let mut session = QuerySessionState::default();
        let mut document = QueryDocument::new("doc-1", "Query 1", "SELECT ?");
        document.parameter_values.insert("?1".to_owned(), "9".to_owned());
        session.add_document(document);
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "SQLite");

        let prepared = context
            .prepare_query_run(RequestId(6), "conn-1".to_owned(), "SELECT ?".to_owned(), false)
            .expect("sqlite query should be prepared");
        assert_eq!(prepared.sql, "SELECT ?");
        assert_eq!(prepared.params, vec![QueryParam::Text("9".to_owned())]);
    }

    #[test]
    fn missing_parameter_value_fails_closed() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT :id"));
        let mut editor = QueryEditorState::default();
        let mut policy = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let mut context = context(&mut session, &mut editor, &mut policy, &mut feedback, "PostgreSQL");

        assert!(context
            .prepare_query_run(RequestId(9), "conn-1".to_owned(), "SELECT :id".to_owned(), false)
            .is_none());
        assert!(session.active_running_request().is_none());
        assert!(feedback.runtime_message.contains("Fill parameter :id"));
    }