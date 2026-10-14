//! Tests for `../introspect.rs` (moved out of the source file).
use super::*;

#[test]
fn postgres_trigger_enabled_state_maps_all_catalog_modes() {
    assert!(!trigger_enabled_from_flag("D"));
    for flag in ["O", "R", "A"] {
        assert!(trigger_enabled_from_flag(flag), "{flag} should be enabled");
    }
}

#[test]
fn test_parse_simple_columns() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (col1, col2)";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["col1", "col2"]);
}

#[test]
fn test_parse_single_column() {
    let indexdef = "CREATE INDEX idx ON tbl USING hash (id)";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["id"]);
}

#[test]
fn test_quoted_identifier_with_parenthesis_and_comma() {
    let indexdef = "CREATE INDEX idx ON \"tbl(name)\" USING btree (\"a,b\", \"quoted\"\"name\")";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["\"a,b\"", "\"quoted\"\"name\""]);
}

#[test]
fn test_parse_functional_index() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (lower(name))";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["lower(name)"]);
}

#[test]
fn test_parse_nested_parentheses() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (lower(name), (a + b))";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["lower(name)", "(a + b)"]);
}

#[test]
fn test_parse_function_with_multiple_args() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (coalesce(a, b), c)";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["coalesce(a, b)", "c"]);
}

#[test]
fn test_parse_mixed_columns_and_functions() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (col1, lower(col2), (a * b + c))";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["col1", "lower(col2)", "(a * b + c)"]);
}

#[test]
fn parse_routine_parameters_handles_modes_and_defaults() {
    let params = parse_routine_parameters("order_id integer, OUT total numeric DEFAULT 0");
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "order_id");
    assert_eq!(params[0].mode, "IN");
    assert_eq!(params[1].mode, "OUT");
    assert!(params[1].has_default);
    assert_eq!(params[1].default_expr, "0");
}

#[test]
fn test_parse_deeply_nested() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (func1(func2(x, y), z))";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["func1(func2(x, y), z)"]);
}

#[test]
fn test_parse_empty_parentheses() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree ()";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, Vec::<String>::new());
}

#[test]
fn test_parse_no_parentheses() {
    let indexdef = "CREATE INDEX idx ON tbl";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, Vec::<String>::new());
}

#[test]
fn test_parse_whitespace_handling() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree (  col1  ,  col2  )";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["col1", "col2"]);
}

#[test]
fn test_parse_expression_with_spaces() {
    let indexdef = "CREATE INDEX idx ON tbl USING btree ((a + b) * c, lower(d))";
    let cols = parse_index_columns(indexdef);
    assert_eq!(cols, vec!["(a + b) * c", "lower(d)"]);
}

#[test]
// allow: test simulates composite FK grouping logic; tuple map deliberately mirrors
// production data structure for direct comparison.
#[allow(clippy::type_complexity)]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
fn test_composite_fk_grouping() {
    // Simulate the grouping logic from introspect_foreign_keys
    let mut map: std::collections::HashMap<(String, String, String, String, String), (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();
    let mut order: Vec<(String, String, String, String, String)> = Vec::new();

    // Simulate rows for a composite FK with 2 columns
    let rows = vec![
        (
            "fk_composite".to_string(),
            "orders".to_string(),
            "customers".to_string(),
            "public".to_string(),
            "public".to_string(),
            "customer_id".to_string(),
            "id".to_string(),
        ),
        (
            "fk_composite".to_string(),
            "orders".to_string(),
            "customers".to_string(),
            "public".to_string(),
            "public".to_string(),
            "tenant_id".to_string(),
            "tenant_id".to_string(),
        ),
    ];

    for (name, from_table, to_table, schema, to_schema, from_col, to_col) in rows {
        let key = (
            name.clone(),
            from_table.clone(),
            to_table.clone(),
            schema.clone(),
            to_schema.clone(),
        );
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(key.clone(), (Vec::new(), Vec::new()));
        }
        let (from_cols, to_cols) = map.get_mut(&key).unwrap();
        from_cols.push(from_col);
        to_cols.push(to_col);
    }

    assert_eq!(order.len(), 1);
    let key = &order[0];
    let (from_columns, to_columns) = map.get(key).unwrap();
    assert_eq!(from_columns, &vec!["customer_id".to_string(), "tenant_id".to_string()]);
    assert_eq!(to_columns, &vec!["id".to_string(), "tenant_id".to_string()]);
}

#[test]
// allow: test covers multiple separate FKs; retains exact production tuple shape
// rather than introducing a test-only struct that would require proving equivalence.
#[allow(clippy::type_complexity)]
fn test_multiple_separate_fks() {
    let mut map: std::collections::HashMap<(String, String, String, String, String), (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();
    let mut order: Vec<(String, String, String, String, String)> = Vec::new();

    // Simulate two separate FKs
    let rows = vec![
        (
            "fk_user".to_string(),
            "orders".to_string(),
            "users".to_string(),
            "public".to_string(),
            "public".to_string(),
            "user_id".to_string(),
            "id".to_string(),
        ),
        (
            "fk_product".to_string(),
            "orders".to_string(),
            "products".to_string(),
            "public".to_string(),
            "public".to_string(),
            "product_id".to_string(),
            "id".to_string(),
        ),
    ];

    for (name, from_table, to_table, schema, to_schema, from_col, to_col) in rows {
        let key = (
            name.clone(),
            from_table.clone(),
            to_table.clone(),
            schema.clone(),
            to_schema.clone(),
        );
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(key.clone(), (Vec::new(), Vec::new()));
        }
        let (from_cols, to_cols) = map.get_mut(&key).unwrap();
        from_cols.push(from_col);
        to_cols.push(to_col);
    }

    assert_eq!(order.len(), 2);
    assert_eq!(map.get(&order[0]).unwrap().0, vec!["user_id".to_string()]);
    assert_eq!(map.get(&order[1]).unwrap().0, vec!["product_id".to_string()]);
}
