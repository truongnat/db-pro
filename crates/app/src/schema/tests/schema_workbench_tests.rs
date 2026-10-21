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

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn replan_fingerprint_detects_form_mutation_after_plan() {
        use db_pro_core::application::ObjectMutationService;
        use db_pro_core::domain::object_mutation::ObjectAction;

        let mut state = SchemaWorkbenchState {
            schema: "public".into(),
            name: "audit".into(),
            mode: SchemaWorkbenchMode::SchemaDb,
            cascade: true,
            ..Default::default()
        };

        // Plan a drop and store the same linkage `run_plan` records.
        let request = state
            .build_mutation_request(ObjectAction::Drop, "postgresql")
            .expect("request");
        let preview = ObjectMutationService::plan_for_driver(&request).expect("plan");
        state.preview_sql = preview.statements.join(";\n");
        state.preview_fingerprint = preview.fingerprint.clone();
        state.pending_plan = Some(WorkbenchPendingPlan::Object(ObjectAction::Drop));

        // Unchanged form replans to the same fingerprint.
        assert_eq!(
            state.replan_fingerprint("postgresql").expect("replan"),
            preview.fingerprint
        );

        // Mutating the form after the plan makes the preview stale.
        state.name = "audit_v2".into();
        let live = state.replan_fingerprint("postgresql").expect("replan");
        assert_ne!(live, preview.fingerprint);

        // Without a recorded plan there is nothing to verify against.
        state.pending_plan = None;
        assert!(state.replan_fingerprint("postgresql").is_err());
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn database_action_flags_follow_the_action_kind() {
        use db_pro_core::domain::object_mutation::ObjectAction;

        let state = SchemaWorkbenchState {
            name: "appdb".into(),
            cascade: true,
            ..Default::default()
        };

        let drop = state.build_database_request(ObjectAction::Drop, "postgresql");
        assert!(drop.options.if_exists);
        assert!(!drop.options.if_not_exists);
        assert!(!drop.options.cascade);

        let create = state.build_database_request(ObjectAction::Create, "postgresql");
        assert!(!create.options.if_exists);
        assert!(create.options.if_not_exists);
        assert!(!create.options.cascade);
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn auto_increment_requires_primary_key_and_rejects_postgres_alter() {
        use db_pro_core::domain::object_mutation::ObjectAction;

        let mut state = SchemaWorkbenchState {
            name: "t".into(),
            ..Default::default()
        };
        state.table_columns[0].auto_increment = true;
        state.table_columns[0].is_pk = false;

        // SQLite: non-PK auto_increment is rejected instead of rewriting the type.
        assert!(state
            .build_mutation_request(ObjectAction::Create, "sqlite")
            .is_err());
        // MySQL: same requirement — AUTO_INCREMENT needs a key.
        assert!(state
            .build_mutation_request(ObjectAction::Create, "mysql")
            .is_err());

        state.table_columns[0].is_pk = true;
        // Postgres create path still maps to a serial type.
        let request = state
            .build_mutation_request(ObjectAction::Create, "postgresql")
            .expect("request");
        match request.definition {
            ObjectDefinition::Table(def) => {
                assert!(matches!(def.columns[0].data_type.as_str(), "SERIAL" | "BIGSERIAL"));
            }
            _ => panic!("expected a table definition"),
        }

        // Postgres alter path must not emit SERIAL.
        state.table_baseline = state.table_columns.clone();
        assert!(state
            .build_mutation_request(ObjectAction::Create, "postgresql")
            .is_err());
    }
