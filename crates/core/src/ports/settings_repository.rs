use async_trait::async_trait;

use crate::domain::error::DbError;
use crate::domain::history::Settings;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn save(&self, settings: &Settings) -> Result<(), DbError>;

    async fn load(&self) -> Result<Settings, DbError>;
}
