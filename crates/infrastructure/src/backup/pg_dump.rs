use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

use db_pro_core::domain::backup::{BackupFormat, BackupOptions, BackupResult, RestoreOptions};
use db_pro_core::domain::connection::ConnectionConfig;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::BackupEngine;

use crate::ssh::{SshTunnel, SshTunnelConfig, SshTunnelHandle};

pub struct PgDumpEngine {
    config: ConnectionConfig,
}

impl PgDumpEngine {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config }
    }

    async fn effective_config(&self) -> Result<(ConnectionConfig, Option<SshTunnelHandle>), DbError> {
        let Some(ssh_config) = self.config.ssh_tunnel.as_ref() else {
            return Ok((self.config.clone(), None));
        };

        let tunnel_config = SshTunnelConfig {
            host: ssh_config.host.clone(),
            port: ssh_config.port,
            user: ssh_config.user.clone(),
            private_key_path: ssh_config.private_key_path.clone(),
            password: ssh_config.password.clone(),
        };
        let tunnel = SshTunnel::start(&tunnel_config, &self.config.host, self.config.port).await?;

        let mut effective_config = self.config.clone();
        effective_config.host = "127.0.0.1".to_owned();
        effective_config.port = tunnel.local_port();
        Ok((effective_config, Some(tunnel)))
    }

    async fn run_command(
        command: &mut Command,
        timeout_ms: u64,
        operation: &str,
    ) -> Result<std::process::Output, DbError> {
        command.kill_on_drop(true);
        tokio::time::timeout(Duration::from_millis(timeout_ms), command.output())
            .await
            .map_err(|_| DbError::QueryTimeout { timeout_ms })?
            .map_err(|error| DbError::Internal(format!("failed to run {operation}: {error}")))
    }

    async fn reserve_backup_output(path: &Path) -> Result<(), DbError> {
        tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .await
            .map(|_| ())
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    DbError::Validation(format!("backup output already exists: {}", path.display()))
                } else {
                    DbError::Internal(format!("failed to reserve backup output {}: {error}", path.display()))
                }
            })
    }

    async fn remove_failed_output(path: &Path) {
        if let Err(error) = tokio::fs::remove_file(path).await {
            tracing::warn!(path = %path.display(), %error, "failed to remove incomplete PostgreSQL backup output");
        }
    }

    /// The `psql`/`pg_restore` invocation used by [`Self::restore`].
    ///
    /// Split out so the argv shape is testable without a live PostgreSQL server (#244, E-2 asks for
    /// exactly that): the restore runs **without** `--single-transaction` and without
    /// `--exit-on-error`, which is the behaviour LIM-020 documents.
    fn restore_command(options: &RestoreOptions, config: &ConnectionConfig, password: &str) -> Command {
        let mut cmd = match options.format {
            BackupFormat::Plain => {
                let mut command = Command::new("psql");
                command.arg("-f").arg(&options.input_path);
                command
            }
            BackupFormat::Custom => {
                let mut command = Command::new("pg_restore");
                command.arg(&options.input_path);
                command
            }
        };
        cmd.arg("-h")
            .arg(&config.host)
            .arg("-p")
            .arg(config.port.to_string())
            .arg("-U")
            .arg(&config.username)
            .arg("-d")
            .arg(&config.database)
            .env("PGPASSWORD", password)
            .stdin(Stdio::null());
        cmd
    }
}

#[async_trait::async_trait]
impl BackupEngine for PgDumpEngine {
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError> {
        let output_path = Path::new(&options.output_path);
        Self::reserve_backup_output(output_path).await?;

        let result = async {
            let (config, _tunnel) = self.effective_config().await?;
            let mut cmd = Command::new("pg_dump");
            cmd.arg("-h")
                .arg(&config.host)
                .arg("-p")
                .arg(config.port.to_string())
                .arg("-U")
                .arg(&config.username)
                .arg("-d")
                .arg(&config.database)
                .arg("-f")
                .arg(&options.output_path)
                .env("PGPASSWORD", password);

            match options.format {
                BackupFormat::Plain => {
                    cmd.arg("--format=plain");
                }
                BackupFormat::Custom => {
                    cmd.arg("--format=custom");
                }
            }

            for schema in &options.schemas {
                cmd.arg("-n").arg(schema);
            }
            for table in &options.tables {
                cmd.arg("-t").arg(table);
            }

            cmd.stdin(Stdio::null());

            let output = Self::run_command(&mut cmd, config.query_timeout_ms, "pg_dump").await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DbError::Internal(format!("pg_dump failed: {stderr}")));
            }

