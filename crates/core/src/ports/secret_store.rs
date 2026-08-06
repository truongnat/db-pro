use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::error::DbError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), DbError>;

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError>;

    async fn delete_secret(&self, key: &str) -> Result<(), DbError>;
}

#[async_trait]
impl<T: SecretStore + ?Sized> SecretStore for Arc<T> {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.as_ref().store_secret(key, value).await
    }

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        self.as_ref().retrieve_secret(key).await
    }

    async fn delete_secret(&self, key: &str) -> Result<(), DbError> {
        self.as_ref().delete_secret(key).await
    }
}
