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
#[path = "tests/sql_builder_tests.rs"]
mod tests;
