//! Tests for `../query_library_state.rs` (moved out of the source file).
    use super::QueryLibraryState;

    #[test]
    fn default_library_has_no_remote_items_or_draft_folder() {
        let state = QueryLibraryState::default();

        assert!(state.saved_queries.is_empty());
        assert!(state.query_folders.is_empty());
        assert!(state.query_folder.is_empty());
    }

    #[test]
    fn folder_draft_normalization_stays_in_library_state() {
        let state = QueryLibraryState::default();
        assert_eq!(state.normalized_folder(), None);
        assert_eq!(state.required_folder_name(), Err("Folder name is required".to_owned()));
    }

    #[test]
    fn folder_draft_trims_whitespace() {
        let state = QueryLibraryState {
            query_folder: " reports ".to_owned(),
            ..QueryLibraryState::default()
        };
        assert_eq!(state.normalized_folder().as_deref(), Some("reports"));
        assert_eq!(state.required_folder_name().unwrap(), "reports");
    }