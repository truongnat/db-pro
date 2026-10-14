//! Tests for `../schema_workbench.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn editing_a_loaded_table_plans_alter_not_create() {
        use crate::runtime::{UiSchemaColumn, UiTableSummary};
        use db_pro_core::application::ObjectMutationService;
        use db_pro_core::domain::object_mutation::ObjectAction;

        let mut state = SchemaWorkbenchState::default();
        state.load_from_table_details(&UiTableSummary {
            schema: "public".into(),
            name: "orders".into(),
            row_count: None,
            columns: vec![
                UiSchemaColumn {
                    name: "id".into(),
                    data_type: "integer".into(),
                    nullable: false,
                    is_primary_key: true,
                },
                UiSchemaColumn {
                    name: "label".into(),
                    data_type: "text".into(),
                    nullable: true,
                    is_primary_key: false,
                },
            ],
            foreign_keys: vec![],
        });
        state.table_columns[0].data_type = "bigint".into();
        state.table_columns[1].name = "title".into();
        state.table_columns[1].nullable = false;
        state.table_columns[1].default_expr = "'x'".into();

        let request = state
            .build_mutation_request(ObjectAction::Create, "postgresql")
            .expect("request");
        let preview = ObjectMutationService::plan_for_driver(&request).expect("plan");
        let sql = preview.statements.join("\n");
        let upper = sql.to_ascii_uppercase();
        assert!(!upper.contains("CREATE TABLE"), "{sql}");
        assert!(!upper.contains("IF NOT EXISTS"), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"id\" TYPE bigint"), "{sql}");
        assert!(sql.contains("RENAME COLUMN \"label\" TO \"title\""), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"title\" SET NOT NULL"), "{sql}");
        assert!(sql.contains("ALTER COLUMN \"title\" SET DEFAULT 'x'"), "{sql}");
    }

    #[test]
    fn parse_columns_with_pk_flag() {
        let cols = parse_column_defs("public", "t", "id:INTEGER:pk,name:TEXT:nn,note:TEXT").unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "id");
        assert!(cols[0].is_pk);
        assert_eq!(cols[1].name, "name");
        assert!(!cols[1].nullable);
        assert!(!cols[1].is_pk);
        assert_eq!(cols[2].name, "note");
        assert!(cols[2].nullable);
    }

    #[test]
    fn execution_history_recording_and_capping() {
        let mut wb = SchemaWorkbenchState::default();
        for i in 0..110 {
            wb.record_execution("Table", &format!("t_{i}"), "CREATE TABLE ...", "Safe", true);
        }
        assert_eq!(wb.history.len(), 100);
        assert_eq!(wb.history[0].object_name, "t_109");
    }
