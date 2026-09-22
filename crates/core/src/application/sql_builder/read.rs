use crate::domain::error::DbError;
use crate::domain::query::QueryParam;
use crate::ports::SqlDialect;

use super::{cell_to_param, qualify, FilterOp, PlaceholderWriter, SortClause, SortDir, TableFilter};

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
    let limit = i64::try_from(limit).map_err(|_| DbError::Validation("limit exceeds database integer range".into()))?;
    let offset =
        i64::try_from(offset).map_err(|_| DbError::Validation("offset exceeds database integer range".into()))?;
    let (where_clause, params) = build_where(dialect, filters);
    let order_clause = build_order(dialect, sorts);
    let mut pw = PlaceholderWriter::new(dialect);
    pw.counter = params.len();
    let limit_ph = pw.next();
    let offset_ph = pw.next();

    let pagination = dialect.pagination_clause(&limit_ph, &offset_ph);
    let target = qualify(dialect, schema, table);

    let sql = format!("SELECT * FROM {target}{}{}{pagination}", where_clause, order_clause,);

    let mut all_params = params;
    all_params.push(QueryParam::Int64(limit));
    all_params.push(QueryParam::Int64(offset));

    Ok((sql, all_params))
}

pub fn build_count(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> (String, Vec<QueryParam>) {
    let (where_clause, params) = build_where(dialect, filters);
    let target = qualify(dialect, schema, table);

    let sql = format!("SELECT COUNT(*) FROM {target}{}", where_clause,);

    (sql, params)
}

fn build_where(dialect: &dyn SqlDialect, filters: &[TableFilter]) -> (String, Vec<QueryParam>) {
    if filters.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut conditions = Vec::with_capacity(filters.len());
    let mut params = Vec::new();
    let mut pw = PlaceholderWriter::new(dialect);

    for filter in filters {
        let column = dialect.quote_identifier(&filter.column);
        match filter.op {
            FilterOp::IsNull => conditions.push(format!("{column} IS NULL")),
            FilterOp::IsNotNull => conditions.push(format!("{column} IS NOT NULL")),
            FilterOp::Eq => {
                conditions.push(format!("{column} = {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Neq => {
                conditions.push(format!("{column} != {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Lt => {
                conditions.push(format!("{column} < {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Lte => {
                conditions.push(format!("{column} <= {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Gt => {
                conditions.push(format!("{column} > {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Gte => {
                conditions.push(format!("{column} >= {}", pw.next()));
                params.push(cell_to_param(&filter.value));
            }
            FilterOp::Like => {
                conditions.push(format!("CAST({column} AS TEXT) LIKE {}", pw.next()));
                params.push(cell_to_param(&filter.value));
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
        .map(|sort| {
            let direction = match sort.direction {
                SortDir::Asc => "ASC",
                SortDir::Desc => "DESC",
            };
            format!("{} {direction}", dialect.quote_identifier(&sort.column))
        })
        .collect();

    format!(" ORDER BY {}", parts.join(", "))
}
