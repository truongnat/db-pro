//! Tests for `../visual_builder.rs` (moved out of the source file).
use super::*;

#[test]
fn generates_join_select() {
    let mut model = VisualQueryModel::default();
    model.add_table("public", "users");
    model.columns.push(BuilderColumn {
        table_alias: "use".into(),
        column: "id".into(),
        alias: None,
        aggregate: None,
    });
    model.joins.push(BuilderJoin {
        table: BuilderTable {
            schema: "public".into(),
            name: "orders".into(),
            alias: "ord".into(),
        },
        join_type: "INNER".into(),
        left_alias: "use".into(),
        left_column: "id".into(),
        right_alias: "ord".into(),
        right_column: "user_id".into(),
    });
    model.where_clauses.push(BuilderPredicate {
        left_alias: "use".into(),
        left_column: "active".into(),
        op: "=".into(),
        value: "true".into(),
        is_null_check: false,
    });
    model.limit = Some(50);
    let sql = model.generate_sql(BuilderDialect::Postgres).unwrap();
    assert!(sql.contains("FROM \"public\".\"users\" AS \"use\""));
    assert!(sql.contains("INNER JOIN \"public\".\"orders\" AS \"ord\""));
    assert!(sql.contains("LIMIT 50"));
}

#[test]
fn round_trips_supported_select() {
    let sql = r#"
SELECT "u"."id", COUNT("o"."id") AS "order_count"
FROM "public"."users" AS "u"
INNER JOIN "public"."orders" AS "o" ON "u"."id" = "o"."user_id"
WHERE "u"."active" = true
GROUP BY "u"."id"
ORDER BY "u"."id" ASC
LIMIT 10;
"#;
    let model = try_import_select(sql, BuilderDialect::Postgres).unwrap();
    assert_eq!(model.tables.len(), 1);
    assert_eq!(model.joins.len(), 1);
    assert_eq!(model.columns.len(), 2);
    assert_eq!(model.limit, Some(10));
    let again = model.generate_sql(BuilderDialect::Postgres).unwrap();
    let model2 = try_import_select(&again, BuilderDialect::Postgres).unwrap();
    assert_eq!(model2.tables[0].name, "users");
    assert_eq!(model2.joins[0].table.name, "orders");
}

#[test]
fn rejects_union() {
    let err = try_import_select("SELECT 1 UNION SELECT 2", BuilderDialect::Postgres).unwrap_err();
    assert!(err.to_ascii_lowercase().contains("union") || err.contains("not supported"));
}
