//! Tests for `../schema_object_resolver.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn resolves_view_details_and_quoted_query() {
        let schema = UiSchemaSummary {
            views: vec![UiViewSummary {
                schema: "public".into(),
                name: "order\"summary".into(),
                definition: "SELECT 1".into(),
            }],
            ..UiSchemaSummary::default()
        };

        let details = resolve_schema_object(&schema, &SchemaObjectSelection::View("order\"summary".into()))
            .expect("view should resolve");

        assert_eq!(details.kind, "VIEW");
        assert_eq!(
            details.query,
            "SELECT *\nFROM \"public\".\"order\"\"summary\"\nLIMIT 100;"
        );
    }

    #[test]
    fn resolves_function_display_name_and_security_metadata() {
        let schema = UiSchemaSummary {
            functions: vec![UiFunctionSummary {
                schema: "public".into(),
                name: "calculate".into(),
                routine_type: "FUNCTION".into(),
                data_type: "integer".into(),
                definition: "RETURN 1".into(),
                identity_arguments: "value integer".into(),
                language: "plpgsql".into(),
                volatility: "VOLATILE".into(),
                security_definer: true,
                parameters: Vec::new(),
            }],
            ..UiSchemaSummary::default()
        };

        let details = resolve_schema_object(
            &schema,
            &SchemaObjectSelection::Function {
                name: "calculate".into(),
                identity_arguments: "value integer".into(),
            },
        )
        .expect("function should resolve");

        assert_eq!(details.name, "calculate(value integer)");
        assert!(details
            .metadata
            .expect("metadata should exist")
            .contains("SECURITY DEFINER"));
    }

    #[test]
    fn missing_selection_returns_none() {
        assert!(resolve_schema_object(
            &UiSchemaSummary::default(),
            &SchemaObjectSelection::Trigger("missing".into())
        )
        .is_none());
    }