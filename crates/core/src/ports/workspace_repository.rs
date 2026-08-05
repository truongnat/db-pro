use async_trait::async_trait;

use crate::domain::error::DbError;
use crate::domain::history::Workspace;

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn save(&self, workspace: &Workspace) -> Result<(), DbError>;

    async fn list(&self) -> Result<Vec<Workspace>, DbError>;

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError>;
}
