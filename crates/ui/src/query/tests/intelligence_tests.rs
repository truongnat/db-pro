//! Tests for `../intelligence.rs` (moved out of the source file).
use super::*;
use crate::{
    editor::TextBuffer,
    runtime::{UiRoutineParameter, UiSchemaColumn, UiTableSummary},
};

fn tokens(sql: &str, dialect: SqlDialect) -> CachedSqlTokens {
    let mut tokens = CachedSqlTokens::new();
    tokens.get_or_recompute(&TextBuffer::from_string(sql), dialect);
    tokens
}

#[test]
fn index_resolves_active_schema_and_rejects_ambiguous_relations() {
    let schema = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: Some(4),
                columns: vec![],
                foreign_keys: vec![],
            },
            UiTableSummary {
                schema: "audit".to_owned(),
                name: "users".to_owned(),
                row_count: None,
                columns: vec![],
                foreign_keys: vec![],
            },
        ],
        ..Default::default()
    };
    let index = SchemaSymbolIndex::build(&schema);
    assert_eq!(
        index
            .resolve_symbol("users", (0, 5), "public", SqlDialect::Postgres)
            .unwrap()
            .title,
        "public.users"
    );
    assert!(index
        .resolve_symbol("users", (0, 5), "", SqlDialect::Postgres)
        .is_none());
}

#[test]
fn qualified_column_hover_includes_type_and_constraints() {
    let schema = UiSchemaSummary {
        table_details: vec![UiTableSummary {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "bigint".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![],
        }],
        ..Default::default()
    };
    let index = SchemaSymbolIndex::build(&schema);
    let sql = "SELECT users.id";
    let help = index
        .resolve_symbol(sql, (13, 15), "public", SqlDialect::Postgres)
        .unwrap();
    assert_eq!(help.title, "public.users.id");
    assert!(help.detail.contains("primary key"));
}

#[test]
fn nested_function_call_tracks_only_same_level_commas() {
    let sql = "SELECT coalesce(round(total, 2), fallback, 'a,b'";
    let cached = tokens(sql, SqlDialect::Postgres);
    let call = function_call_context(sql, sql.len(), &cached).unwrap();
    assert_eq!(call.name, "coalesce");
    assert_eq!(call.active_argument, 2);
}

#[test]
fn signature_help_prefers_introspected_active_schema_function() {
    let schema = UiSchemaSummary {
        functions: vec![UiFunctionSummary {
            schema: "public".to_owned(),
            name: "calculate_tax".to_owned(),
            routine_type: "FUNCTION".to_owned(),
            data_type: "numeric".to_owned(),
            definition: String::new(),
            identity_arguments: "amount numeric".to_owned(),
            language: "sql".to_owned(),
            volatility: "stable".to_owned(),
            security_definer: false,
            parameters: vec![UiRoutineParameter {
                name: "amount".to_owned(),
                data_type: "numeric".to_owned(),
                mode: "IN".to_owned(),
                has_default: false,
                default_expr: String::new(),
            }],
        }],
        ..Default::default()
    };
    let index = SchemaSymbolIndex::build(&schema);
    let sql = "SELECT calculate_tax(";
    let cached = tokens(sql, SqlDialect::Postgres);
    let signature = index
        .signature_help(sql, sql.len(), "public", SqlDialect::Postgres, &cached)
        .unwrap();
    assert!(signature.label.starts_with("public.calculate_tax"));
    assert_eq!(signature.active_parameter, 0);
}

#[test]
fn sqlite_and_postgres_builtins_remain_provider_specific() {
    let index = SchemaSymbolIndex::default();
    let sqlite_sql = "SELECT substr(value, ";
    let sqlite_tokens = tokens(sqlite_sql, SqlDialect::SQLite);
    assert!(index
        .signature_help(sqlite_sql, sqlite_sql.len(), "main", SqlDialect::SQLite, &sqlite_tokens,)
        .is_some());

    let postgres_tokens = tokens(sqlite_sql, SqlDialect::Postgres);
    assert!(index
        .signature_help(
            sqlite_sql,
            sqlite_sql.len(),
            "public",
            SqlDialect::Postgres,
            &postgres_tokens,
        )
        .is_none());
}

#[test]
fn rich_hover_resolves_table_columns_and_foreign_keys() {
    let schema = UiSchemaSummary {
        table_details: vec![UiTableSummary {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: Some(150),
            columns: vec![
                UiSchemaColumn {
                    name: "id".to_owned(),
                    data_type: "bigint".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                },
                UiSchemaColumn {
                    name: "user_id".to_owned(),
                    data_type: "bigint".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                },
            ],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_orders_user".to_owned(),
                from_columns: vec!["user_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "users".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        }],
        ..Default::default()
    };
    let index = SchemaSymbolIndex::build(&schema);
    let sql = "SELECT * FROM orders";
    let hover = index
        .rich_hover(sql, (14, 20), "public", SqlDialect::Postgres, &schema)
        .expect("should resolve orders table hover");

    match hover {
        RichHoverHelp::Table {
            qualified_name,
            row_count,
            columns,
            foreign_keys,
            ..
        } => {
            assert_eq!(qualified_name, "public.orders");
            assert_eq!(row_count, Some(150));
            assert_eq!(columns.len(), 2);
            assert!(columns[0].is_primary_key);
            assert!(columns[1].is_foreign_key);
            assert_eq!(foreign_keys.len(), 1);
            assert_eq!(foreign_keys[0].to_table, "users");
        }
        other => panic!("expected RichHoverHelp::Table, got {other:?}"),
    }
}

#[test]
fn rich_hover_resolves_qualified_column_with_fk_target() {
    let schema = UiSchemaSummary {
        table_details: vec![UiTableSummary {
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "user_id".to_owned(),
                data_type: "bigint".to_owned(),
                nullable: false,
                is_primary_key: false,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_orders_user".to_owned(),
                from_columns: vec!["user_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "users".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        }],
        ..Default::default()
    };
    let index = SchemaSymbolIndex::build(&schema);
    let sql = "SELECT orders.user_id FROM orders";
    let hover = index
        .rich_hover(sql, (7, 21), "public", SqlDialect::Postgres, &schema)
        .expect("should resolve column hover");

    match hover {
        RichHoverHelp::Column {
            qualified_name,
            data_type,
            foreign_key_target,
            ..
        } => {
            assert_eq!(qualified_name, "public.orders.user_id");
            assert_eq!(data_type, "bigint");
            assert_eq!(foreign_key_target, Some("public.users(id)".to_owned()));
        }
        other => panic!("expected RichHoverHelp::Column, got {other:?}"),
    }
}

#[test]
fn rich_hover_keyword_returns_documentation_and_dialect_note() {
    let index = SchemaSymbolIndex::default();
    let schema = UiSchemaSummary::default();
    let sql = "SELECT * FROM orders RETURNING id";
    let hover = index
        .rich_hover(sql, (0, 6), "public", SqlDialect::Postgres, &schema)
        .expect("should resolve SELECT keyword");

    match hover {
        RichHoverHelp::Keyword {
            keyword,
            dialect_note,
            documentation,
            example,
        } => {
            assert_eq!(keyword, "SELECT");
            assert_eq!(dialect_note, "PostgreSQL");
            assert!(!documentation.is_empty());
            assert!(example.is_some());
        }
        other => panic!("expected RichHoverHelp::Keyword, got {other:?}"),
    }
}
