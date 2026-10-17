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
                "SELECT data, updated_at FROM introspection_cache WHERE connection_id = ?1".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        // TTL guard: caches older than 10 minutes are expired to prevent serving stale schemas indefinitely (#467).
        const CACHE_TTL_SECS: i64 = 600;
        if let Some(updated_at_str) = row.get(1) {
            if let Ok(updated_at) = chrono::DateTime::parse_from_rfc3339(updated_at_str) {
                let age = chrono::Utc::now().signed_duration_since(updated_at).num_seconds();
                if age > CACHE_TTL_SECS {
                    tracing::info!(
                        connection_id = %connection_id,
                        age_secs = age,
                        "introspection cache expired by TTL"
                    );
                    self.invalidate(connection_id).await?;
                    return Ok(None);
                }
            }
        }
        match serde_json::from_str::<IntrospectResult>(&row[0]) {
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
#[path = "tests/introspection_cache_repo_tests.rs"]
mod tests;
