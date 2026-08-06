use db_pro_core::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use db_pro_core::domain::connection::ConnectionConfig;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::BackupEngine;

pub struct SqliteBackupEngine {
    config: ConnectionConfig,
}

impl SqliteBackupEngine {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl BackupEngine for SqliteBackupEngine {
    async fn backup(&self, options: &BackupOptions, _password: &str) -> Result<BackupResult, DbError> {
        let src = &self.config.database;
        let dst = &options.output_path;

        tokio::fs::copy(src, dst).await.map_err(|e| {
            DbError::Internal(format!("failed to copy SQLite database: {e}"))
        })?;

        let metadata = tokio::fs::metadata(dst).await
            .map_err(|e| DbError::Internal(format!("failed to read backup file: {e}")))?;

        Ok(BackupResult {
            output_path: options.output_path.clone(),
            size_bytes: metadata.len(),
        })
    }

    async fn restore(&self, options: &RestoreOptions, _password: &str) -> Result<(), DbError> {
        let src = &options.input_path;
        let dst = &self.config.database;

        tokio::fs::copy(src, dst).await.map_err(|e| {
            DbError::Internal(format!("failed to restore SQLite database: {e}"))
        })?;

        Ok(())
    }
}
