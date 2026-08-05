use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::DbConnector;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

use super::actor::SqliteHandle;

pub struct SQLiteConnector {
    actors: RwLock<HashMap<u64, SqliteHandle>>,
    next_id: AtomicU64,
}

impl Default for SQLiteConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl SQLiteConnector {
    pub fn new() -> Self {
        Self {
            actors: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl DbConnector for SQLiteConnector {
    async fn connect(&self, config: &ConnectionConfig, _password: &str) -> Result<ConnectionHandle, DbError> {
        let db_path = if config.database.is_empty() {
            ":memory:"
        } else {
            &config.database
        };
        let handle = super::actor::SqliteActor::spawn(db_path)?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.actors.write().await.insert(id, handle);
        Ok(ConnectionHandle::new(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        if let Some(actor) = self.actors.write().await.remove(&handle.0) {
            actor.shutdown().await;
        }
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let handle = self.connect(config, password).await?;
        let actors = self.actors.read().await;
        let actor = actors
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        actor.execute("SELECT 1".into(), vec![]).await.map(|_| ())?;
        drop(actors);
        self.disconnect(&handle).await
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        let actors = self.actors.read().await;
        let actor = actors
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        actor.execute(sql.into(), params.to_vec()).await
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let actors = self.actors.read().await;
        let actor = actors
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        actor.introspect().await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        let actors = self.actors.read().await;
        let actor = actors
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed("handle not found".into()))?;
        actor.explain(sql.into()).await
    }
}
