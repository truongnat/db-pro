//! Tests for `../sql_builder.rs` (moved out of the source file).
use super::*;
use crate::domain::error::DbError;

struct QuestionDialect;
impl SqlDialect for QuestionDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

struct DollarNDialect;
impl SqlDialect for DollarNDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

#[test]
fn empty_schema_omits_schema_prefix() {
    let (select_sql, _) = build_select(&QuestionDialect, "", "users", &[], &[], 50, 0).unwrap();
    assert_eq!(select_sql, r#"SELECT * FROM "users" LIMIT ? OFFSET ?"#);

    let (count_sql, _) = build_count(&QuestionDialect, "", "users", &[]);
    assert_eq!(count_sql, r#"SELECT COUNT(*) FROM "users""#);

    let (insert_sql, _) = build_insert(
        &QuestionDialect,
        "",
        "users",
        &["name".into()],
        &[CellValue::Text("alice".into())],
    )
    .unwrap();
    assert_eq!(insert_sql, r#"INSERT INTO "users" ("name") VALUES (?)"#);

    let (update_sql, _) = build_update(
        &QuestionDialect,
        "",
        "users",
        &["name".into()],
        &[CellValue::Text("bob".into())],
        &["id".into()],
        &[CellValue::Int64(1)],
    )
    .unwrap();
    assert_eq!(update_sql, r#"UPDATE "users" SET "name" = ? WHERE "id" = ?"#);

    let (delete_sql, _) = build_delete(&QuestionDialect, "", "users", &["id".into()], &[CellValue::Int64(1)]).unwrap();
    assert_eq!(delete_sql, r#"DELETE FROM "users" WHERE "id" = ?"#);
}

#[test]
fn select_no_filters_no_sorts() {
    let (sql, params) = build_select(&QuestionDialect, "public", "users", &[], &[], 50, 0).unwrap();
    assert_eq!(sql, r#"SELECT * FROM "public"."users" LIMIT ? OFFSET ?"#);
    assert_eq!(params.len(), 2);
}

#[test]
fn select_with_filters_and_sorts() {
    let filters = vec![
        TableFilter {
            column: "name".into(),
            op: FilterOp::Eq,
            value: CellValue::Text("alice".into()),
        },
        TableFilter {
            column: "age".into(),
            op: FilterOp::Gte,
            value: CellValue::Int64(18),
        },
    ];
    let sorts = vec![SortClause {
        column: "name".into(),
        direction: SortDir::Asc,
    }];
    let (sql, params) = build_select(&QuestionDialect, "public", "users", &filters, &sorts, 25, 50).unwrap();
    assert_eq!(
        sql,
        r#"SELECT * FROM "public"."users" WHERE "name" = ? AND "age" >= ? ORDER BY "name" ASC LIMIT ? OFFSET ?"#
    );
    assert_eq!(params.len(), 4);
}

#[test]
fn select_with_null_filters() {
    let filters = vec![TableFilter {
        column: "email".into(),
        op: FilterOp::IsNotNull,
        value: CellValue::Null,
    }];
    let (sql, params) = build_select(&QuestionDialect, "public", "users", &filters, &[], 50, 0).unwrap();
    assert_eq!(
        sql,
        r#"SELECT * FROM "public"."users" WHERE "email" IS NOT NULL LIMIT ? OFFSET ?"#
    );
    assert_eq!(params.len(), 2);
}

#[test]
fn select_rejects_pagination_values_outside_database_integer_range() {
    let result = build_select(&QuestionDialect, "public", "users", &[], &[], u64::MAX, 0);
    assert!(matches!(result, Err(DbError::Validation(message)) if message.contains("limit")));

    let result = build_select(&QuestionDialect, "public", "users", &[], &[], 50, u64::MAX);
    assert!(matches!(result, Err(DbError::Validation(message)) if message.contains("offset")));
}

#[test]
fn count_with_filter() {
    let filters = vec![TableFilter {
        column: "active".into(),
        op: FilterOp::Eq,
        value: CellValue::Bool(true),
    }];
    let (sql, params) = build_count(&QuestionDialect, "public", "users", &filters);
    assert_eq!(sql, r#"SELECT COUNT(*) FROM "public"."users" WHERE "active" = ?"#);
    assert_eq!(params.len(), 1);
}

#[test]
fn insert_basic() {
    let columns = vec!["name".into(), "email".into()];
    let values = vec![CellValue::Text("bob".into()), CellValue::Text("bob@test.com".into())];
    let (sql, params) = build_insert(&QuestionDialect, "public", "users", &columns, &values).unwrap();
    assert_eq!(sql, r#"INSERT INTO "public"."users" ("name", "email") VALUES (?, ?)"#);
    assert_eq!(params.len(), 2);
}

#[test]
fn date_cell_uses_typed_date_parameter() {
    let filter = TableFilter {
        column: "hire_date".into(),
        op: FilterOp::Eq,
        value: CellValue::Date("2026-08-17".into()),
    };
    let (_, params) = build_count(&DollarNDialect, "public", "employees", &[filter]);

    assert!(matches!(params.as_slice(), [QueryParam::DateTime(value)] if value == "2026-08-17"));
}

/// A2 (#52): each temporal class keeps its own variant and still binds through a
/// typed parameter — never through TEXT, which would let the server reinterpret
/// the value.
#[test]
fn dedicated_temporal_cells_bind_as_typed_parameters() {
    let filters = vec![
        TableFilter {
            column: "created_at".into(),
            op: FilterOp::Eq,
            value: CellValue::Timestamp("2024-03-15T10:20:30.123456".into()),
        },
        TableFilter {
            column: "expires_at".into(),
            op: FilterOp::Eq,
            value: CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".into()),
        },
        TableFilter {
            column: "opens_at".into(),
            op: FilterOp::Eq,
            value: CellValue::TimeTz("10:20:30.123456+07:00".into()),
        },
    ];

    let (_, params) = build_count(&DollarNDialect, "public", "events", &filters);

    assert!(matches!(
        params.as_slice(),
        [
            QueryParam::DateTime(timestamp),
            QueryParam::DateTime(instant),
            QueryParam::Time(with_offset)
        ] if timestamp == "2024-03-15T10:20:30.123456"
            && instant == "2024-03-15T10:20:30.123456Z"
            && with_offset == "10:20:30.123456+07:00"
    ));
}

#[test]
fn temporal_and_network_cells_use_typed_parameters() {
    let filters = vec![
        TableFilter {
            column: "start_time".into(),
            op: FilterOp::Eq,
            value: CellValue::Time("12:34:56.123456".into()),
        },
        TableFilter {
            column: "duration".into(),
            op: FilterOp::Eq,
            value: CellValue::Interval("1 days 02:00:00".into()),
        },
        TableFilter {
            column: "address".into(),
            op: FilterOp::Eq,
            value: CellValue::Inet("192.0.2.1/24".into()),
        },
    ];

    let (_, params) = build_count(&DollarNDialect, "public", "events", &filters);

    assert!(matches!(
        params.as_slice(),
        [
            QueryParam::Time(time),
            QueryParam::Interval(interval),
            QueryParam::Inet(address)
        ] if time == "12:34:56.123456"
            && interval == "1 days 02:00:00"
            && address == "192.0.2.1/24"
    ));
}

#[test]
fn update_with_pk() {
    let columns = vec!["name".into()];
    let values = vec![CellValue::Text("alice2".into())];
    let pk_columns = vec!["id".into()];
    let pk_values = vec![CellValue::Int64(1)];
    let (sql, params) = build_update(
        &QuestionDialect,
        "public",
        "users",
        &columns,
        &values,
        &pk_columns,
        &pk_values,
    )
    .unwrap();
    assert_eq!(sql, r#"UPDATE "public"."users" SET "name" = ? WHERE "id" = ?"#);
    assert_eq!(params.len(), 2);
}

#[test]
fn delete_with_composite_pk() {
    let pk_columns = vec!["order_id".into(), "product_id".into()];
    let pk_values = vec![CellValue::Int64(10), CellValue::Int64(20)];
    let (sql, params) = build_delete(&QuestionDialect, "public", "order_items", &pk_columns, &pk_values).unwrap();
    assert_eq!(
        sql,
        r#"DELETE FROM "public"."order_items" WHERE "order_id" = ? AND "product_id" = ?"#
    );
    assert_eq!(params.len(), 2);
}

#[test]
fn quote_identifier_special_chars() {
    let d = QuestionDialect;
    assert_eq!(d.quote_identifier("table"), r#""table""#);
    assert_eq!(d.quote_identifier("my table"), r#""my table""#);
    assert_eq!(d.quote_identifier(r#"has"quote"#), r#""has""quote""#);
}

#[test]
fn multi_sort() {
    let sorts = vec![
        SortClause {
            column: "last_name".into(),
            direction: SortDir::Asc,
        },
        SortClause {
            column: "first_name".into(),
            direction: SortDir::Desc,
        },
    ];
    let (sql, _) = build_select(&QuestionDialect, "public", "users", &[], &sorts, 50, 0).unwrap();
    assert!(sql.contains(r#"ORDER BY "last_name" ASC, "first_name" DESC"#));
}

#[test]
fn like_filter() {
    let filters = vec![TableFilter {
        column: "name".into(),
        op: FilterOp::Like,
        value: CellValue::Text("%alice%".into()),
    }];
    let (sql, params) = build_select(&QuestionDialect, "public", "users", &filters, &[], 50, 0).unwrap();
    assert!(sql.contains(r#"CAST("name" AS TEXT) LIKE ?"#));
    assert_eq!(params.len(), 3);
}

#[test]
fn dollar_n_select_with_filters() {
    let filters = vec![TableFilter {
        column: "name".into(),
        op: FilterOp::Eq,
        value: CellValue::Text("alice".into()),
    }];
    let (sql, params) = build_select(&DollarNDialect, "public", "users", &filters, &[], 25, 50).unwrap();
    assert_eq!(
        sql,
        r#"SELECT * FROM "public"."users" WHERE "name" = $1 LIMIT $2 OFFSET $3"#
    );
    assert_eq!(params.len(), 3);
}

#[test]
fn dollar_n_insert() {
    let columns = vec!["name".into(), "email".into()];
    let values = vec![CellValue::Text("bob".into()), CellValue::Text("bob@test.com".into())];
    let (sql, params) = build_insert(&DollarNDialect, "public", "users", &columns, &values).unwrap();
    assert_eq!(sql, r#"INSERT INTO "public"."users" ("name", "email") VALUES ($1, $2)"#);
    assert_eq!(params.len(), 2);
}

#[test]
fn dollar_n_update_with_pk() {
    let columns = vec!["name".into()];
    let values = vec![CellValue::Text("alice2".into())];
    let pk_columns = vec!["id".into()];
    let pk_values = vec![CellValue::Int64(1)];
    let (sql, params) = build_update(
        &DollarNDialect,
        "public",
        "users",
        &columns,
        &values,
        &pk_columns,
        &pk_values,
    )
    .unwrap();
    assert_eq!(sql, r#"UPDATE "public"."users" SET "name" = $1 WHERE "id" = $2"#);
    assert_eq!(params.len(), 2);
}

#[test]
fn dollar_n_delete() {
    let pk_columns = vec!["order_id".into(), "product_id".into()];
    let pk_values = vec![CellValue::Int64(10), CellValue::Int64(20)];
    let (sql, params) = build_delete(&DollarNDialect, "public", "order_items", &pk_columns, &pk_values).unwrap();
    assert_eq!(
        sql,
        r#"DELETE FROM "public"."order_items" WHERE "order_id" = $1 AND "product_id" = $2"#
    );
    assert_eq!(params.len(), 2);
}

#[test]
fn insert_rejects_column_value_mismatch() {
    let columns = vec!["name".into(), "email".into()];
    let values = vec![CellValue::Text("bob".into())];
    let result = build_insert(&QuestionDialect, "public", "users", &columns, &values);
    assert!(result.is_err());
}

#[test]
fn insert_rejects_empty_columns() {
    let result = build_insert(&QuestionDialect, "public", "users", &[], &[]);
    assert!(matches!(result, Err(DbError::Validation(message)) if message.contains("at least one column")));
}

#[test]
fn update_rejects_column_value_mismatch() {
    let columns = vec!["name".into(), "email".into()];
    let values = vec![CellValue::Text("bob".into())];
    let result = build_update(
        &QuestionDialect,
        "public",
        "users",
        &columns,
        &values,
        &["id".into()],
        &[CellValue::Int64(1)],
    );
    assert!(result.is_err());
}

#[test]
fn update_rejects_empty_columns() {
    let result = build_update(
        &QuestionDialect,
        "public",
        "users",
        &[],
        &[],
        &["id".into()],
        &[CellValue::Int64(1)],
    );
    assert!(matches!(result, Err(DbError::Validation(message)) if message.contains("at least one column")));
}

#[test]
fn delete_rejects_empty_pk() {
    let result = build_delete(&QuestionDialect, "public", "users", &[], &[]);
    assert!(result.is_err());
}

#[test]
fn delete_rejects_pk_length_mismatch() {
    let result = build_delete(
        &QuestionDialect,
        "public",
        "users",
        &["id".into(), "org_id".into()],
        &[CellValue::Int64(1)],
    );
    assert!(result.is_err());
}

#[test]
fn update_rejects_empty_pk() {
    let result = build_update(
        &QuestionDialect,
        "public",
        "users",
        &["name".into()],
        &[CellValue::Text("x".into())],
        &[],
        &[],
    );
    assert!(result.is_err());
}

#[test]
fn update_rejects_pk_length_mismatch() {
    let result = build_update(
        &QuestionDialect,
        "public",
        "users",
        &["name".into()],
        &[CellValue::Text("x".into())],
        &["id".into(), "org_id".into()],
        &[CellValue::Int64(1)],
    );
    assert!(result.is_err());
}

struct MySqlDialect;
impl SqlDialect for MySqlDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('`', "``");
        format!("`{escaped}`")
    }
}

struct SqlServerDialect;
impl SqlDialect for SqlServerDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("@p{index}")
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace(']', "]]");
        format!("[{escaped}]")
    }
    fn pagination_clause(&self, limit_ph: &str, offset_ph: &str) -> String {
        format!(" OFFSET {offset_ph} ROWS FETCH NEXT {limit_ph} ROWS ONLY")
    }
    fn pagination_requires_order_by(&self) -> bool {
        true
    }
}

#[test]
fn mysql_select_uses_backtick_quoting() {
    let sorts = vec![SortClause {
        column: "id".into(),
        direction: SortDir::Asc,
    }];
    let (sql, params) = build_select(&MySqlDialect, "dbo", "users", &[], &sorts, 50, 0).unwrap();
    assert_eq!(sql, "SELECT * FROM `dbo`.`users` ORDER BY `id` ASC LIMIT ? OFFSET ?");
    assert_eq!(params.len(), 2);
}

#[test]
fn mysql_insert() {
    let columns = vec!["name".into(), "email".into()];
    let values = vec![CellValue::Text("bob".into()), CellValue::Text("bob@test.com".into())];
    let (sql, params) = build_insert(&MySqlDialect, "dbo", "users", &columns, &values).unwrap();
    assert_eq!(sql, "INSERT INTO `dbo`.`users` (`name`, `email`) VALUES (?, ?)");
    assert_eq!(params.len(), 2);
}

#[test]
fn mysql_update_with_pk() {
    let columns = vec!["name".into()];
    let values = vec![CellValue::Text("alice2".into())];
    let pk_columns = vec!["id".into()];
    let pk_values = vec![CellValue::Int64(1)];
    let (sql, params) = build_update(
        &MySqlDialect,
        "dbo",
        "users",
        &columns,
        &values,
        &pk_columns,
        &pk_values,
    )
    .unwrap();
    assert_eq!(sql, "UPDATE `dbo`.`users` SET `name` = ? WHERE `id` = ?");
    assert_eq!(params.len(), 2);
}

#[test]
fn mysql_delete() {
    let pk_columns = vec!["id".into()];
    let pk_values = vec![CellValue::Int64(1)];
    let (sql, params) = build_delete(&MySqlDialect, "dbo", "users", &pk_columns, &pk_values).unwrap();
    assert_eq!(sql, "DELETE FROM `dbo`.`users` WHERE `id` = ?");
    assert_eq!(params.len(), 1);
}

#[test]
fn mysql_quoting_special_chars() {
    let d = MySqlDialect;
    assert_eq!(d.quote_identifier("table"), "`table`");
    assert_eq!(d.quote_identifier("has`tick"), "`has``tick`");
}

#[test]
fn sqlserver_select_with_order_by() {
    let sorts = vec![SortClause {
        column: "id".into(),
        direction: SortDir::Asc,
    }];
    let (sql, params) = build_select(&SqlServerDialect, "dbo", "users", &[], &sorts, 50, 0).unwrap();
    assert_eq!(
        sql,
        "SELECT * FROM [dbo].[users] ORDER BY [id] ASC OFFSET @p2 ROWS FETCH NEXT @p1 ROWS ONLY"
    );
    assert_eq!(params.len(), 2);
}

#[test]
fn sqlserver_select_without_order_by_fails() {
    let result = build_select(&SqlServerDialect, "dbo", "users", &[], &[], 50, 0);
    assert!(matches!(result, Err(DbError::Validation(_))));
}

#[test]
fn sqlserver_quoting_special_chars() {
    let d = SqlServerDialect;
    assert_eq!(d.quote_identifier("table"), "[table]");
    assert_eq!(d.quote_identifier("has]bracket"), "[has]]bracket]");
}

#[test]
fn select_by_composite_pk_uses_each_provider_pagination() {
    let pk_columns = vec!["tenant_id".into(), "user_id".into()];
    let pk_values = vec![CellValue::Int64(10), CellValue::Text("usr_99' OR 1=1 --".into())];
    for (driver, dialect, schema, expected) in [
        (
            "postgres",
            &DollarNDialect as &dyn SqlDialect,
            "public",
            "SELECT * FROM \"public\".\"accounts\" WHERE \"tenant_id\" = $1 AND \"user_id\" = $2 LIMIT 1 OFFSET 0",
        ),
        (
            "sqlite",
            &QuestionDialect,
            "",
            "SELECT * FROM \"accounts\" WHERE \"tenant_id\" = ? AND \"user_id\" = ? LIMIT 1 OFFSET 0",
        ),
        (
            "mysql",
            &MySqlDialect,
            "app",
            "SELECT * FROM `app`.`accounts` WHERE `tenant_id` = ? AND `user_id` = ? LIMIT 1 OFFSET 0",
        ),
        (
            "sqlserver",
            &SqlServerDialect,
            "dbo",
            concat!(
                "SELECT * FROM [dbo].[accounts] WHERE [tenant_id] = @p1 AND [user_id] = @p2 ",
                "ORDER BY [tenant_id], [user_id] OFFSET 0 ROWS FETCH NEXT 1 ROWS ONLY"
            ),
        ),
    ] {
        let (sql, params) = build_select_by_pk(dialect, schema, "accounts", &pk_columns, &pk_values).unwrap();
        assert_eq!(sql, expected, "{driver}");
        assert_eq!(params.len(), 2, "{driver}");
        assert!(matches!(params[0], QueryParam::Int64(10)), "{driver}");
        assert!(
            matches!(&params[1], QueryParam::Text(value) if value == "usr_99' OR 1=1 --"),
            "{driver}"
        );
    }
}

#[test]
fn select_by_pk_rejects_missing_or_mismatched_keys() {
    for (columns, values) in [
        (vec![], vec![]),
        (vec!["tenant_id".into(), "user_id".into()], vec![CellValue::Int64(10)]),
        (vec!["id".into()], vec![]),
    ] {
        assert!(matches!(
            build_select_by_pk(&SqlServerDialect, "dbo", "accounts", &columns, &values),
            Err(DbError::Validation(_))
        ));
    }
}
