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
            Some(row) => {
                let result: IntrospectResult = serde_json::from_str(&row[0])
                    .map_err(|e| DbError::Internal(format!("deserialize introspection cache: {e}")))?;
                Ok(Some(result))
            }
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
