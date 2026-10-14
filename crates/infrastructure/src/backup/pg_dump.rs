// cc-scan:allow-file LINE_TOO_LONG — SQL literals stay single-line; wrapping changes statement text
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

    /// Detect `pg_dump` / `pg_restore` / `psql` on PATH before spawning (#208).
    pub fn ensure_tool_available(tool: &str) -> Result<(), DbError> {
        Self::which_tool(tool).map(|_| ()).map_err(|hint| {
            DbError::Validation(format!(
                "{tool} was not found on PATH. Install PostgreSQL client tools and ensure `{tool}` is available. {hint}"
            ))
        })
    }

    fn which_tool(tool: &str) -> Result<std::path::PathBuf, String> {
        if let Ok(path) = std::env::var(format!("{}_PATH", tool.to_ascii_uppercase())) {
            let candidate = Path::new(&path);
            if candidate.is_file() {
                return Ok(candidate.to_path_buf());
            }
        }
        let path_env = std::env::var_os("PATH").unwrap_or_default();
        for dir in std::env::split_paths(&path_env) {
            let candidate = dir.join(tool);
            if candidate.is_file() {
                return Ok(candidate);
            }
            #[cfg(windows)]
            {
                let exe = dir.join(format!("{tool}.exe"));
                if exe.is_file() {
                    return Ok(exe);
                }
            }
        }
        Err(format!("Searched PATH but did not find `{tool}`."))
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
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError> {
        Self::ensure_tool_available("pg_dump")?;
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
        let tool = match options.format {
            BackupFormat::Plain => "psql",
            BackupFormat::Custom => "pg_restore",
        };
        Self::ensure_tool_available(tool)?;
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
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                "restore failed: {stderr} -- the restore was NOT run in a transaction, so the target database may now be \
                 partially restored; inspect it before using it (documented in LIM-020). The full psql/pg_restore output is above"
            )));
        }

        Ok(())
    }
}

#[cfg(all(test, unix))]
#[path = "tests/pg_dump_tests.rs"]
mod tests;
