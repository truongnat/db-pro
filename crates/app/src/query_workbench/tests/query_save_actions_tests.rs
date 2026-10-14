//! Tests for `../query_save_actions.rs` (moved out of the source file).
    use super::*;
    use crate::query::QueryDocument;

    #[test]
    fn save_preparation_commits_document_request_after_dispatch() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Report", "SELECT 1"));
        let library = QueryLibraryState::default();
        let mut feedback = FeedbackState::default();
        let prepared = {
            let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);
            context
                .prepare_save(RequestId(7), 0, Some("conn-1".to_owned()))
                .expect("connection should prepare save")
        };

        assert!(matches!(
            prepared,
            PreparedQuerySave {
                request_id: RequestId(7),
                connection_id,
                name,
                sql,
                folder: None,
                ..
            } if connection_id == "conn-1" && name == "Report" && sql == "SELECT 1"
        ));
        assert!(session.save_requests.is_empty());
        assert!(!feedback.runtime_message.contains("Saving"));
        let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);
        context.commit_dispatched(RequestId(7), 0);
        assert_eq!(
            session.save_requests.get(&RequestId(7)).map(String::as_str),
            Some("doc-1")
        );
        assert!(feedback.runtime_message.contains("Saving"));
    }

    #[test]
    fn save_preparation_requires_a_connection_without_registering_request() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Report", "SELECT 1"));
        let library = QueryLibraryState::default();
        let mut feedback = FeedbackState::default();
        let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);

        assert!(context.prepare_save(RequestId(7), 0, None).is_none());
        assert!(session.save_requests.is_empty());
        assert!(feedback.runtime_message.contains("connection"));
    }