//! Tests for `../result_grid_edit.rs` (moved out of the source file).
    use super::result_grid_cell_editor_surface_view::CellEditorKind;
    use super::*;
    use crate::{UiColumn, UiTableColumn, UiTableInfo};

    fn result(columns: &[(&str, &str, bool)], rows: Vec<Vec<UiCell>>) -> UiQueryResult {
        UiQueryResult {
            columns: columns
                .iter()
                .map(|(name, data_type, nullable)| UiColumn {
                    name: name.to_string(),
                    data_type: data_type.to_string(),
                    nullable: *nullable,
                })
                .collect(),
            row_count: rows.len() as u64,
            rows,
            duration_ms: 0,
        }
    }

    #[test]
    fn boolean_columns_use_the_select_editor() {
        let app = DbProApp::default();
        let result = result(&[("active", "boolean", true)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Boolean { nullable: true }
        );
    }

    #[test]
    fn enum_columns_use_labels_from_table_metadata() {
        let mut app = DbProApp::default();
        app.table.state.table_info = Some(UiTableInfo {
            schema: "main".to_owned(),
            name: "customers".to_owned(),
            row_count: None,
            columns: vec![UiTableColumn {
                name: "status".to_owned(),
                data_type: "order_status".to_owned(),
                nullable: true,
                enum_labels: vec!["pending".to_owned(), "shipped".to_owned()],
                ..Default::default()
            }],
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        });
        let result = result(&[("status", "order_status", true)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Select(vec!["pending".to_owned(), "shipped".to_owned(), "NULL".to_owned()])
        );
    }

    #[test]
    fn enum_named_types_fall_back_to_distinct_result_values() {
        let app = DbProApp::default();
        let result = result(
            &[("status", "enum", false)],
            vec![
                vec![UiCell::Text("open".to_owned())],
                vec![UiCell::Text("closed".to_owned())],
                vec![UiCell::Text("open".to_owned())],
                vec![UiCell::Null],
            ],
        );
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Select(vec!["closed".to_owned(), "open".to_owned()])
        );
    }

    #[test]
    fn timestamp_columns_use_the_temporal_editor() {
        let app = DbProApp::default();
        let result = result(&[("created_at", "timestamp without time zone", false)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Temporal { date_only: false }
        );
    }

    #[test]
    fn date_columns_use_a_date_only_temporal_editor() {
        let app = DbProApp::default();
        let result = result(&[("due_on", "date", false)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Temporal { date_only: true }
        );
    }

    #[test]
    fn text_and_number_columns_keep_the_text_input() {
        let app = DbProApp::default();
        let result = result(&[("name", "text", false), ("total", "numeric", false)], Vec::new());
        assert_eq!(app.classify_cell_editor(&result, 0), CellEditorKind::Text);
        assert_eq!(app.classify_cell_editor(&result, 1), CellEditorKind::Text);
    }