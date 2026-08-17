use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryParam};
use crate::ports::SqlDialect;

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

pub fn build_select(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
    sorts: &[SortClause],
    limit: u64,
    offset: u64,
) -> Result<(String, Vec<QueryParam>), DbError> {
    if dialect.pagination_requires_order_by() && sorts.is_empty() {
        return Err(DbError::Validation("pagination requires an ORDER BY clause".into()));
    }
    let (where_clause, params) = build_where(dialect, filters);
    let order_clause = build_order(dialect, sorts);
    let mut pw = PlaceholderWriter::new(dialect);
    pw.counter = params.len();
    let limit_ph = pw.next();
    let offset_ph = pw.next();

    let pagination = dialect.pagination_clause(&limit_ph, &offset_ph);

    let sql = format!(
        "SELECT * FROM {}.{}{}{}{pagination}",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(table),
        where_clause,
        order_clause,
    );

    let mut all_params = params;
    all_params.push(QueryParam::Int64(limit as i64));
    all_params.push(QueryParam::Int64(offset as i64));

    Ok((sql, all_params))
}

pub fn build_count(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> (String, Vec<QueryParam>) {
    let (where_clause, params) = build_where(dialect, filters);

    let sql = format!(
        "SELECT COUNT(*) FROM {}.{}{}",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(table),
        where_clause,
    );

    (sql, params)
}

pub fn build_insert(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    if columns.len() != values.len() {
        return Err(DbError::Validation(format!(
            "column count ({}) does not match value count ({})",
            columns.len(),
            values.len()
        )));
    }
    let cols = columns
        .iter()
        .map(|c| dialect.quote_identifier(c))
        .collect::<Vec<_>>()
        .join(", ");
    let mut pw = PlaceholderWriter::new(dialect);
    let placeholders = values.iter().map(|_| pw.next()).collect::<Vec<_>>().join(", ");

    let sql = format!(
        "INSERT INTO {}.{} ({}) VALUES ({})",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(table),
        cols,
        placeholders,
    );

    let params = values.iter().map(cell_to_param).collect();
    Ok((sql, params))
}

pub fn build_update(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    if columns.len() != values.len() {
        return Err(DbError::Validation(format!(
            "column count ({}) does not match value count ({})",
            columns.len(),
            values.len()
        )));
    }
    if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
        return Err(DbError::Validation(format!(
            "pk column count ({}) does not match pk value count ({})",
            pk_columns.len(),
            pk_values.len()
        )));
    }
    let mut pw = PlaceholderWriter::new(dialect);
    let set_parts: Vec<String> = columns
        .iter()
        .map(|c| format!("{} = {}", dialect.quote_identifier(c), pw.next()))
        .collect();
    let pk_where: Vec<String> = pk_columns
        .iter()
        .map(|c| format!("{} = {}", dialect.quote_identifier(c), pw.next()))
        .collect();

    let sql = format!(
        "UPDATE {}.{} SET {} WHERE {}",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(table),
        set_parts.join(", "),
        pk_where.join(" AND "),
    );

    let mut params: Vec<QueryParam> = values.iter().map(cell_to_param).collect();
    params.extend(pk_values.iter().map(cell_to_param));
    Ok((sql, params))
}

pub fn build_delete(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
        return Err(DbError::Validation(format!(
            "pk column count ({}) does not match pk value count ({})",
            pk_columns.len(),
            pk_values.len()
        )));
    }
    let mut pw = PlaceholderWriter::new(dialect);
    let pk_where: Vec<String> = pk_columns
        .iter()
        .map(|c| format!("{} = {}", dialect.quote_identifier(c), pw.next()))
        .collect();

    let sql = format!(
        "DELETE FROM {}.{} WHERE {}",
        dialect.quote_identifier(schema),
        dialect.quote_identifier(table),
        pk_where.join(" AND "),
    );

    let params = pk_values.iter().map(cell_to_param).collect();
    Ok((sql, params))
}

fn build_where(dialect: &dyn SqlDialect, filters: &[TableFilter]) -> (String, Vec<QueryParam>) {
    if filters.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut conditions = Vec::with_capacity(filters.len());
    let mut params = Vec::new();
    let mut pw = PlaceholderWriter::new(dialect);

    for f in filters {
        let col = dialect.quote_identifier(&f.column);
        match f.op {
            FilterOp::IsNull => conditions.push(format!("{col} IS NULL")),
            FilterOp::IsNotNull => conditions.push(format!("{col} IS NOT NULL")),
            FilterOp::Eq => {
                conditions.push(format!("{col} = {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Neq => {
                conditions.push(format!("{col} != {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Lt => {
                conditions.push(format!("{col} < {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Lte => {
                conditions.push(format!("{col} <= {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Gt => {
                conditions.push(format!("{col} > {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Gte => {
                conditions.push(format!("{col} >= {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
            FilterOp::Like => {
                conditions.push(format!("{col} LIKE {}", pw.next()));
                params.push(cell_to_param(&f.value));
            }
        }
    }

    (format!(" WHERE {}", conditions.join(" AND ")), params)
}

fn build_order(dialect: &dyn SqlDialect, sorts: &[SortClause]) -> String {
    if sorts.is_empty() {
        return String::new();
    }

    let parts: Vec<String> = sorts
        .iter()
        .map(|s| {
            let dir = match s.direction {
                SortDir::Asc => "ASC",
                SortDir::Desc => "DESC",
            };
            format!("{} {dir}", dialect.quote_identifier(&s.column))
        })
        .collect();

    format!(" ORDER BY {}", parts.join(", "))
}

fn cell_to_param(cell: &CellValue) -> QueryParam {
    match cell {
        CellValue::Null => QueryParam::Null,
        CellValue::Bool(v) => QueryParam::Bool(*v),
        CellValue::Int64(v) => QueryParam::Int64(*v),
        CellValue::Float64(v) => QueryParam::Float64(*v),
        CellValue::Decimal(v) => QueryParam::Decimal(v.clone()),
        CellValue::Text(v) => QueryParam::Text(v.clone()),
        CellValue::Bytes(v) => QueryParam::Bytes(v.clone()),
        CellValue::Uuid(v) => QueryParam::Uuid(v.clone()),
        CellValue::DateTime(v) => QueryParam::DateTime(v.clone()),
        CellValue::Json(v) => QueryParam::Json(v.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(sql.contains(r#""name" LIKE ?"#));
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
}
