use async_trait::async_trait;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::Settings;
use db_pro_core::ports::SettingsRepository;

use super::store::SQLiteMetaStore;

#[async_trait]
impl SettingsRepository for SQLiteMetaStore {
    async fn save(&self, settings: &Settings) -> Result<(), DbError> {
        let data =
            serde_json::to_string(settings).map_err(|e| DbError::Internal(format!("serialize settings: {e}")))?;
        self.actor
            .raw_query(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('settings', ?1)".into(),
                vec![data],
            )
            .await?;
        Ok(())
    }

    async fn load(&self) -> Result<Settings, DbError> {
        let rows = self
            .actor
            .raw_query("SELECT value FROM settings WHERE key = 'settings'".into(), vec![])
            .await?;
        match rows.first() {
            Some(row) => {
                let settings: Settings = serde_json::from_str(&row[0])
                    .map_err(|e| DbError::Internal(format!("deserialize settings: {e}")))?;
                Ok(settings)
            }
            None => Ok(Settings::default()),
        }
    }
}
