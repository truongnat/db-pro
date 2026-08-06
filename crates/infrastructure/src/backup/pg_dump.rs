use std::process::Stdio;
use tokio::process::Command;

use db_pro_core::domain::backup::{BackupFormat, BackupOptions, BackupResult, RestoreOptions};
use db_pro_core::domain::connection::ConnectionConfig;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::BackupEngine;

pub struct PgDumpEngine {
    config: ConnectionConfig,
}

impl PgDumpEngine {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl BackupEngine for PgDumpEngine {
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError> {
        let mut cmd = Command::new("pg_dump");
        cmd.arg("-h").arg(&self.config.host)
            .arg("-p").arg(self.config.port.to_string())
            .arg("-U").arg(&self.config.username)
            .arg("-d").arg(&self.config.database)
            .arg("-f").arg(&options.output_path)
            .env("PGPASSWORD", password);

        match options.format {
            BackupFormat::Plain => { cmd.arg("--format=plain"); }
            BackupFormat::Custom => { cmd.arg("--format=custom"); }
        }

        for schema in &options.schemas {
            cmd.arg("-n").arg(schema);
        }
        for table in &options.tables {
            cmd.arg("-t").arg(table);
        }

        cmd.stdin(Stdio::null());

        let output = cmd.output().await.map_err(|e| {
            DbError::Internal(format!("failed to run pg_dump: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DbError::Internal(format!("pg_dump failed: {stderr}")));
        }

        let metadata = tokio::fs::metadata(&options.output_path).await
            .map_err(|e| DbError::Internal(format!("failed to read backup file: {e}")))?;

        Ok(BackupResult {
            output_path: options.output_path.clone(),
            size_bytes: metadata.len(),
        })
    }

    async fn restore(&self, options: &RestoreOptions, password: &str) -> Result<(), DbError> {
        let output = match options.format {
            BackupFormat::Plain => {
                Command::new("psql")
                    .arg("-h").arg(&self.config.host)
                    .arg("-p").arg(self.config.port.to_string())
                    .arg("-U").arg(&self.config.username)
                    .arg("-d").arg(&self.config.database)
                    .arg("-f").arg(&options.input_path)
                    .env("PGPASSWORD", password)
                    .stdin(Stdio::null())
                    .output()
                    .await
            }
            BackupFormat::Custom => {
                Command::new("pg_restore")
                    .arg("-h").arg(&self.config.host)
                    .arg("-p").arg(self.config.port.to_string())
                    .arg("-U").arg(&self.config.username)
                    .arg("-d").arg(&self.config.database)
                    .arg(&options.input_path)
                    .env("PGPASSWORD", password)
                    .stdin(Stdio::null())
                    .output()
                    .await
            }
        };

        let output = output.map_err(|e| {
            DbError::Internal(format!("failed to run restore: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DbError::Internal(format!("restore failed: {stderr}")));
        }

        Ok(())
    }
}
