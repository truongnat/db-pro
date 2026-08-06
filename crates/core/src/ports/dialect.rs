pub trait SqlDialect: Send + Sync {
    fn placeholder(&self, index: usize) -> String;
    fn quote_identifier(&self, name: &str) -> String;
    fn pagination_clause(&self, limit_placeholder: &str, offset_placeholder: &str) -> String {
        format!(" LIMIT {limit_placeholder} OFFSET {offset_placeholder}")
    }
    fn pagination_requires_order_by(&self) -> bool {
        false
    }
}
