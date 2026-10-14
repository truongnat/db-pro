//! Tests for `../query_session.rs` (moved out of the source file).
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
