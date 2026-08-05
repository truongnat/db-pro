use async_trait::async_trait;
use db_pro_core::domain::connection::{Connection, ConnectionConfig, ConnectionId};
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::ConnectionRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl ConnectionRepository for SQLiteMetaStore {
    async fn save(&self, connection: &Connection) -> Result<(), DbError> {
        let data =
            serde_json::to_string(connection).map_err(|e| DbError::Internal(format!("serialize connection: {e}")))?;
        let id = connection.id.to_string();
        let created_at = connection.created_at.to_rfc3339();
        let updated_at = connection.updated_at.to_rfc3339();
        let sql = "INSERT OR REPLACE INTO connections (id, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)";
        self.actor
            .raw_query(sql.into(), vec![id, data, created_at, updated_at])
            .await?;
        Ok(())
    }

    async fn get(&self, id: &ConnectionId) -> Result<Option<Connection>, DbError> {
        let rows = self
            .actor
            .raw_query(
                "SELECT data FROM connections WHERE id = ?1".into(),
                vec![id.to_string()],
            )
            .await?;
        match rows.first() {
            Some(row) => {
                let data = &row[0];
                let conn: Connection = serde_json::from_str(data)
                    .map_err(|e| DbError::Internal(format!("deserialize connection: {e}")))?;
                Ok(Some(conn))
            }
            None => Ok(None),
        }
    }

    async fn get_config(&self, id: &ConnectionId) -> Result<Option<ConnectionConfig>, DbError> {
        Ok(self.get(id).await?.map(|c| c.config))
    }

    async fn list(&self) -> Result<Vec<Connection>, DbError> {
        let rows = self
            .actor
            .raw_query("SELECT data FROM connections ORDER BY created_at".into(), vec![])
            .await?;
        let mut connections = Vec::new();
        for row in rows {
            let conn: Connection =
                serde_json::from_str(&row[0]).map_err(|e| DbError::Internal(format!("deserialize connection: {e}")))?;
            connections.push(conn);
        }
        Ok(connections)
    }

    async fn delete(&self, id: &ConnectionId) -> Result<(), DbError> {
        self.actor
            .raw_query("DELETE FROM connections WHERE id = ?1".into(), vec![id.to_string()])
            .await?;
        Ok(())
    }
}
