//! Tests for `../query_schema.rs` (moved out of the source file).
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
