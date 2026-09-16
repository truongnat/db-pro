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
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
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

    fn temporary_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("db-pro-backup-{name}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn values(database: &Path) -> Vec<String> {
        let connection = rusqlite::Connection::open(database).unwrap();
        let mut statement = connection.prepare("SELECT value FROM items ORDER BY value").unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|row| row.unwrap())
            .collect()
    }

    fn create_items_database(database: &Path, values_sql: &str) {
        let connection = rusqlite::Connection::open(database).unwrap();
        connection
            .execute_batch(&format!(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); {values_sql}"
            ))
            .unwrap();
    }

    fn options(database: &Path) -> BackupOptions {
        BackupOptions {
            connection_id: "test".into(),
            output_path: database.to_string_lossy().into_owned(),
            format: BackupFormat::Plain,
            schemas: vec![],
            tables: vec![],
        }
    }

    /// A database in WAL mode keeps committed data in the `-wal` file until a
    /// checkpoint. The backup must still contain it.
    #[tokio::test]
    async fn backup_captures_committed_writes_that_are_still_in_the_wal() {
        let root = temporary_root("wal-source");
        let source = root.join("source.db");
        let backup = root.join("backup.db");

        let writer = rusqlite::Connection::open(&source).unwrap();
        writer.pragma_update(None, "journal_mode", "WAL").unwrap();
        writer
            .execute_batch(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO items VALUES (1, 'committed');",
            )
            .unwrap();
        assert!(
            root.join("source.db-wal").exists(),
            "the write must still be in the WAL"
        );

        SqliteBackupEngine::new(config(&source))
            .backup(&options(&backup), "")
            .await
            .expect("backup should succeed");

        assert_eq!(values(&backup), vec!["committed"]);
        drop(writer);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A writer holding an open write transaction must not produce a torn backup:
    /// the artifact contains the pre-transaction state, never the uncommitted row.
    #[tokio::test]
    async fn backup_during_an_open_write_transaction_is_a_consistent_snapshot() {
        let root = temporary_root("concurrent-writer");
        let source = root.join("source.db");
        let backup = root.join("backup.db");

        let writer = rusqlite::Connection::open(&source).unwrap();
        writer.pragma_update(None, "journal_mode", "WAL").unwrap();
        writer
            .execute_batch(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO items VALUES (1, 'before');",
            )
            .unwrap();
        writer.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
        writer
            .execute_batch("BEGIN IMMEDIATE; INSERT INTO items VALUES (2, 'uncommitted');")
            .unwrap();

        SqliteBackupEngine::new(config(&source))
            .backup(&options(&backup), "")
            .await
            .expect("backup should succeed while a transaction is open");

        // Exactly the committed state: the open transaction is neither visible
        // nor able to produce a half-applied snapshot.
        assert_eq!(values(&backup), vec!["before"]);

        writer.execute_batch("COMMIT").unwrap();
        assert_eq!(values(&source), vec!["before", "uncommitted"]);
        drop(writer);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A `-wal` left at the output path (its database is gone) belongs to that
    /// other database: publishing the artifact next to it would make the backup
    /// read back as foreign content.
    #[tokio::test]
    async fn backup_does_not_publish_next_to_a_stale_wal() {
        let root = temporary_root("output-sidecar");
        let source = root.join("source.db");
        let backup = root.join("backup.db");

        create_items_database(&source, "INSERT INTO items VALUES (1, 'real-backup-content');");
        // A valid WAL written for a different database, left at the output path.
        let other = root.join("other.db");
        let writer = rusqlite::Connection::open(&other).unwrap();
        writer.pragma_update(None, "journal_mode", "WAL").unwrap();
        writer
            .execute_batch(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO items VALUES (1, 'foreign-wal-content');",
            )
            .unwrap();
        let valid_wal = std::fs::read(root.join("other.db-wal")).unwrap();
        drop(writer);
        std::fs::remove_file(&other).unwrap();
        std::fs::write(root.join("backup.db-wal"), &valid_wal).unwrap();

        SqliteBackupEngine::new(config(&source))
            .backup(&options(&backup), "")
            .await
            .expect("backup should succeed");

        assert!(
            !root.join("backup.db-wal").exists(),
            "the stale sidecar must not survive"
        );
        assert_eq!(values(&backup), vec!["real-backup-content"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Replacing the database file is not enough: a `-wal`/`-shm` pair left next
    /// to the target by an earlier session belongs to the *old* database and
    /// would be replayed into the restored file, so the restore would silently
    /// not take effect on what a reader sees.
    #[tokio::test]
    async fn restore_replaces_the_database_and_invalidates_stale_sidecars() {
        let root = temporary_root("stale-sidecars");
        let target = root.join("target.db");
        let artifact = root.join("artifact.db");

        create_items_database(&artifact, "INSERT INTO items VALUES (1, 'from-backup');");

        // A target whose sidecars were left behind by a crashed WAL-mode session.
        let writer = rusqlite::Connection::open(&target).unwrap();
        writer.pragma_update(None, "journal_mode", "WAL").unwrap();
        writer
            .execute_batch(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO items VALUES (1, 'from-old-db');",
            )
            .unwrap();
        let stale_wal = std::fs::read(root.join("target.db-wal")).unwrap();
        let stale_shm = std::fs::read(root.join("target.db-shm")).unwrap();
        drop(writer);
        std::fs::write(root.join("target.db-wal"), &stale_wal).unwrap();
        std::fs::write(root.join("target.db-shm"), &stale_shm).unwrap();

        SqliteBackupEngine::new(config(&target))
            .restore(
                &RestoreOptions {
                    connection_id: "test".into(),
                    input_path: artifact.to_string_lossy().into_owned(),
                    format: BackupFormat::Plain,
                },
                "",
            )
            .await
            .expect("restore should succeed");

        assert!(
            !root.join("target.db-wal").exists(),
            "the old database's WAL must not survive"
        );
        assert!(
            !root.join("target.db-shm").exists(),
            "the old database's SHM must not survive"
        );
        assert_eq!(values(&target), vec!["from-backup"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A failed publish must keep the previous target and remove the staged file.
    #[tokio::test]
    async fn failed_restore_keeps_the_previous_target_and_removes_the_staged_file() {
        let root = temporary_root("failed-publish");
        let input = root.join("input.db");
        let target = root.join("target.db");

        create_items_database(&input, "INSERT INTO items VALUES (1, 'good');");
        // A target that rename(2) cannot replace: a non-empty directory at the path.
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep.txt"), b"kept").unwrap();

        let error = SqliteBackupEngine::new(config(&target))
            .restore(
                &RestoreOptions {
                    connection_id: "test".into(),
                    input_path: input.to_string_lossy().into_owned(),
                    format: BackupFormat::Plain,
                },
                "",
            )
            .await
            .expect_err("publishing over a directory must fail");

        assert!(
            error.to_string().contains("failed to publish"),
            "unexpected error: {error}"
        );
        assert_eq!(std::fs::read(target.join("keep.txt")).unwrap(), b"kept");
        let leftovers: Vec<String> = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".db-pro-"))
            .collect();
        assert!(leftovers.is_empty(), "staged files left behind: {leftovers:?}");
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A backup that cannot be produced must leave no accepted or partial artifact.
    #[tokio::test]
    async fn failed_backup_leaves_no_artifact_and_no_staged_file() {
        let root = temporary_root("failed-backup");
        let source = root.join("source.db");
        let backup = root.join("backup.db");

        // Not a database: VACUUM INTO cannot read it.
        std::fs::write(&source, b"not a sqlite database").unwrap();

        SqliteBackupEngine::new(config(&source))
            .backup(&options(&backup), "")
            .await
            .expect_err("backing up a non-database must fail");

        assert!(!backup.exists(), "a failed backup must not publish an artifact");
        let leftovers: Vec<String> = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".db-pro-"))
            .collect();
        assert!(leftovers.is_empty(), "staged files left behind: {leftovers:?}");
        std::fs::remove_dir_all(root).unwrap();
    }
}
