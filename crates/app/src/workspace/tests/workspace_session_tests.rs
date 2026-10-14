//! Tests for `../workspace_session.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn session_round_trip_json_preserves_document_refs_only() {
        let mut session = WorkspaceSession::new_named("Focus");
        session.open_document_ids = vec!["doc-1".into(), "doc-2".into()];
        session.active_connection_id = Some("conn-1".into());
        let json = serde_json::to_string(&session).unwrap();
        assert!(!json.to_lowercase().contains("password"));
        assert!(!json.contains("SELECT"));
        let loaded: WorkspaceSession = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.open_document_ids, vec!["doc-1", "doc-2"]);
        assert_eq!(loaded.name, "Focus");
    }

    #[test]
    fn named_store_upsert_and_remove() {
        let mut store = NamedSessionStore::new();
        let session = WorkspaceSession::new_named("A");
        let id = session.id.clone();
        store.upsert(session);
        assert!(store.get(&id).is_some());
        assert!(store.remove(&id));
        assert!(store.get(&id).is_none());
    }