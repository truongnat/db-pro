//! Tests for `../security_rls.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn table_rls_preview_rejects_missing_identity() {
        assert_eq!(
            plan_table_rls(TableRlsPreviewRequest {
                schema: "public",
                table: "",
                force: false,
                enable: true,
            }),
            Err("Schema and table are required".into())
        );
    }

    #[test]
    fn policy_preview_quotes_identifiers_and_keeps_explicit_roles() {
        let sql = plan_policy(PolicyPreviewRequest {
            action: ObjectAction::Create,
            schema: "public",
            table: "orders",
            name: "orders_read",
            command: "SELECT",
            roles_csv: "app_user, reporting",
            using_expr: "tenant_id = current_setting('app.tenant')::int",
            with_check_expr: "",
            restrictive: false,
        })
        .expect("preview");

        assert!(sql.contains("CREATE POLICY"));
        assert!(sql.contains("app_user"));
        assert!(sql.ends_with(';'));
    }