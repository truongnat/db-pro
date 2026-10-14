//! Tests for `../table_editor_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn test_generate_sample_value_types() {
        let uuid_val = table_editor_values::generate_sample_value("id", "uuid");
        assert!(uuid::Uuid::parse_str(&uuid_val).is_ok());

        let time_val = table_editor_values::generate_sample_value("created_at", "timestamptz");
        assert!(chrono::DateTime::parse_from_rfc3339(&time_val).is_ok());

        let date_val = table_editor_values::generate_sample_value("birth_date", "date");
        assert_eq!(date_val.len(), 10);

        let email_val = table_editor_values::generate_sample_value("user_email", "varchar");
        assert!(email_val.contains('@'));

        let bool_val = table_editor_values::generate_sample_value("is_active", "boolean");
        assert!(bool_val == "true" || bool_val == "false");
    }

    #[test]
    fn test_parse_insert_value_uuid() {
        let valid_uuid = "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11";
        let parsed = table_editor_values::parse_insert_value(valid_uuid, "uuid");
        assert_eq!(parsed, Ok(Some(UiCell::Text(valid_uuid.to_owned()))));

        let invalid_uuid = "not-a-uuid";
        assert!(table_editor_values::parse_insert_value(invalid_uuid, "uuid").is_err());
    }

    #[test]
    fn update_parser_preserves_null_empty_and_whitespace_text() {
        assert_eq!(
            table_editor_values::parse_update_value("NULL", "text"),
            Ok(UiCell::Null)
        );
        assert_eq!(
            table_editor_values::parse_update_value("", "text"),
            Ok(UiCell::Text(String::new()))
        );
        assert_eq!(
            table_editor_values::parse_update_value("   ", "text"),
            Ok(UiCell::Text("   ".to_owned()))
        );
        assert!(table_editor_values::parse_update_value("", "integer").is_err());
    }

    #[test]
    fn update_parser_validates_temporal_json_and_binary_values() {
        assert!(table_editor_values::parse_update_value("2026-09-12", "date").is_ok());
        assert!(table_editor_values::parse_update_value("12:30:45", "time").is_ok());
        assert!(table_editor_values::parse_update_value("2026-09-12T12:30:45Z", "timestamptz").is_ok());
        assert!(table_editor_values::parse_update_value("not-a-time", "time").is_err());
        assert!(table_editor_values::parse_update_value("{\"ok\":true}", "jsonb").is_ok());
        assert!(table_editor_values::parse_update_value("not-json", "jsonb").is_err());
        assert!(table_editor_values::parse_update_value("deadbeef", "bytea").is_err());
    }

    #[test]
    fn filter_operators_follow_column_type_and_keep_null_operators_universally() {
        assert!(TableDataQueryState::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::GreaterThan
        ));
        assert!(!TableDataQueryState::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::Contains
        ));
        assert!(TableDataQueryState::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::Equals
        ));
        assert!(!TableDataQueryState::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::StartsWith
        ));
        assert!(TableDataQueryState::filter_operator_supported(
            "boolean",
            &UiTableFilterOperator::IsNotNull
        ));
    }

    #[test]
    fn filter_draft_supports_multiple_and_editable_same_column_filters() {
        let mut app = DbProApp {
            table: TableEditorState {
                state: TableState {
                    table_info: Some(UiTableInfo {
                        schema: "public".to_owned(),
                        name: "orders".to_owned(),
                        row_count: None,
                        columns: vec![crate::UiTableColumn {
                            name: "amount".to_owned(),
                            data_type: "numeric(12,2)".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: false,
                            ..Default::default()
                        }],
                        primary_key: None,
                        indexes: Vec::new(),
                        foreign_keys: Vec::new(),
                        check_constraints: Vec::new(),
                        dependencies: Vec::new(),
                    }),
                    ..Default::default()
                },
                data_query: TableDataQueryState {
                    filter_column: "amount".to_owned(),
                    filter_operator: UiTableFilterOperator::GreaterThan,
                    filter_value: "10.00".to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        app.commit_table_filter_draft();
        app.table.data_query.filter_operator = UiTableFilterOperator::LessThan;
        app.table.data_query.filter_value = "20.00".to_owned();
        app.commit_table_filter_draft();

        assert_eq!(app.table.data_query.filters.len(), 2);
        app.table.data_query.filter_editing = Some(0);
        app.table.data_query.filter_value = "11.00".to_owned();
        app.commit_table_filter_draft();
        assert_eq!(app.table.data_query.filters[0].value, "11.00");
        assert_eq!(app.table.data_query.filters.len(), 2);
    }