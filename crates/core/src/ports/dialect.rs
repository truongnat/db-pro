pub trait SqlDialect: Send + Sync {
    fn placeholder(&self, index: usize) -> String;
    fn quote_identifier(&self, name: &str) -> String;
    fn pagination_clause(&self, limit_placeholder: &str, offset_placeholder: &str) -> String {
        format!(" LIMIT {limit_placeholder} OFFSET {offset_placeholder}")
    }
    fn pagination_requires_order_by(&self) -> bool {
        false
    }

    /// Text comparison for contains / starts with / ends with.
    /// Postgres and SQLite accept `CAST AS TEXT`. MySQL and SQL Server do not.
    fn text_like(&self, column: &str, placeholder: &str) -> String {
        format!("CAST({column} AS TEXT) LIKE {placeholder} ESCAPE '\\'")
    }
}
