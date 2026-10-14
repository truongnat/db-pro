//! Tests for `../query_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_query_session_starts_without_documents_or_pending_requests() {
        let state = QuerySessionState::default();

        assert!(state.documents.is_empty());
        assert_eq!(state.active_document_index, 0);
        assert!(state.document_requests.is_empty());
        assert!(state.save_requests.is_empty());
        assert!(state.pending_dirty_close.is_none());
        assert!(state.pending_close_after_save.is_none());
        assert!(!state.save_as_open);
    }

    #[test]
    fn document_collection_operations_keep_active_index_valid() {
        let mut state = QuerySessionState::default();
        let first = QueryDocument::new("query-1", "Query 1", "");
        let second = QueryDocument::new("query-2", "Query 2", "");
        let third = QueryDocument::new("query-3", "Query 3", "");

        assert_eq!(state.add_document(first), 0);
        assert_eq!(state.add_document(second), 1);
        assert_eq!(state.add_document(third), 2);
        assert!(state.select_document(1));

        assert_eq!(state.remove_document(0).expect("first document").id, "query-1");
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-2"));
        assert!(state.close_documents_to_right(0));
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-2"));
    }

    #[test]
    fn keep_document_replaces_collection_and_selects_it() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", ""));
        state.add_document(QueryDocument::new("query-2", "Query 2", ""));

        assert!(state.keep_document(0));
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-1"));
        assert!(!state.keep_document(4));
    }

    #[test]
    fn active_document_projections_are_owned_by_query_session_state() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", "SELECT 1"));

        assert_eq!(state.active_text(), "SELECT 1");
        assert!(state.append_active_text("SELECT 2"));
        assert_eq!(state.active_text(), "SELECT 1\n\nSELECT 2");
        assert_eq!(state.active_result_count(), 0);
        assert!(state.active_result().is_none());
        assert!(state.active_messages().is_empty());
        assert!(!state.set_active_result(0));
    }

    #[test]
    fn document_metadata_and_prediction_invalidation_are_state_owned() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", ""));

        assert!(state.set_document_connection(0, Some("conn-1".to_owned())));
        assert_eq!(state.active_connection_id(), Some("conn-1"));
        assert!(state.set_document_schema(0, Some("analytics".to_owned())));
        assert_eq!(state.active_schema(), Some("analytics"));
        assert!(!state.set_document_schema(9, None));
        assert!(state.invalidate_prediction(0).is_none());
    }

    #[test]
    fn prediction_request_commit_is_idempotent_for_a_document() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", "SELECT 1"));

        assert!(state.commit_prediction_request("query-1", RequestId(7), 42));
        assert_eq!(state.documents[0].pending_prediction_request, Some(RequestId(7)));
        assert_eq!(state.documents[0].prediction_context_fingerprint, Some(42));
        assert_eq!(state.documents[0].prediction_requests_sent, 1);
        assert!(!state.commit_prediction_request("query-1", RequestId(8), 43));
        assert_eq!(state.documents[0].pending_prediction_request, Some(RequestId(7)));
        assert_eq!(state.documents[0].prediction_requests_sent, 1);
    }