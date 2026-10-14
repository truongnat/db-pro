//! Tests for `../synthetic_data.rs` (moved out of the source file).
    use super::*;
    use crate::{UiSchemaColumn, UiSchemaForeignKey};

    fn summary() -> UiTableSummary {
        UiTableSummary {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "customer_id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: false,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "orders_customer_fk".to_owned(),
                from_columns: vec!["customer_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "customers".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        }
    }

    #[test]
    fn plan_builder_resolves_table_and_bounded_fk_seed_values() {
        let plan = build_plan("public.orders", "75", "42", "10", &[summary()]).expect("plan");

        assert_eq!(plan.schema, "public");
        assert_eq!(plan.table, "orders");
        assert_eq!(plan.row_count, 75);
        assert_eq!(plan.columns[0].fk_values.len(), 50);
        assert_eq!(plan.columns[0].fk_values[0], "1");
    }

    #[test]
    fn plan_builder_reports_invalid_inputs_at_the_boundary() {
        let error = build_plan("public.orders", "not-a-number", "42", "0", &[summary()]).expect_err("error");

        assert_eq!(error, "invalid row count");
    }