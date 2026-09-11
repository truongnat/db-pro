use db_pro_core::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use db_pro_core::domain::connection::ConnectionConfig;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::BackupEngine;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct SqliteBackupEngine {
    config: ConnectionConfig,
}

impl SqliteBackupEngine {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(format!(".db-pro-{}.tmp", Uuid::new_v4()));
    PathBuf::from(value)
}

async fn publish_backup_without_overwrite(source: &Path, destination: &Path) -> Result<(), DbError> {
    tokio::fs::hard_link(source, destination).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            DbError::Validation(format!("backup output already exists: {}", destination.display()))
        } else {
            DbError::Internal(format!(
                "failed to publish SQLite backup {}: {error}",
                destination.display()
            ))
        }
    })?;

    if let Err(error) = tokio::fs::remove_file(source).await {
        tracing::warn!(
            path = %source.display(),
            %error,
            "failed to remove temporary SQLite backup after publishing"
        );
    }
    Ok(())
}

fn validate_sqlite_file(path: &Path) -> Result<(), DbError> {
    let connection = rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(crate::error::from_rusqlite)?;
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(crate::error::from_rusqlite)?;
    if result != "ok" {
        return Err(DbError::DataFailed(format!("SQLite integrity check failed: {result}")));
    }
    Ok(())
}

#[async_trait::async_trait]
impl BackupEngine for SqliteBackupEngine {
    async fn backup(&self, options: &BackupOptions, _password: &str) -> Result<BackupResult, DbError> {
        let src = Path::new(&self.config.database);
        let dst = Path::new(&options.output_path);
        if !src.is_file() {
            return Err(DbError::NotFound(format!(
                "SQLite database not found: {}",
                src.display()
            )));
        }
        if src == dst {
            return Err(DbError::Validation(
                "SQLite backup output must differ from the database path".into(),
            ));
        }
        if dst.exists() {
            return Err(DbError::Validation(format!(
                "backup output already exists: {}",
                dst.display()
            )));
        }

        let temporary = temporary_path(dst);
        let source = src.to_path_buf();
        let vacuum_target = temporary.clone();
        let result = tokio::task::spawn_blocking(move || {
            let connection = rusqlite::Connection::open(&source).map_err(crate::error::from_rusqlite)?;
            connection
                .busy_timeout(std::time::Duration::from_secs(5))
                .map_err(crate::error::from_rusqlite)?;
            connection
                .execute("VACUUM INTO ?1", [&vacuum_target.to_string_lossy().to_string()])
                .map_err(crate::error::from_rusqlite)
        })
        .await
        .map_err(|e| DbError::Internal(format!("SQLite backup worker failed: {e}")))?;
        if let Err(error) = result {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        if let Err(error) = publish_backup_without_overwrite(&temporary, dst).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        let metadata = tokio::fs::metadata(dst)
            .await
            .map_err(|e| DbError::Internal(format!("failed to read backup file: {e}")))?;

        Ok(BackupResult {
            output_path: options.output_path.clone(),
            size_bytes: metadata.len(),
        })
    }

    async fn restore(&self, options: &RestoreOptions, _password: &str) -> Result<(), DbError> {
        let src = Path::new(&options.input_path);
        let dst = Path::new(&self.config.database);
        if !src.is_file() {
            return Err(DbError::NotFound(format!(
                "SQLite restore input not found: {}",
                src.display()
            )));
        }
        if src == dst {
            return Err(DbError::Validation(
                "SQLite restore input must differ from the database path".into(),
            ));
        }

        let source = src.to_path_buf();
        tokio::task::spawn_blocking(move || validate_sqlite_file(&source))
            .await
            .map_err(|e| DbError::Internal(format!("SQLite restore validation worker failed: {e}")))??;

        let temporary = temporary_path(dst);
        if let Err(error) = tokio::fs::copy(src, &temporary).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(DbError::Internal(format!("failed to stage SQLite restore: {error}")));
        }

        let staged = temporary.clone();
        let validation = tokio::task::spawn_blocking(move || validate_sqlite_file(&staged))
            .await
            .map_err(|e| DbError::Internal(format!("SQLite restore validation worker failed: {e}")))?;
        if let Err(error) = validation {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        if let Err(error) = tokio::fs::rename(&temporary, dst).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(DbError::Internal(format!("failed to publish SQLite restore: {error}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_pro_core::domain::backup::{BackupFormat, BackupOptions, RestoreOptions};
    use db_pro_core::domain::connection::{DriverType, SslMode};

    fn config(database: &Path) -> ConnectionConfig {
        ConnectionConfig {
            name: "test".into(),
            host: String::new(),
            port: 0,
            database: database.to_string_lossy().into_owned(),
            username: String::new(),
            driver: DriverType::SQLite,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        }
    }

    #[tokio::test]
    async fn backup_and_restore_publish_only_valid_sqlite_files() {
        let root = std::env::temp_dir().join(format!("db-pro-backup-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.db");
        let backup = root.join("backup.db");
        let invalid = root.join("invalid.db");

        let connection = rusqlite::Connection::open(&source).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO items VALUES (1, 'snapshot');",
            )
            .unwrap();
        drop(connection);

        let engine = SqliteBackupEngine::new(config(&source));
        let result = engine
            .backup(
                &BackupOptions {
                    connection_id: "test".into(),
                    output_path: backup.to_string_lossy().into_owned(),
                    format: BackupFormat::Plain,
                    schemas: vec![],
                    tables: vec![],
                },
                "",
            )
            .await
            .unwrap();
        assert!(result.size_bytes > 0);
        let backup_connection = rusqlite::Connection::open(&backup).unwrap();
        assert_eq!(
            backup_connection
                .query_row("SELECT value FROM items", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "snapshot"
        );
        drop(backup_connection);

        std::fs::write(&invalid, b"not a sqlite database").unwrap();
        let error = engine
            .restore(
                &RestoreOptions {
                    connection_id: "test".into(),
                    input_path: invalid.to_string_lossy().into_owned(),
                    format: BackupFormat::Plain,
                },
                "",
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("SQLite") || error.to_string().contains("database"));
        assert!(source.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn backup_publish_does_not_overwrite_existing_destination() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("db-pro-backup-race-{}", Uuid::new_v4()));
        tokio::fs::create_dir_all(&root).await?;
        let source = root.join("temporary.db");
        let destination = root.join("backup.db");
        tokio::fs::write(&source, b"new backup").await?;
        tokio::fs::write(&destination, b"old backup").await?;

        let error = match publish_backup_without_overwrite(&source, &destination).await {
            Ok(()) => return Err("existing backup destination was overwritten".into()),
            Err(error) => error,
        };

        assert!(matches!(error, DbError::Validation(message) if message.contains("already exists")));
        assert_eq!(tokio::fs::read(&destination).await?, b"old backup");
        assert_eq!(tokio::fs::read(&source).await?, b"new backup");
        tokio::fs::remove_dir_all(root).await?;
        Ok(())
    }
}
