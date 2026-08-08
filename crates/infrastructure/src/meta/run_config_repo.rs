use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::run_config::RunConfig;
use db_pro_core::ports::RunConfigRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl RunConfigRepository for SQLiteMetaStore {
    async fn save(&self, config: &RunConfig) -> Result<(), DbError> {
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO run_configs (id, connection_id, name, sql, timeout_ms, max_rows, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)".into(),
                vec![
                    config.id.to_string(),
                    config.connection_id.to_string(),
                    config.name.clone(),
                    config.sql.clone(),
                    config.timeout_ms.to_string(),
                    config.max_rows.to_string(),
                    config.created_at.to_rfc3339(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn list(&self, connection_id: &ConnectionId) -> Result<Vec<RunConfig>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT id, connection_id, name, sql, timeout_ms, max_rows, created_at FROM run_configs WHERE connection_id = ?1 ORDER BY name".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        let mut configs = Vec::new();
        for row in rows {
            configs.push(RunConfig {
                id: uuid::Uuid::parse_str(&row[0]).map_err(|e| DbError::Internal(format!("invalid uuid: {e}")))?,
                connection_id: ConnectionId::parse(&row[1])
                    .map_err(|e| DbError::Internal(format!("invalid connection id: {e}")))?,
                name: row[2].clone(),
                sql: row[3].clone(),
                timeout_ms: row[4]
                    .parse()
                    .map_err(|e| DbError::Internal(format!("invalid timeout_ms: {e}")))?,
                max_rows: row[5]
                    .parse()
                    .map_err(|e| DbError::Internal(format!("invalid max_rows: {e}")))?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row[6])
                    .map_err(|e| DbError::Internal(format!("invalid datetime: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(configs)
    }

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.actor
            .raw_query("DELETE FROM run_configs WHERE id = ?1".into(), vec![id.to_string()])
            .await?;
        Ok(())
    }
}
