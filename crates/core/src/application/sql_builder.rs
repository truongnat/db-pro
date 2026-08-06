use crate::domain::query::{CellValue, PlaceholderStyle, QueryParam};

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

struct PlaceholderWriter {
    style: PlaceholderStyle,
    counter: u32,
}

impl PlaceholderWriter {
    fn new(style: PlaceholderStyle) -> Self {
        Self { style, counter: 0 }
    }

    fn next(&mut self) -> String {
        match self.style {
            PlaceholderStyle::Question => "?".to_string(),
            PlaceholderStyle::DollarN => {
                self.counter += 1;
                format!("${}", self.counter)
            }
        }
    }
}

pub fn build_select(
    style: PlaceholderStyle,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
    sorts: &[SortClause],
    limit: u64,
    offset: u64,
) -> (String, Vec<QueryParam>) {
    let (where_clause, params) = build_where(style, filters);
    let order_clause = build_order(sorts);
    let mut pw = PlaceholderWriter::new(style);
    pw.counter = params.len() as u32;
    let limit_ph = pw.next();
    let offset_ph = pw.next();

    let sql = format!(
        "SELECT * FROM {}.{}{}{} LIMIT {} OFFSET {}",
        quote_ident(schema),
        quote_ident(table),
        where_clause,
        order_clause,
        limit_ph,
        offset_ph,
    );

    let mut all_params = params;
    all_params.push(QueryParam::Int64(limit as i64));
    all_params.push(QueryParam::Int64(offset as i64));

    (sql, all_params)
}

