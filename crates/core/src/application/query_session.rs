//! Query tab that owns the open transaction, visible to the driver for this task.
//!
//! An empty or missing id is the anonymous slot. Schema apply and the table grid
//! leave it unset, so they do not join a query tab's transaction.

tokio::task_local! {
    static ACTIVE_QUERY_SESSION: String;
}

/// Document id of the query tab this task is running for.
pub fn active_query_session() -> Option<String> {
    ACTIVE_QUERY_SESSION.try_with(|session| session.clone()).ok()
}

pub async fn with_query_session<T>(session: Option<&str>, fut: impl std::future::Future<Output = T>) -> T {
    match session.map(str::trim).filter(|value| !value.is_empty()) {
        Some(session) => ACTIVE_QUERY_SESSION.scope(session.to_owned(), fut).await,
        None => fut.await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn session_is_visible_only_inside_the_query_scope() {
        assert!(active_query_session().is_none());
        with_query_session(Some(" query-2 "), async {
            assert_eq!(active_query_session().as_deref(), Some("query-2"));
        })
        .await;
        assert!(active_query_session().is_none());
        with_query_session(Some("   "), async {
            assert!(active_query_session().is_none());
        })
        .await;
    }
}
