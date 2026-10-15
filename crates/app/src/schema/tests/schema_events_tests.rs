//! Tests for `../schema_events.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn stale_schema_results_do_not_mutate_state() {
        let mut schema_explorer = SchemaExplorerState {
            schema_request: Some(RequestId(7)),
            ..Default::default()
        };
        let mut table_state = TableState::default();
        let mut data_query = TableDataQueryState::default();
        let mut workspace = WorkspaceShellState::default();
        let mut palette = PaletteState::default();
        let mut feedback = FeedbackState::default();

        let mut context = SchemaLoadedContext {
            schema_explorer: &mut schema_explorer,
            table_state: &mut table_state,
            data_query: &mut data_query,
            workspace: &mut workspace,
            palette: &mut palette,
            feedback: &mut feedback,
            driver: "PostgreSQL",
        };
        let transition = on_schema_loaded(&mut context, RequestId(8), UiSchemaSummary::default());

        assert_eq!(transition, SchemaLoadedTransition::default());
        assert_eq!(schema_explorer.schema_request, Some(RequestId(7)));
        assert!(schema_explorer.schema.tables.is_empty());
    }

    #[test]
    fn missing_selected_table_clears_table_state_and_returns_to_welcome() {
        let mut schema_explorer = SchemaExplorerState {
            schema_request: Some(RequestId(7)),
            selected_table: Some("gone".to_owned()),
            ..Default::default()
        };
        let mut table_state = TableState {
            table_info: Some(UiTableInfo {
                schema: "public".to_owned(),
                name: "gone".to_owned(),
                row_count: None,
                columns: Vec::new(),
                primary_key: None,
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                check_constraints: Vec::new(),
                dependencies: Vec::new(),
            }),
            refresh_table_info_after_schema: true,
            ..Default::default()
        };
        let mut data_query = TableDataQueryState::default();
        let mut workspace = WorkspaceShellState {
            active_tab: WorkspaceTab::Table,
            ..Default::default()
        };
        let mut palette = PaletteState::default();
        let mut feedback = FeedbackState::default();

        let mut context = SchemaLoadedContext {
            schema_explorer: &mut schema_explorer,
            table_state: &mut table_state,
            data_query: &mut data_query,
            workspace: &mut workspace,
            palette: &mut palette,
            feedback: &mut feedback,
            driver: "PostgreSQL",
        };
        let transition = on_schema_loaded(&mut context, RequestId(7), UiSchemaSummary::default());

        assert!(!transition.refresh_selected_table);
        assert!(schema_explorer.selected_table.is_none());
        assert!(table_state.table_info.is_none());
        assert_eq!(workspace.active_tab, WorkspaceTab::Welcome);
    }