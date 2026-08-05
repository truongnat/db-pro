use async_trait::async_trait;

use crate::domain::error::DbError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), DbError>;

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError>;

    async fn delete_secret(&self, key: &str) -> Result<(), DbError>;
}
