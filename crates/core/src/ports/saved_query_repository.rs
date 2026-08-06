use async_trait::async_trait;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::{SavedQuery, SavedQueryFolder};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SavedQueryRepository: Send + Sync {
    async fn save(&self, query: &SavedQuery) -> Result<(), DbError>;

    async fn list(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError>;

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError>;

    async fn create_folder(&self, folder: &SavedQueryFolder) -> Result<(), DbError>;

    async fn list_folders(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQueryFolder>, DbError>;

    async fn delete_folder(&self, id: &uuid::Uuid) -> Result<(), DbError>;
}
