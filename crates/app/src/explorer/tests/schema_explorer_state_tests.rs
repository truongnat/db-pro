//! Tests for `../schema_explorer_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_schema_explorer_state_has_no_selection_or_pending_request() {
        let state = SchemaExplorerState::default();

        assert!(state.schema.tables.is_empty());
        assert!(state.selected_schema.is_none());
        assert!(state.selected_table.is_none());
        assert!(state.schema_request.is_none());
        assert_eq!(state.schema_object_view, SchemaObjectView::Definition);
    }

    #[test]
    fn recent_tables_are_owned_as_a_bounded_mru_list() {
        let mut state = SchemaExplorerState::default();

        state.record_recent_table("users");
        state.record_recent_table("orders");
        state.record_recent_table("users");
        state.remove_recent_table("orders");

        assert_eq!(state.recent_tables, vec!["users"]);
    }

    #[test]
    fn schema_activation_resets_table_workspace_and_welcome_surface() {
        let mut explorer = SchemaExplorerState {
            selected_schema: Some("public".to_owned()),
            selected_table: Some("users".to_owned()),
            selected_schema_object: Some(SchemaObjectSelection::View("active_view".to_owned())),
            schema_object_view: SchemaObjectView::Data,
            explorer_nav_cache: Some(ExplorerNavCache {
                connection_id: "conn-1".to_owned(),
                schema: "public".to_owned(),
                search: String::new(),
                mode: ExplorerMatchMode::Contains,
                total_count: 1,
                matching_count: 1,
                visible: vec!["users".to_owned()],
            }),
            ..Default::default()
        };
        let mut table = super::TableEditorState::default();
        table.mutation.pending_changes_open = true;
        table.editing.data_edit_value = "draft".to_owned();
        let mut workspace = super::WorkspaceShellState {
            active_tab: super::WorkspaceTab::Table,
            ..Default::default()
        };
        let mut feedback = super::FeedbackState::default();

        SchemaActivationContext::new(&mut explorer, &mut table, &mut workspace, &mut feedback).activate("analytics");

        assert_eq!(explorer.selected_schema.as_deref(), Some("analytics"));
        assert!(explorer.selected_table.is_none());
        assert!(explorer.selected_schema_object.is_none());
        assert_eq!(explorer.schema_object_view, SchemaObjectView::Definition);
        assert!(explorer.explorer_nav_cache.is_none());
        assert!(!table.mutation.pending_changes_open);
        assert!(table.editing.data_edit_value.is_empty());
        assert_eq!(workspace.active_tab, super::WorkspaceTab::Welcome);
    }