use crate::domain::query::{CellValue, QueryParam};
use crate::ports::SqlDialect;

pub use mutation::{build_delete, build_insert, build_select_by_pk, build_update};
pub use read::{build_count, build_select};

#[path = "sql_builder/mutation.rs"]
mod mutation;
#[path = "sql_builder/read.rs"]
mod read;

#[derive(Debug, Clone)]
pub struct TableFilter {
    pub column: String,
    pub op: FilterOp,
    pub value: CellValue,
}

#[derive(Debug, Clone)]
pub enum FilterOp {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone)]
pub struct SortClause {
    pub column: String,
    pub direction: SortDir,
}

#[derive(Debug, Clone)]
pub enum SortDir {
    Asc,
    Desc,
}

struct PlaceholderWriter<'a> {
    dialect: &'a dyn SqlDialect,
    counter: usize,
}

impl<'a> PlaceholderWriter<'a> {
    fn new(dialect: &'a dyn SqlDialect) -> Self {
        Self { dialect, counter: 0 }
    }

    fn next(&mut self) -> String {
        self.counter += 1;
        self.dialect.placeholder(self.counter)
    }
}

pub fn qualify(dialect: &dyn SqlDialect, schema: &str, table: &str) -> String {
    if schema.is_empty() {
        dialect.quote_identifier(table)
    } else {
        format!(
            "{}.{}",
            dialect.quote_identifier(schema),
            dialect.quote_identifier(table)
        )
    }
}

fn cell_to_param(cell: &CellValue) -> QueryParam {
    match cell {
        CellValue::Null => QueryParam::Null,
        CellValue::Bool(value) => QueryParam::Bool(*value),
        CellValue::Int64(value) => QueryParam::Int64(*value),
        CellValue::Float64(value) => QueryParam::Float64(*value),
        CellValue::Decimal(value) => QueryParam::Decimal(value.clone()),
        CellValue::Text(value) => QueryParam::Text(value.clone()),
        CellValue::Bytes(value) => QueryParam::Bytes(value.clone()),
        CellValue::Uuid(value) => QueryParam::Uuid(value.clone()),
        CellValue::DateTime(value) => QueryParam::DateTime(value.clone()),
        // Dedicated temporal values retain the same shape-aware binding semantics
        // as the original builder: no precision-sensitive value is coerced to text.
        CellValue::Timestamp(value) => QueryParam::DateTime(value.clone()),
        CellValue::TimestampTz(value) => QueryParam::DateTime(value.clone()),
        CellValue::TimeTz(value) => QueryParam::Time(value.clone()),
        CellValue::Date(value) => QueryParam::DateTime(value.clone()),
        CellValue::Time(value) => QueryParam::Time(value.clone()),
        CellValue::Interval(value) => QueryParam::Interval(value.clone()),
        CellValue::Inet(value) => QueryParam::Inet(value.clone()),
        CellValue::Json(value) => QueryParam::Json(value.clone()),
    }
}

#[cfg(test)]
mod tests {
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

        let (delete_sql, _) =
            build_delete(&QuestionDialect, "", "users", &["id".into()], &[CellValue::Int64(1)]).unwrap();
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
    fn test_build_select_by_composite_pk() {
        let pk_columns = vec!["tenant_id".into(), "user_id".into()];
        let pk_values = vec![CellValue::Int64(10), CellValue::Text("usr_99".into())];
        let (sql, params) = build_select_by_pk(&DollarNDialect, "public", "accounts", &pk_columns, &pk_values).unwrap();
        assert_eq!(
            sql,
            "SELECT * FROM \"public\".\"accounts\" WHERE \"tenant_id\" = $1 AND \"user_id\" = $2 LIMIT 1"
        );
        assert_eq!(params.len(), 2);
    }
}
