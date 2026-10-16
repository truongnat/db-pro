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

/// The files SQLite keeps next to a database file, in the order they are removed.
///
/// `-wal` is not bound to the main database file: SQLite replays a WAL it finds
/// next to a database without checking that the WAL was written for *that*
/// database. A `-wal`/`-shm` pair left behind by an earlier session therefore
/// survives a plain file replacement and wins over the file that replaced it.
fn sidecar_paths(database: &Path) -> [PathBuf; 3] {
    ["-wal", "-shm", "-journal"].map(|suffix| {
        let mut value = database.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    })
}

async fn remove_database_sidecars(database: &Path) -> Result<(), DbError> {
    for sidecar in sidecar_paths(database) {
        match tokio::fs::remove_file(&sidecar).await {
            Ok(()) => tracing::info!(
                path = %sidecar.display(),
                "removed a SQLite sidecar file left by the database being replaced"
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(DbError::Internal(format!(
                    "failed to remove outdated SQLite sidecar {}: {error}",
                    sidecar.display()
                )))
            }
        }
    }
    Ok(())
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

/// Flush a finished file to stable storage before it is published by name.
async fn flush_file(path: &Path) -> std::io::Result<()> {
    // Opened writable: `sync_all` is not required to work on a read-only handle
    // on every platform.
    let handle = tokio::fs::OpenOptions::new().write(true).open(path).await?;
    handle.sync_all().await
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

/// Source must be an existing database file, destination must be a fresh path —
/// refusing both mistakes keeps the backup a copy, never an overwrite.
fn validate_backup_paths(src: &Path, dst: &Path) -> Result<(), DbError> {
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
    Ok(())
}

/// VACUUM INTO on a blocking rusqlite connection, off the async executor.
async fn vacuum_into(source: &Path, temporary: &Path) -> Result<(), DbError> {
    let source = source.to_path_buf();
    let vacuum_target = temporary.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let connection = rusqlite::Connection::open(&source).map_err(crate::error::from_rusqlite)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(crate::error::from_rusqlite)?;
        connection
            .execute("VACUUM INTO ?1", [&vacuum_target.to_string_lossy().to_string()])
            .map_err(crate::error::from_rusqlite)?;
        Ok(())
    })
    .await
    .map_err(|e| DbError::Internal(format!("SQLite backup worker failed: {e}")))?
}

#[async_trait::async_trait]
impl BackupEngine for SqliteBackupEngine {
    async fn backup(&self, options: &BackupOptions, _password: &str) -> Result<BackupResult, DbError> {
        let src = Path::new(&self.config.database);
        let dst = Path::new(&options.output_path);
        validate_backup_paths(src, dst)?;

        let temporary = temporary_path(dst);
        if let Err(error) = vacuum_into(src, &temporary).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        // Flush the artifact before it becomes visible under its final name, so a
        // published backup cannot be reported as written while still buffered.
        flush_file(&temporary).await.map_err(|error| {
            let _ = std::fs::remove_file(&temporary);
            DbError::Internal(format!(
                "failed to flush SQLite backup {}: {error}",
                temporary.display()
            ))
        })?;

        // The output path does not hold a database (checked above), so a sidecar
        // file at that name belongs to a database that no longer exists. Leaving
        // it would let SQLite replay it into the artifact being published, and the
        // backup would read back as that other database.
        if let Err(error) = remove_database_sidecars(dst).await {
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

    // cc-scan:allow COMPLEXITY,LONG_FUNCTION — classifier/dispatch ladder — one case per branch
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
        // Copying the main file omits committed transactions that are still in
        // the source database's WAL. VACUUM INTO reads a consistent SQLite
        // snapshot (including WAL content) into a self-contained staged file.
        if let Err(error) = vacuum_into(src, &temporary).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        let staged = temporary.clone();
        let validation = tokio::task::spawn_blocking(move || validate_sqlite_file(&staged))
            .await
            .map_err(|e| DbError::Internal(format!("SQLite restore validation worker failed: {e}")))?;
        if let Err(error) = validation {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }

        // The staged copy is valid, so the restore is committed to replacing the
        // target: drop the target's own sidecar files first. `-wal` would
        // otherwise be replayed into the file published below and a reader would
        // keep seeing the replaced database. Only the rename can still fail after
        // this point, and that path is reported explicitly with the previous
        // target still in place.
        if let Err(error) = remove_database_sidecars(dst).await {
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
#[path = "tests/sqlite_backup_tests.rs"]
mod tests;
