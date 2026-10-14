//! Tests for `../completion_scope.rs` (moved out of the source file).
use super::*;
use crate::{
    editor::completion::CompletionItemKind,
    query::schema_completion::{CompletionContext, SchemaCompletionProvider},
    runtime::{UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiTableSummary},
};

fn at_end(sql: &str) -> CompletionScope {
    analyze(sql, sql.len(), 0)
}

#[test]
fn outer_where_ignores_from_inside_subquery() {
    let sql = "SELECT * FROM orders WHERE id IN (SELECT id FROM other) AND ";
    let scope = at_end(sql);
    assert_eq!(scope.clause, SqlClause::Where);
    assert_eq!(scope.referenced_tables, vec!["orders".to_string()]);
}

#[test]
fn cursor_inside_subquery_uses_inner_from() {
    let sql = "SELECT * FROM orders WHERE id IN (SELECT id FROM other WHERE ";
    let scope = at_end(sql);
    assert_eq!(scope.clause, SqlClause::Where);
    assert!(scope.referenced_tables.iter().any(|table| table == "other"));
}

#[test]
fn comma_from_maps_both_aliases() {
    let sql = "SELECT * FROM orders o, customers c WHERE ";
    let scope = at_end(sql);
    assert_eq!(scope.aliases.get("o").map(String::as_str), Some("orders"));
    assert_eq!(scope.aliases.get("c").map(String::as_str), Some("customers"));
    assert_eq!(
        scope.referenced_tables,
        vec!["orders".to_string(), "customers".to_string()]
    );

    let bare = at_end("SELECT * FROM a, b b2 ");
    assert_eq!(bare.aliases.get("b2").map(String::as_str), Some("b"));
    assert!(!bare.aliases.contains_key("a"));
    assert_eq!(bare.referenced_tables, vec!["a".to_string(), "b".to_string()]);

    let with_as = at_end("SELECT * FROM public.orders AS o, customers AS c ");
    assert_eq!(with_as.aliases.get("o").map(String::as_str), Some("orders"));
    assert_eq!(with_as.aliases.get("c").map(String::as_str), Some("customers"));
    assert!(!with_as.referenced_tables.iter().any(|table| table == "public"));
}

#[test]
fn references_stay_inside_the_statement_at_the_cursor() {
    let sql = "SELECT * FROM other o; SELECT * FROM orders o, customers c WHERE ";
    let scope = at_end(sql);
    assert_eq!(scope.aliases.get("o").map(String::as_str), Some("orders"));
    assert_eq!(scope.aliases.get("c").map(String::as_str), Some("customers"));
    assert!(!scope.referenced_tables.iter().any(|table| table == "other"));
}

#[test]
fn table_inside_another_word_is_not_referenced() {
    let sql = "SELECT status, created_at FROM orders WHERE ";
    let scope = at_end(sql);
    assert_eq!(scope.referenced_tables, vec!["orders".to_string()]);
    assert!(!scope.referenced_tables.iter().any(|table| table == "at"));
}

#[test]
fn strings_and_comments_are_not_keywords() {
    let quoted = at_end("SELECT * FROM orders WHERE note = 'FROM (other)' /* JOIN */ AND ");
    assert_eq!(quoted.clause, SqlClause::Where);
    assert_eq!(quoted.referenced_tables, vec!["orders".to_string()]);

    let comment = at_end("SELECT * FROM orders -- WHERE other\n");
    assert_eq!(comment.clause, SqlClause::From);
    assert_eq!(comment.referenced_tables, vec!["orders".to_string()]);
}

#[test]
fn insert_column_list_keeps_insert_clause() {
    let scope = analyze("INSERT INTO users (id, em", "INSERT INTO users (id, em".len(), 2);
    assert_eq!(scope.clause, SqlClause::InsertInto);
    assert_eq!(scope.referenced_tables, vec!["users".to_string()]);

    let updated = analyze("UPDATE users SET em", "UPDATE users SET em".len(), 2);
    assert_eq!(updated.clause, SqlClause::Set);
    assert_eq!(updated.referenced_tables, vec!["users".to_string()]);
}

