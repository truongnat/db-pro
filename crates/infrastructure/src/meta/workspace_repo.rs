use async_trait::async_trait;
use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::Workspace;
use db_pro_core::ports::WorkspaceRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl WorkspaceRepository for SQLiteMetaStore {
    async fn save(&self, workspace: &Workspace) -> Result<(), DbError> {
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO workspaces (id, name, default_connection_id, created_at) VALUES (?1, ?2, ?3, ?4)".into(),
                vec![
                    workspace.id.to_string(),
                    workspace.name.clone(),
                    workspace.default_connection_id.as_ref().map(|c| c.to_string()).unwrap_or_default(),
                    workspace.created_at.to_rfc3339(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Workspace>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT id, name, default_connection_id, created_at FROM workspaces ORDER BY created_at".into(),
                vec![],
            )
            .await?;
        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(Workspace {
                id: uuid::Uuid::parse_str(&row[0]).map_err(|e| DbError::Internal(format!("invalid uuid: {e}")))?,
                name: row[1].clone(),
                default_connection_id: if row[2].is_empty() {
                    None
                } else {
                    Some(
                        ConnectionId::parse(&row[2])
                            .map_err(|e| DbError::Internal(format!("invalid connection id: {e}")))?,
                    )
                },
                created_at: chrono::DateTime::parse_from_rfc3339(&row[3])
                    .map_err(|e| DbError::Internal(format!("invalid datetime: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(workspaces)
    }

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError> {
        self.actor
            .raw_query("DELETE FROM workspaces WHERE id = ?1".into(), vec![id.to_string()])
            .await?;
        Ok(())
    }
}