pub fn build_count(
    style: PlaceholderStyle,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> (String, Vec<QueryParam>) {
    let (where_clause, params) = build_where(style, filters);

    let sql = format!(
        "SELECT COUNT(*) FROM {}.{}{}",
        quote_ident(schema),
        quote_ident(table),
        where_clause,
    );

    (sql, params)
}

pub fn build_insert(
    style: PlaceholderStyle,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
) -> (String, Vec<QueryParam>) {
    let cols = columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
    let mut pw = PlaceholderWriter::new(style);
    let placeholders = values.iter().map(|_| pw.next()).collect::<Vec<_>>().join(", ");

    let sql = format!(
        "INSERT INTO {}.{} ({}) VALUES ({})",
        quote_ident(schema),
        quote_ident(table),
        cols,
        placeholders,
    );

    let params = values.iter().map(cell_to_param).collect();
    (sql, params)
}

pub fn build_update(
    style: PlaceholderStyle,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> (String, Vec<QueryParam>) {
    let mut pw = PlaceholderWriter::new(style);
    let set_parts: Vec<String> = columns.iter().map(|c| format!("{} = {}", quote_ident(c), pw.next())).collect();
    let pk_where: Vec<String> = pk_columns.iter().map(|c| format!("{} = {}", quote_ident(c), pw.next())).collect();

    let sql = format!(
        "UPDATE {}.{} SET {} WHERE {}",
        quote_ident(schema),
        quote_ident(table),
        set_parts.join(", "),
        pk_where.join(" AND "),
    );

    let mut params: Vec<QueryParam> = values.iter().map(cell_to_param).collect();
    params.extend(pk_values.iter().map(cell_to_param));
    (sql, params)
}

pub fn build_delete(
    style: PlaceholderStyle,
    schema: &str,
    table: &str,
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> (String, Vec<QueryParam>) {
    let mut pw = PlaceholderWriter::new(style);
    let pk_where: Vec<String> = pk_columns.iter().map(|c| format!("{} = {}", quote_ident(c), pw.next())).collect();

    let sql = format!(
        "DELETE FROM {}.{} WHERE {}",
        quote_ident(schema),
        quote_ident(table),
        pk_where.join(" AND "),
    );

    let params = pk_values.iter().map(cell_to_param).collect();
    (sql, params)
}

fn build_where(style: PlaceholderStyle, filters: &[TableFilter]) -> (String, Vec<QueryParam>) {
    if filters.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut conditions = Vec::with_capacity(filters.len());
    let mut params = Vec::new();
    let mut pw = PlaceholderWriter::new(style);

    for f in filters {
        let col = quote_ident(&f.column);
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

fn build_order(sorts: &[SortClause]) -> String {
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
            format!("{} {dir}", quote_ident(&s.column))
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
        CellValue::Text(v) => QueryParam::Text(v.clone()),
        CellValue::Bytes(v) => QueryParam::Bytes(v.clone()),
        CellValue::Uuid(v) => QueryParam::Uuid(v.clone()),
        CellValue::DateTime(v) => QueryParam::DateTime(v.clone()),
        CellValue::Json(v) => QueryParam::Json(v.clone()),
    }
}

fn quote_ident(name: &str) -> String {
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_no_filters_no_sorts() {
        let (sql, params) = build_select(PlaceholderStyle::Question, "public", "users", &[], &[], 50, 0);
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
        let (sql, params) = build_select(PlaceholderStyle::Question, "public", "users", &filters, &sorts, 25, 50);
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
        let (sql, params) = build_select(PlaceholderStyle::Question, "public", "users", &filters, &[], 50, 0);
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
        let (sql, params) = build_count(PlaceholderStyle::Question, "public", "users", &filters);
        assert_eq!(sql, r#"SELECT COUNT(*) FROM "public"."users" WHERE "active" = ?"#);
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn insert_basic() {
        let columns = vec!["name".into(), "email".into()];
        let values = vec![CellValue::Text("bob".into()), CellValue::Text("bob@test.com".into())];
        let (sql, params) = build_insert(PlaceholderStyle::Question, "public", "users", &columns, &values);
        assert_eq!(
            sql,
            r#"INSERT INTO "public"."users" ("name", "email") VALUES (?, ?)"#
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn update_with_pk() {
        let columns = vec!["name".into()];
        let values = vec![CellValue::Text("alice2".into())];
        let pk_columns = vec!["id".into()];
        let pk_values = vec![CellValue::Int64(1)];
        let (sql, params) = build_update(PlaceholderStyle::Question, "public", "users", &columns, &values, &pk_columns, &pk_values);
        assert_eq!(
            sql,
            r#"UPDATE "public"."users" SET "name" = ? WHERE "id" = ?"#
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn delete_with_composite_pk() {
        let pk_columns = vec!["order_id".into(), "product_id".into()];
        let pk_values = vec![CellValue::Int64(10), CellValue::Int64(20)];
        let (sql, params) = build_delete(PlaceholderStyle::Question, "public", "order_items", &pk_columns, &pk_values);
        assert_eq!(
            sql,
            r#"DELETE FROM "public"."order_items" WHERE "order_id" = ? AND "product_id" = ?"#
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn quote_ident_special_chars() {
        assert_eq!(quote_ident("table"), r#""table""#);
        assert_eq!(quote_ident("my table"), r#""my table""#);
        assert_eq!(quote_ident(r#"has"quote"#), r#""has""quote""#);
    }

    #[test]
    fn multi_sort() {
        let sorts = vec![
            SortClause { column: "last_name".into(), direction: SortDir::Asc },
            SortClause { column: "first_name".into(), direction: SortDir::Desc },
        ];
        let (sql, _) = build_select(PlaceholderStyle::Question, "public", "users", &[], &sorts, 50, 0);
        assert!(sql.contains(r#"ORDER BY "last_name" ASC, "first_name" DESC"#));
    }

    #[test]
    fn like_filter() {
        let filters = vec![TableFilter {
            column: "name".into(),
            op: FilterOp::Like,
            value: CellValue::Text("%alice%".into()),
        }];
        let (sql, params) = build_select(PlaceholderStyle::Question, "public", "users", &filters, &[], 50, 0);
        assert!(sql.contains(r#""name" LIKE ?"#));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn dollar_n_select_with_filters() {
        let filters = vec![
            TableFilter {
                column: "name".into(),
                op: FilterOp::Eq,
                value: CellValue::Text("alice".into()),
            },
        ];
        let (sql, params) = build_select(PlaceholderStyle::DollarN, "public", "users", &filters, &[], 25, 50);
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
        let (sql, params) = build_insert(PlaceholderStyle::DollarN, "public", "users", &columns, &values);
        assert_eq!(
            sql,
            r#"INSERT INTO "public"."users" ("name", "email") VALUES ($1, $2)"#
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn dollar_n_update_with_pk() {
        let columns = vec!["name".into()];
        let values = vec![CellValue::Text("alice2".into())];
        let pk_columns = vec!["id".into()];
        let pk_values = vec![CellValue::Int64(1)];
        let (sql, params) = build_update(PlaceholderStyle::DollarN, "public", "users", &columns, &values, &pk_columns, &pk_values);
        assert_eq!(
            sql,
            r#"UPDATE "public"."users" SET "name" = $1 WHERE "id" = $2"#
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn dollar_n_delete() {
        let pk_columns = vec!["order_id".into(), "product_id".into()];
        let pk_values = vec![CellValue::Int64(10), CellValue::Int64(20)];
        let (sql, params) = build_delete(PlaceholderStyle::DollarN, "public", "order_items", &pk_columns, &pk_values);
        assert_eq!(
            sql,
            r#"DELETE FROM "public"."order_items" WHERE "order_id" = $1 AND "product_id" = $2"#
        );
        assert_eq!(params.len(), 2);
    }
}
