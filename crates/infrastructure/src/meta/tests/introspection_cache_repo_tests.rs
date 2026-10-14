//! Tests for `../introspection_cache_repo.rs` (moved out of the source file).
use super::*;
use db_pro_core::ports::IntrospectionCache;

#[tokio::test]
async fn malformed_cache_is_discarded_and_reported_as_a_miss() {
    let store = SQLiteMetaStore::new(":memory:").await.unwrap();
    let connection_id = ConnectionId::new();
    store
        .actor
        .raw_query(
            "INSERT INTO introspection_cache (connection_id, data, updated_at) VALUES (?1, ?2, ?3)".into(),
            vec![
                connection_id.to_string(),
                r#"{"foreign_keys":[{"name":"old"}]}"#.to_owned(),
                "now".to_owned(),
            ],
        )
        .await
        .unwrap();

    assert!(store.get(&connection_id).await.unwrap().is_none());
    assert!(store
        .actor
        .raw_query(
            "SELECT data FROM introspection_cache WHERE connection_id = ?1".into(),
            vec![connection_id.to_string()],
        )
        .await
        .unwrap()
        .is_empty());
}
