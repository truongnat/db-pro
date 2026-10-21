//! Tests for `../routine_workbench_surface_view.rs` (moved out of the source file).
    use super::*;
    use db_pro_core::domain::query::QueryParam;

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

    fn param(name: &str, data_type: &str) -> UiRoutineParameter {
        UiRoutineParameter {
            name: name.into(),
            data_type: data_type.into(),
            mode: "IN".into(),
            has_default: false,
            default_expr: String::new(),
        }
    }

    fn param_with_default(name: &str, data_type: &str, default_expr: &str) -> UiRoutineParameter {
        UiRoutineParameter {
            name: name.into(),
            data_type: data_type.into(),
            mode: "IN".into(),
            has_default: true,
            default_expr: default_expr.into(),
        }
    }

    fn two_arg_function() -> UiFunctionSummary {
        sample_fn(
            "FUNCTION",
            vec![param("x", "integer"), param_with_default("label", "text", "'hi'")],
        )
    }

    // ── Statement shape ────────────────────────────────────────────────────

    #[test]
    fn function_execute_uses_select_with_placeholders_only() {
        let function = two_arg_function();
        // `x` bound, `label` empty with a default → DEFAULT keyword, no bind.
        // `integer` binds as bigint, so the placeholder carries the declared
        // type as an explicit cast for signature-exact resolution (#252).
        let invoke = build_routine_invoke_sql(
            &function,
            &["42".into(), String::new()],
            &[false, false],
            "postgresql",
        );
        assert_eq!(
            invoke.sql,
            "SELECT * FROM \"public\".\"calc\"($1::integer, DEFAULT);"
        );
        assert_eq!(invoke.arguments, Ok(vec![QueryParam::Int64(42)]));
        assert!(
            !invoke.bound_with_cast,
            "a directly bound family with a resolution cast must not trigger the text-bound note"
        );
    }

    #[test]
    fn procedure_execute_uses_call() {
        let function = sample_fn("PROCEDURE", vec![]);
        let invoke = build_routine_invoke_sql(&function, &[], &[], "postgresql");
        assert_eq!(invoke.sql, "CALL \"public\".\"calc\"();");
        assert_eq!(invoke.arguments, Ok(vec![]));
    }

    #[test]
    fn sqlite_placeholder_style_is_used_for_non_postgres_drivers() {
        let function = two_arg_function();
        let invoke = build_routine_invoke_sql(&function, &["42".into(), "x".into()], &[false, false], "sqlite");
        assert_eq!(invoke.sql, "SELECT * FROM \"public\".\"calc\"(?, ?);");
    }

    // ── §12.1: no literal argument value appears in the generated SQL ─────

    #[test]
    fn generated_sql_contains_no_literal_argument_values() {
        let function = sample_fn(
            "FUNCTION",
            vec![
                param("x", "integer"),
                param("label", "text"),
                param("payload", "jsonb"),
                param("flag", "boolean"),
            ],
        );
        let values = ["7".to_owned(), "O'Brien".to_owned(), "{\"a\":1}".to_owned(), "true".to_owned()];
        let invoke = build_routine_invoke_sql(&function, &values, &[false, false, false, false], "postgresql");
        // `integer` binds as bigint, so only it carries the resolution cast.
        assert_eq!(
            invoke.sql,
            "SELECT * FROM \"public\".\"calc\"($1::integer, $2, $3, $4);"
        );
        for literal in values {
            assert!(!invoke.sql.contains(&literal), "literal {literal:?} leaked into generated SQL: {}", invoke.sql);
        }
        assert_eq!(
            invoke.arguments,
            Ok(vec![
                QueryParam::Int64(7),
                QueryParam::Text("O'Brien".to_owned()),
                QueryParam::Json(serde_json::json!({"a": 1})),
                QueryParam::Bool(true),
            ])
        );
    }

    // ── Injection regression: the audit payload never reaches SQL text ─────

    #[test]
    fn numeric_injection_payload_stays_out_of_sql_and_is_a_typed_failure() {
        let function = two_arg_function();
        let invoke = build_routine_invoke_sql(
            &function,
            &["1) OR (1=1".to_owned(), "x".to_owned()],
            &[false, false],
            "postgresql",
        );
        // Placeholders only: the payload is nowhere in the statement…
        // (a `"1)"` substring check would false-positive on the `$1);` tail)
        assert!(!invoke.sql.contains("OR"), "injection payload leaked into SQL: {}", invoke.sql);
        assert!(!invoke.sql.contains("(1=1"), "injection payload leaked into SQL: {}", invoke.sql);
        assert!(invoke.sql.contains("$1"), "expected a bind placeholder, got: {}", invoke.sql);
        // …and it is not coerced into a wrong bind either — typed validation failure.
        let error = invoke
            .arguments
            .expect_err("\"1) OR (1=1\" must not bind as an integer");
        assert!(error.contains("x"), "failure must name the parameter: {error}");
        assert!(error.contains("integer"), "failure must name the type: {error}");
    }

    #[test]
    fn injection_payload_in_text_argument_binds_as_ordinary_text() {
        let function = sample_fn("FUNCTION", vec![param("label", "text")]);
        let invoke = build_routine_invoke_sql(&function, &["'; DROP TABLE users; --".to_owned()], &[false], "postgresql");
        assert_eq!(invoke.sql, "SELECT * FROM \"public\".\"calc\"($1);");
        assert_eq!(
            invoke.arguments,
            Ok(vec![QueryParam::Text("'; DROP TABLE users; --".to_owned())])
        );
    }

    // ── Semantics: no silent text magic ────────────────────────────────────

    #[test]
    fn text_null_string_stays_a_text_value_not_sql_null() {
        let function = sample_fn("FUNCTION", vec![param("label", "text")]);
        let invoke = build_routine_invoke_sql(&function, &["null".to_owned()], &[false], "postgresql");
        assert_eq!(invoke.sql, "SELECT * FROM \"public\".\"calc\"($1);");
        assert_eq!(invoke.arguments, Ok(vec![QueryParam::Text("null".to_owned())]));
    }

    #[test]
    fn null_checkbox_binds_typed_null_behind_a_placeholder() {
        let function = two_arg_function();
        let invoke = build_routine_invoke_sql(&function, &["42".into(), "x".into()], &[false, true], "postgresql");
        assert_eq!(
            invoke.sql,
            "SELECT * FROM \"public\".\"calc\"($1::integer, $2);"
        );
        assert_eq!(invoke.arguments, Ok(vec![QueryParam::Int64(42), QueryParam::Null]));
    }

    #[test]
    fn empty_numeric_without_null_checkbox_is_a_typed_failure_not_magic_null() {
        let function = sample_fn("FUNCTION", vec![param("x", "integer")]);
        let invoke = build_routine_invoke_sql(&function, &["".to_owned()], &[false], "postgresql");
        let error = invoke.arguments.expect_err("empty numeric must not coerce to NULL");
        assert!(error.contains("NULL"), "error must point at the NULL checkbox: {error}");
    }

    // ── Unicode / multiline values ─────────────────────────────────────────

    #[test]
    fn unicode_and_multiline_text_arguments_bind_unchanged() {
        let function = sample_fn("FUNCTION", vec![param("label", "text")]);
        let value = "héllo wörld\n多行\n'quoted'".to_owned();
        let invoke = build_routine_invoke_sql(&function, std::slice::from_ref(&value), &[false], "postgresql");
        assert_eq!(invoke.sql, "SELECT * FROM \"public\".\"calc\"($1);");
        assert_eq!(invoke.arguments, Ok(vec![QueryParam::Text(value)]));
    }

    // ── §9.3 family mapping ────────────────────────────────────────────────

    #[test]
    fn type_families_map_to_their_query_param_variants() {
        let function = sample_fn(
            "FUNCTION",
            vec![
                param("b", "boolean"),
                param("n", "numeric(10,2)"),
                param("f", "double precision"),
                param("u", "uuid"),
                param("ts", "timestamp with time zone"),
                param("d", "date"),
                param("t", "time without time zone"),
                param("iv", "interval"),
                param("ip", "inet"),
                param("raw", "bytea"),
            ],
        );
        let values = [
            "yes",
            "10.25",
            "1.5",
            "11111111-1111-1111-1111-111111111111",
            "2026-10-22T09:00:00Z",
            "2026-10-22",
            "09:30:00",
            "1 day",
            "10.0.0.1",
            "00ff10",
        ]
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<String>>();
        let nulls = vec![false; values.len()];
        let invoke = build_routine_invoke_sql(&function, &values, &nulls, "postgresql");
        assert_eq!(
            invoke.arguments,
            Ok(vec![
                QueryParam::Bool(true),
                QueryParam::Decimal("10.25".to_owned()),
                QueryParam::Float64(1.5),
                QueryParam::Uuid("11111111-1111-1111-1111-111111111111".to_owned()),
                QueryParam::DateTime("2026-10-22T09:00:00Z".to_owned()),
                QueryParam::DateTime("2026-10-22".to_owned()),
                QueryParam::Time("09:30:00".to_owned()),
                QueryParam::Interval("1 day".to_owned()),
                QueryParam::Inet("10.0.0.1".to_owned()),
                QueryParam::Bytes(vec![0x00, 0xff, 0x10]),
            ])
        );
        // Only `time` needs the cast: whether it binds as `time` or `timetz`
        // depends on the entered text, so resolution is pinned explicitly.
        assert_eq!(
            invoke.sql,
            "SELECT * FROM \"public\".\"calc\"($1, $2, $3, $4, $5, $6, $7::time without time zone, $8, $9, $10);"
        );
    }

    #[test]
    fn non_bindable_types_are_text_bound_with_an_explicit_cast() {
        let function = sample_fn("FUNCTION", vec![param("ids", "integer[]")]);
        let invoke = build_routine_invoke_sql(&function, &["{1,2,3}".to_owned()], &[false], "postgresql");
        assert_eq!(invoke.sql, "SELECT * FROM \"public\".\"calc\"($1::integer[]);");
        assert_eq!(invoke.arguments, Ok(vec![QueryParam::Text("{1,2,3}".to_owned())]));
        assert!(invoke.bound_with_cast, "cast limitation must be stated in the form");
    }

    #[test]
    fn unsafe_catalog_type_names_fail_validation_instead_of_reaching_the_cast() {
        let mut parameter = param("ids", "integer[]); DROP TABLE users; --");
        parameter.data_type = "integer[]); DROP TABLE users; --".into();
        let function = sample_fn("FUNCTION", vec![parameter]);
        let invoke = build_routine_invoke_sql(&function, &["{1}".to_owned()], &[false], "postgresql");
        assert!(!invoke.sql.contains("DROP"), "catalog-shaped payload must not reach SQL text: {}", invoke.sql);
        let error = invoke.arguments.expect_err("unsafe type must be a typed validation failure");
        assert!(error.contains("cannot be safely cast"), "{error}");
    }
