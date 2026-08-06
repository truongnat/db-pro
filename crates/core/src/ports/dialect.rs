pub trait SqlDialect: Send + Sync {
    fn placeholder(&self, index: usize) -> String;
}
