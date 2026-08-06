use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::{SavedQuery, SavedQueryFolder};
use db_pro_core::ports::SavedQueryRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl SavedQueryRepository for SQLiteMetaStore {
    async fn save(&self, query: &SavedQuery) -> Result<(), DbError> {
        let data =
            serde_json::to_string(query).map_err(|e| DbError::Internal(format!("serialize saved query: {e}")))?;
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO saved_queries (id, connection_id, name, sql, folder, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)".into(),
                vec![
                    query.id.to_string(),
                    query.connection_id.to_string(),
                    query.name.clone(),
                    query.sql.clone(),
                    query.folder.clone().unwrap_or_default(),
                    query.created_at.to_rfc3339(),
                ],
            )
            .await?;
        let _ = data;
        Ok(())
    }

    async fn list(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT id, connection_id, name, sql, folder, created_at FROM saved_queries WHERE connection_id = ?1 ORDER BY created_at DESC".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        let mut queries = Vec::new();
        for row in rows {
            queries.push(SavedQuery {
                id: uuid::Uuid::parse_str(&row[0]).map_err(|e| DbError::Internal(format!("invalid uuid: {e}")))?,
                connection_id: ConnectionId::parse(&row[1])
                    .map_err(|e| DbError::Internal(format!("invalid connection id: {e}")))?,
                name: row[2].clone(),
                sql: row[3].clone(),
                folder: if row[4].is_empty() { None } else { Some(row[4].clone()) },
                created_at: chrono::DateTime::parse_from_rfc3339(&row[5])
                    .map_err(|e| DbError::Internal(format!("invalid datetime: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(queries)
    }

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.actor
            .raw_query("DELETE FROM saved_queries WHERE id = ?1".into(), vec![id.to_string()])
            .await?;
        Ok(())
    }

    async fn create_folder(&self, folder: &SavedQueryFolder) -> Result<(), DbError> {
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO saved_query_folders (id, connection_id, name, created_at) VALUES (?1, ?2, ?3, ?4)".into(),
                vec![
                    folder.id.to_string(),
                    folder.connection_id.to_string(),
                    folder.name.clone(),
                    folder.created_at.to_rfc3339(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn list_folders(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQueryFolder>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT id, connection_id, name, created_at FROM saved_query_folders WHERE connection_id = ?1 ORDER BY name".into(),
                vec![connection_id.to_string()],
            )
            .await?;
        let mut folders = Vec::new();
        for row in rows {
            folders.push(SavedQueryFolder {
                id: uuid::Uuid::parse_str(&row[0]).map_err(|e| DbError::Internal(format!("invalid uuid: {e}")))?,
                connection_id: ConnectionId::parse(&row[1])
                    .map_err(|e| DbError::Internal(format!("invalid connection id: {e}")))?,
                name: row[2].clone(),
                created_at: chrono::DateTime::parse_from_rfc3339(&row[3])
                    .map_err(|e| DbError::Internal(format!("invalid datetime: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(folders)
    }

    async fn delete_folder(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.actor
            .raw_query("DELETE FROM saved_query_folders WHERE id = ?1".into(), vec![id.to_string()])
            .await?;
        Ok(())
    }
}