            let metadata = tokio::fs::metadata(&options.output_path)
                .await
                .map_err(|e| DbError::Internal(format!("failed to read backup file: {e}")))?;

            Ok(BackupResult {
                output_path: options.output_path.clone(),
                size_bytes: metadata.len(),
            })
        }
        .await;

        if result.is_err() {
            Self::remove_failed_output(output_path).await;
        }
        result
    }

    async fn restore(&self, options: &RestoreOptions, password: &str) -> Result<(), DbError> {
        let (config, _tunnel) = self.effective_config().await?;
        let mut cmd = Self::restore_command(options, &config, password);

        let output = Self::run_command(&mut cmd, config.query_timeout_ms, "PostgreSQL restore").await?;

        if !output.status.success() {
            // #244 (E-2), option (b) of the issue: the restore is deliberately **not** run inside a
            // transaction, so a mid-script failure can leave the target database partially restored.
            // The state is what matters to the user and it is reported here rather than left implicit
            // in a bare subprocess error; `psql --single-transaction` / `pg_restore
            // --single-transaction` (option (a)) would trade that for failing as a whole on dumps
            // that contain non-transactional statements, which is a compatibility decision the owner
            // owns (recorded in LIM-020).
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DbError::Internal(format!(
                "restore failed: {stderr} -- the restore was NOT run in a transaction, so the target database may now be \
                 partially restored; inspect it before using it (documented in LIM-020). The full psql/pg_restore output is above"
            )));
        }

        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    /// #244 (E-2) asks for the argv shape to be asserted without a live server. This pins the
    /// **chosen** behaviour (option (b) of the issue): the restore is not wrapped in a transaction,
    /// so the app must say so on failure rather than silently implying an atomic restore.
    #[test]
    fn restore_argv_has_no_transaction_boundary_and_is_documented_as_such() {
        let config = ConnectionConfig {
            name: "fixture".to_owned(),
            host: "127.0.0.1".to_owned(),
            port: 55432,
            database: "dbpro_fixture".to_owned(),
            username: "dbpro".to_owned(),
            driver: db_pro_core::domain::connection::DriverType::Postgres,
            ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: Vec::new(),
            group: None,
            readonly: false,
        };
        let plain = RestoreOptions {
            connection_id: "c1".to_owned(),
            input_path: "/tmp/dump.sql".to_owned(),
            format: BackupFormat::Plain,
        };
        let custom = RestoreOptions {
            connection_id: "c1".to_owned(),
            input_path: "/tmp/dump.dump".to_owned(),
            format: BackupFormat::Custom,
        };

        let plain_args = PgDumpEngine::restore_command(&plain, &config, "pw")
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let custom_args = PgDumpEngine::restore_command(&custom, &config, "pw")
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(plain_args.first().map(String::as_str), Some("-f"));
        assert_eq!(plain_args.get(1).map(String::as_str), Some("/tmp/dump.sql"));
        assert_eq!(custom_args.first().map(String::as_str), Some("/tmp/dump.dump"));
        for args in [&plain_args, &custom_args] {
            assert!(
                !args.iter().any(|arg| arg.contains("single-transaction")),
                "option (b) is chosen: the restore must not silently become transactional: {args:?}"
            );
            assert!(
                !args.iter().any(|arg| arg == "--exit-on-error"),
                "option (b) is chosen: no exit-on-error flag either: {args:?}"
            );
            assert!(args.windows(2).any(|pair| pair == ["-d", "dbpro_fixture"]), "{args:?}");
        }
    }

    #[tokio::test]
    async fn external_command_timeout_returns_query_timeout() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 1"]);

        let error = PgDumpEngine::run_command(&mut command, 50, "test command")
            .await
            .expect_err("a sleeping command must exceed the test timeout");

        assert!(matches!(error, DbError::QueryTimeout { timeout_ms: 50 }));
    }

    #[tokio::test]
    async fn backup_output_reservation_rejects_existing_file() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!("db-pro-pg-backup-{}.dump", uuid::Uuid::new_v4()));
        tokio::fs::write(&path, b"existing backup").await?;

        let error = match PgDumpEngine::reserve_backup_output(&path).await {
            Ok(()) => return Err("existing backup output was reserved".into()),
            Err(error) => error,
        };

        assert!(matches!(error, DbError::Validation(message) if message.contains("already exists")));
        tokio::fs::remove_file(path).await?;
        Ok(())
    }
}
