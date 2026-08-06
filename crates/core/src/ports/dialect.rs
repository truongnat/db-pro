pub trait SqlDialect: Send + Sync {
    fn placeholder(&self, index: usize) -> String;
    fn quote_identifier(&self, name: &str) -> String;
}
