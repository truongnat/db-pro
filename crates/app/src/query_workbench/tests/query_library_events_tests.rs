//! Tests for `../query_library_events.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn saved_query_load_replaces_the_read_model() {
        let mut state = QueryLibraryState {
            saved_queries: vec![UiSavedQuerySummary {
                id: "old".to_owned(),
                name: "Old query".to_owned(),
                sql: "select 1".to_owned(),
                folder: None,
            }],
            ..Default::default()
        };

        on_saved_queries_loaded(
            &mut state,
            vec![UiSavedQuerySummary {
                id: "new".to_owned(),
                name: "New query".to_owned(),
                sql: "select 2".to_owned(),
                folder: None,
            }],
        );

        assert_eq!(state.saved_queries[0].id, "new");
    }

    #[test]
    fn folder_load_replaces_the_read_model() {
        let mut state = QueryLibraryState::default();

        on_query_folders_loaded(
            &mut state,
            vec![UiQueryFolderSummary {
                id: "reports".to_owned(),
                name: "Reports".to_owned(),
            }],
        );

        assert_eq!(state.query_folders[0].name, "Reports");
    }