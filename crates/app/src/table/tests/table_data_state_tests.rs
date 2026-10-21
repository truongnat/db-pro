//! Tests for `../table_data_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_table_data_state_has_no_selection_or_pending_edit() {
        let state = TableDataState::default();

        assert!(state.grid_filter.is_empty());
        assert!(state.selected_cell.is_none());
        assert!(state.selected_rows.is_empty());
    }

    #[test]
    fn layout_scope_requires_connection_and_table_identity() {
        assert_eq!(
            TableDataState::layout_scope(Some("connection"), "public", Some("users")),
            Some("connection|public|users".to_owned())
        );
        assert_eq!(TableDataState::layout_scope(None, "public", Some("users")), None);
        assert_eq!(TableDataState::layout_scope(Some("connection"), "public", None), None);
    }

    /// Switching tables used to keep the previous grid's client sort/filter,
    /// so the new table rendered "sorted by" a column it might not even have.
    #[test]
    fn table_switch_clears_client_side_view_state() {
        let mut state = TableDataState {
            grid_filter: "active".to_owned(),
            grid_sort_column: Some(3),
            grid_sort_desc: true,
            manual_row_identity: Some(vec!["email".to_owned()]),
            ..Default::default()
        };

        state.reset_for_table_switch();

        assert!(state.grid_filter.is_empty());
        assert_eq!(state.grid_sort_column, None);
        assert!(!state.grid_sort_desc);
        assert_eq!(state.manual_row_identity, None);
    }

    #[test]
    fn layout_persists_and_restores_named_columns() {
        let mut state = TableDataState {
            grid_layout_column_names: vec!["id".to_owned(), "email".to_owned()],
            grid_column_order: vec![1, 0],
            grid_column_widths: vec![100.0, 240.0],
            ..Default::default()
        };
        state.grid_hidden_columns.insert(0);

        state.persist_layout(Some("connection|public|users".to_owned()));

        state.grid_layout_column_names.clear();
        state.grid_column_order.clear();
        state.grid_column_widths.clear();
        state.grid_hidden_columns.clear();
        state.restore_layout(Some("connection|public|users".to_owned()));

        let pending = state.grid_pending_named_layout.as_ref().expect("named layout");
        assert_eq!(pending[0].column_name, "email");
        assert_eq!(pending[0].width, 240.0);
        assert!(!pending[0].hidden);
        assert_eq!(pending[1].column_name, "id");
        assert_eq!(pending[1].width, 100.0);
        assert!(pending[1].hidden);
        assert!(state.grid_column_order.is_empty());
        assert!(state.grid_column_widths.is_empty());
        assert!(state.grid_hidden_columns.is_empty());
        assert!(!state.grid_legacy_layout_pending);
    }

    #[test]
    fn restoring_unknown_layout_scope_clears_grid_projection() {
        let mut state = TableDataState {
            grid_column_order: vec![1, 0],
            grid_column_widths: vec![120.0, 180.0],
            ..Default::default()
        };
        state.grid_hidden_columns.insert(1);
        state.grid_pending_named_layout = Some(Vec::new());
        state.grid_legacy_layout_pending = true;
        state.grid_columns_user_resized = true;

        state.restore_layout(Some("missing".to_owned()));

        assert!(state.grid_column_order.is_empty());
        assert!(state.grid_column_widths.is_empty());
        assert!(state.grid_hidden_columns.is_empty());
        assert!(state.grid_pending_named_layout.is_none());
        assert!(!state.grid_legacy_layout_pending);
        assert!(!state.grid_columns_user_resized);
    }

    #[test]
    fn grid_cache_invalidation_is_owned_by_table_data_state() {
        let mut state = TableDataState::default();
        state.grid_row_identity_cache.insert(
            0,
            RowIdentity {
                original_pk_columns: Vec::new(),
                original_pk_values: Vec::new(),
            },
        );
        state.grid_row_identity_cache_ready = true;

        state.invalidate_grid_row_caches();

        assert!(state.grid_row_identity_cache.is_empty());
        assert!(!state.grid_row_identity_cache_ready);
        assert_eq!(state.grid_projection_epoch, 1);
    }

    #[test]
    fn navigation_target_follows_visual_row_and_column_order() {
        let mut state = TableDataState::default();
        let indexes = [4, 1, 7];
        let order = [2, 0, 1];
        let lookup = super::result_grid_view::GridSelectionLookup::new(&indexes, &order);
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, false),
            None
        );
        state.select_single_cell((1, 0));

        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, false),
            Some((1, 1))
        );
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, true),
            Some((1, 2))
        );
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::ArrowDown, false),
            Some((7, 0))
        );
    }

    #[test]
    fn projection_key_tracks_grid_state_and_result_shape() {
        let state = TableDataState {
            grid_projection_epoch: 7,
            grid_filter: "active".to_owned(),
            grid_sort_column: Some(2),
            grid_sort_desc: true,
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "status".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
            ],
            rows: Vec::new(),
            row_count: 12,
            duration_ms: 4,
        };

        assert_eq!(
            state.projection_key(&result),
            GridProjectionKey {
                epoch: 7,
                filter: "active".to_owned(),
                sort_column: Some(2),
                sort_desc: true,
                row_count: 12,
                column_count: 2,
            }
        );
    }