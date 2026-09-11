use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::IntrospectionCache;

use super::store::SQLiteMetaStore;

#[async_trait]
impl IntrospectionCache for SQLiteMetaStore {
    async fn save(&self, connection_id: &ConnectionId, result: &IntrospectResult) -> Result<(), DbError> {
        let data = serde_json::to_string(result)
            .map_err(|e| DbError::Internal(format!("serialize introspection cache: {e}")))?;
        let updated_at = chrono::Utc::now().to_rfc3339();
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO introspection_cache (connection_id, data, updated_at) VALUES (?1, ?2, ?3)"
                    .into(),
                vec![connection_id.to_string(), data, updated_at],
            )
            .await?;
        Ok(())
    }

    async fn get(&self, connection_id: &ConnectionId) -> Result<Option<IntrospectResult>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT data FROM introspection_cache WHERE connection_id = ?1".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        match rows.first() {
            Some(row) => match serde_json::from_str::<IntrospectResult>(&row[0]) {
                Ok(result) => Ok(Some(result)),
                Err(error) => {
                    tracing::warn!(
                        connection_id = %connection_id,
                        error = %error,
                        "discarding malformed introspection cache"
                    );
                    self.invalidate(connection_id).await?;
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }

    async fn invalidate(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        self.actor
            .raw_query(
                "DELETE FROM introspection_cache WHERE connection_id = ?1".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
