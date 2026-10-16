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
    #[test]
    fn policy_preview_rejects_expression_that_escapes_the_policy_wrapper() {
        let err = plan_policy(PolicyPreviewRequest {
            action: ObjectAction::Create,
            schema: "public",
            table: "orders",
            name: "tenant_read",
            command: "SELECT",
            roles_csv: "",
            using_expr: "true); DROP TABLE important_data; --",
            with_check_expr: "",
            restrictive: false,
        })
        .expect_err("malicious expression must be rejected");
        assert!(err.contains("using_expr"), "unexpected error: {err}");
    }

    #[test]
    fn policy_preview_keeps_valid_expression() {
        let sql = plan_policy(PolicyPreviewRequest {
            action: ObjectAction::Create,
            schema: "public",
            table: "orders",
            name: "tenant_read",
            command: "SELECT",
            roles_csv: "",
            using_expr: "tenant_id = current_setting('app.tenant')::uuid",
            with_check_expr: "",
            restrictive: false,
        })
        .expect("valid expression");
        assert!(sql.contains("USING (tenant_id = current_setting('app.tenant')::uuid)"));
    }
