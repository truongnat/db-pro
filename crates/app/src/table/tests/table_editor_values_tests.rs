//! Tests for `../table_editor_values.rs` (moved out of the source file).
    use super::*;
    use crate::{UiTableColumn, UiTableInfo};

    fn info() -> UiTableInfo {
        UiTableInfo {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: None,
            columns: vec![
                UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: true,
                    is_primary_key: true,
                    is_identity: true,
                    ..Default::default()
                },
                UiTableColumn {
                    name: "name".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn insert_row_mapping_skips_identity_and_preserves_text() {
        let table = info();

        let (columns, values) = parse_insert_row_values(&table, &[String::new(), "Ada".to_owned()]).expect("row");

        assert_eq!(columns, vec!["name"]);
        assert_eq!(values, vec![UiCell::Text("Ada".to_owned())]);
    }

    #[test]
    fn duplicate_row_mapping_clears_identity_and_copies_writable_values() {
        let table = info();
        let row = vec![UiCell::Number("7".to_owned()), UiCell::Text("Ada".to_owned())];

        let values = duplicate_row_values(&table, &row);

        assert_eq!(values, vec![String::new(), "Ada".to_owned()]);
    }