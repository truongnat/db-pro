use async_trait::async_trait;

use crate::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use crate::domain::error::DbError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BackupEngine: Send + Sync {
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError>;
    async fn restore(&self, options: &RestoreOptions, password: &str) -> Result<(), DbError>;
}