fn column(name: &str, primary: bool) -> UiSchemaColumn {
    UiSchemaColumn {
        name: name.to_owned(),
        data_type: "integer".to_owned(),
        nullable: !primary,
        is_primary_key: primary,
    }
}

fn provide(before: &str, after: &str, summary: &UiSchemaSummary) -> Vec<crate::editor::completion::CompletionItem> {
    let context = CompletionContext {
        text_before_cursor: before,
        text_after_cursor: after,
        cursor_offset: before.len(),
        active_schema: "public",
        schema_summary: summary,
        cached_tokens: None,
        is_sqlite: false,
        is_manual_trigger: true,
    };
    SchemaCompletionProvider::provide(&context).1
}

#[test]
fn substring_table_name_is_not_a_referenced_completion() {
    let summary = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: None,
                columns: vec![column("email", false)],
                foreign_keys: vec![],
            },
            UiTableSummary {
                schema: "public".to_owned(),
                name: "at".to_owned(),
                row_count: None,
                columns: vec![column("email_secret", false)],
                foreign_keys: vec![],
            },
        ],
        ..Default::default()
    };
    let items = provide("SELECT status, created_at FROM orders WHERE em", "", &summary);
    assert!(items.iter().any(|item| item.label == "email"));
    assert!(!items.iter().any(|item| item.label.contains("email_secret")));
}

#[test]
fn join_suggests_the_missing_foreign_key_table_with_on() {
    let summary = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: None,
                columns: vec![column("customer_id", false)],
                foreign_keys: vec![UiSchemaForeignKey {
                    name: "orders_customer".to_owned(),
                    from_columns: vec!["customer_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: "customers".to_owned(),
                    to_columns: vec!["id".to_owned()],
                }],
            },
            UiTableSummary {
                schema: "public".to_owned(),
                name: "customers".to_owned(),
                row_count: None,
                columns: vec![column("id", true)],
                foreign_keys: vec![],
            },
        ],
        ..Default::default()
    };
    let items = provide("SELECT * FROM orders JOIN cu", "", &summary);
    let snippet = items
        .iter()
        .find(|item| item.insert_text.contains("customers ON"))
        .expect("missing FK join target");
    assert!(snippet.insert_text.contains("orders.customer_id = customers.id"));
}

#[test]
fn ambiguous_columns_use_alias_or_table_qualifier() {
    let summary = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: None,
                columns: vec![column("id", true)],
                foreign_keys: vec![],
            },
            UiTableSummary {
                schema: "public".to_owned(),
                name: "customers".to_owned(),
                row_count: None,
                columns: vec![column("id", true)],
                foreign_keys: vec![],
            },
        ],
        ..Default::default()
    };
    let aliased = provide("SELECT i", " FROM orders o, customers c", &summary);
    assert!(aliased.iter().any(|item| item.insert_text == "o.id"));
    assert!(aliased.iter().any(|item| item.insert_text == "c.id"));

    let bare = provide("SELECT i", " FROM orders, customers", &summary);
    assert!(bare.iter().any(|item| item.insert_text == "orders.id"));
    assert!(bare.iter().any(|item| item.insert_text == "customers.id"));
}

#[test]
fn join_suggestion_insert_text_contains_on() {
    let summary = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "customers".to_owned(),
                row_count: None,
                columns: vec![column("id", true)],
                foreign_keys: vec![],
            },
            UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: None,
                columns: vec![column("customer_id", false)],
                foreign_keys: vec![UiSchemaForeignKey {
                    name: "orders_customer_id_fkey".to_owned(),
                    from_columns: vec!["customer_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: "customers".to_owned(),
                    to_columns: vec!["id".to_owned()],
                }],
            },
        ],
        ..Default::default()
    };
    let items = provide("SELECT * FROM orders JOIN customers ON", "", &summary);
    assert!(items.iter().any(|item| {
        item.kind == CompletionItemKind::Snippet
            && item.insert_text.contains("ON")
            && item.insert_text.contains('=')
            && item.insert_text.contains("orders.customer_id")
            && item.insert_text.contains("customers.id")
    }));
}
