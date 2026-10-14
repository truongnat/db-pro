//! Tests for `../schema_compare_state.rs` (moved out of the source file).
    use super::{FeedbackState, SchemaCompareState};

    #[test]
    fn defaults_are_safe_for_schema_change_operations() {
        let state = SchemaCompareState::default();

        assert!(state.schema_snapshot.is_none());
        assert!(state.schema_diff.is_none());
        assert!(state.migration_plan.is_none());
        assert!(!state.migration_confirm_destructive);
        assert_eq!(state.data_diff_schema, "public");
        assert_eq!(state.data_diff_filter, "all");
    }

    #[test]
    fn data_diff_request_requires_target_table_and_key_columns() {
        let state = SchemaCompareState::default();

        assert_eq!(
            state.prepare_data_diff_request().err(),
            Some("Target connection id is required".to_owned())
        );
    }

    #[test]
    fn data_diff_request_normalizes_keys_and_keeps_schema_scope() {
        let state = SchemaCompareState {
            data_diff_target_id: " target ".to_owned(),
            data_diff_schema: " public ".to_owned(),
            data_diff_table: " orders ".to_owned(),
            data_diff_keys: "id, tenant_id, ".to_owned(),
            ..Default::default()
        };

        let request = state
            .prepare_data_diff_request()
            .expect("request");
        assert_eq!(request.target_id, "target");
        assert_eq!(request.schema, "public");
        assert_eq!(request.table, "orders");
        assert_eq!(request.key_columns, vec!["id", "tenant_id"]);
        assert_eq!(request.sample_limit, Some(1_000));
    }

    #[test]
    fn migration_apply_requires_a_planned_migration() {
        let state = SchemaCompareState::default();

        assert_eq!(
            state.prepare_migration_sql("conn · public"),
            Err("Plan a migration before applying".to_owned())
        );
    }

    fn empty_plan() -> db_pro_core::domain::migration::MigrationPlan {
        db_pro_core::domain::migration::MigrationPlan {
            driver: "postgres".to_owned(),
            operations: Vec::new(),
            fingerprint: "fp".to_owned(),
            warnings: Vec::new(),
            has_destructive: false,
        }
    }

    #[test]
    fn migration_apply_is_blocked_when_the_session_target_diverges() {
        let state = SchemaCompareState {
            migration_plan: Some(empty_plan()),
            migration_plan_target: Some("prod-pg · public".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            state.prepare_migration_sql("staging-pg · public"),
            Err(
                "Plan target mismatch — the active session moved since the plan was generated"
                    .to_owned()
            )
        );
    }

    #[test]
    fn migration_apply_proceeds_past_the_target_lock_on_match() {
        let state = SchemaCompareState {
            migration_plan: Some(empty_plan()),
            migration_plan_target: Some("prod-pg · public".to_owned()),
            ..Default::default()
        };

        // Passes the target lock; the next gate (fingerprint) still applies.
        assert_ne!(
            state.prepare_migration_sql("prod-pg · public"),
            Err(
                "Plan target mismatch — the active session moved since the plan was generated"
                    .to_owned()
            )
        );
    }

    #[test]
    fn plan_migration_uses_introspected_columns() {
        use crate::runtime::{UiSchemaColumn, UiSchemaSummary, UiTableSummary};

        let source = UiSchemaSummary {
            table_details: vec![UiTableSummary {
                schema: "public".into(),
                name: "widgets".into(),
                row_count: None,
                columns: vec![UiSchemaColumn {
                    name: "id".into(),
                    data_type: "integer".into(),
                    nullable: false,
                    is_primary_key: true,
                }],
                foreign_keys: vec![],
            }],
            ..Default::default()
        };
        let mut state = SchemaCompareState::default();
        let mut feedback = FeedbackState::default();
        state.take_snapshot(&source, "src", &mut feedback);
        state.diff_against_snapshot(&UiSchemaSummary::default(), &mut feedback);
        state.plan_migration("postgresql", "src · public", &mut feedback);

        assert!(state.migration_preview_sql.contains("\"id\" integer NOT NULL"));
        assert!(state.migration_preview_sql.contains("PRIMARY KEY (\"id\")"));
        assert!(!db_pro_core::application::placeholder_remains(&state.migration_preview_sql));
    }

    #[test]
    fn migration_apply_refuses_a_placeholder() {
        use db_pro_core::domain::migration::{MigrationOpKind, MigrationOperation, MigrationRisk};

        let mut plan = empty_plan();
        plan.operations.push(MigrationOperation {
            id: "op-0001".into(),
            kind: MigrationOpKind::CreateTable,
            schema: "public".into(),
            object: "t".into(),
            sql: "CREATE TABLE t (id int /* placeholder */)".into(),
            risk: MigrationRisk::Mutating,
            dependencies: Vec::new(),
            provider_supported: true,
            unsupported_reason: None,
        });
        let state = SchemaCompareState {
            migration_plan: Some(plan),
            migration_fingerprint_at_preview: "fp".into(),
            migration_plan_target: Some("prod-pg · public".into()),
            ..Default::default()
        };
        let error = state
            .prepare_migration_sql("prod-pg · public")
            .expect_err("placeholder");
        assert!(error.to_ascii_lowercase().contains("placeholder"));
    }