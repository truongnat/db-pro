//! Tests for `../routine_workbench_surface_view.rs` (moved out of the source file).
    use super::*;

    fn sample_fn(routine_type: &str, params: Vec<UiRoutineParameter>) -> UiFunctionSummary {
        UiFunctionSummary {
            schema: "public".into(),
            name: "calc".into(),
            routine_type: routine_type.into(),
            data_type: "integer".into(),
            definition: "CREATE FUNCTION ...".into(),
            identity_arguments: "x integer".into(),
            language: "sql".into(),
            volatility: "VOLATILE".into(),
            security_definer: false,
            parameters: params,
        }
    }

    #[test]
    fn function_execute_uses_select_and_null_handling() {
        let function = sample_fn(
            "FUNCTION",
            vec![
                UiRoutineParameter {
                    name: "x".into(),
                    data_type: "integer".into(),
                    mode: "IN".into(),
                    has_default: false,
                    default_expr: "".into(),
                },
                UiRoutineParameter {
                    name: "label".into(),
                    data_type: "text".into(),
                    mode: "IN".into(),
                    has_default: true,
                    default_expr: "'hi'".into(),
                },
            ],
        );
        let sql = build_routine_invoke_sql(&function, &["42".into(), String::new()], &[false, false]);
        assert_eq!(sql, "SELECT * FROM \"public\".\"calc\"(42, DEFAULT);");
        let sql_null = build_routine_invoke_sql(&function, &["42".into(), "x".into()], &[false, true]);
        assert_eq!(sql_null, "SELECT * FROM \"public\".\"calc\"(42, NULL);");
    }

    #[test]
    fn procedure_execute_uses_call() {
        let function = sample_fn("PROCEDURE", vec![]);
        let sql = build_routine_invoke_sql(&function, &[], &[]);
        assert_eq!(sql, "CALL \"public\".\"calc\"();");
    }