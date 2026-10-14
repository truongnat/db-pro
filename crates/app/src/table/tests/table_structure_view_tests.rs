//! Tests for `../table_structure_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn plan_column_alter_seeds_workbench_and_plans_alter_sql() {
        let mut app = DbProApp::default();
        let column = crate::UiTableColumn {
            name: "email".into(),
            data_type: "VARCHAR(255)".into(),
            ordinal: 2,
            nullable: true,
            default: Some("'x'".into()),
            is_primary_key: false,
            ..Default::default()
        };
        let info = UiTableInfo {
            schema: "public".into(),
            name: "users".into(),
            row_count: None,
            columns: vec![column.clone()],
            primary_key: None,
            indexes: vec![],
            foreign_keys: vec![],
            check_constraints: vec![],
            dependencies: vec![],
        };

        app.plan_column_alter(&info, &column);

        let workbench = &app.schema.workbench;
        assert_eq!(workbench.mode, SchemaWorkbenchMode::Column);
        assert_eq!(workbench.parent_table, "users");
        assert_eq!(workbench.name, "email");
        assert_eq!(workbench.data_type, "VARCHAR(255)");
        assert_eq!(workbench.default_expr, "'x'");
        assert!(workbench.preview_error.is_none());
        assert!(workbench
            .preview_sql
            .contains("ALTER TABLE \"public\".\"users\" ALTER COLUMN \"email\" TYPE VARCHAR(255)"));
        assert_eq!(app.workspace.active_tab, WorkspaceTab::SchemaWorkbench);
    }