//! Tests for `../schema_compare.rs` (moved out of the source file).
    use super::*;
    use crate::runtime::{UiSchemaColumn, UiTableSummary};

    #[test]
    fn detects_added_table_and_column_type_change() {
        let source = UiSchemaSnapshot {
            label: "a".into(),
            tables: vec![UiTableSummary {
                schema: "public".into(),
                name: "t".into(),
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
        let target = UiSchemaSnapshot {
            label: "b".into(),
            tables: vec![
                UiTableSummary {
                    schema: "public".into(),
                    name: "t".into(),
                    row_count: None,
                    columns: vec![UiSchemaColumn {
                        name: "id".into(),
                        data_type: "bigint".into(),
                        nullable: false,
                        is_primary_key: true,
                    }],
                    foreign_keys: vec![],
                },
                UiTableSummary {
                    schema: "public".into(),
                    name: "u".into(),
                    row_count: None,
                    columns: vec![],
                    foreign_keys: vec![],
                },
            ],
            ..Default::default()
        };
        let diff = diff_snapshots(&source, &target);
        assert!(diff.tables_only_in_target.iter().any(|t| t == "public.u"));
        assert!(diff.column_changes.iter().any(|c| c.contains("integer → bigint")));
    }