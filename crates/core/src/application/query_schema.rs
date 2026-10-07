//! Schema selected in the editor, visible to the driver for the current task.
//!
//! A pooled connection must not keep this value. Drivers read it while they hold
//! one connection and put the session back before the connection returns to the pool.

tokio::task_local! {
    static ACTIVE_QUERY_SCHEMA: String;
}

/// Schema the editor asked this task to resolve unqualified names against.
pub fn active_query_schema() -> Option<String> {
    ACTIVE_QUERY_SCHEMA.try_with(|schema| schema.clone()).ok()
}

pub(crate) async fn with_query_schema<T>(schema: Option<&str>, fut: impl std::future::Future<Output = T>) -> T {
    match schema.map(str::trim).filter(|value| !value.is_empty()) {
        Some(schema) => ACTIVE_QUERY_SCHEMA.scope(schema.to_owned(), fut).await,
        None => fut.await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn schema_is_visible_only_inside_the_query_scope() {
        assert!(active_query_schema().is_none());
        with_query_schema(Some(" tenant1 "), async {
            assert_eq!(active_query_schema().as_deref(), Some("tenant1"));
        })
        .await;
        assert!(active_query_schema().is_none());
        with_query_schema(Some("   "), async {
            assert!(active_query_schema().is_none());
        })
        .await;
    }
}
