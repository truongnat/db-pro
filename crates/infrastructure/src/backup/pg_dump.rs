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
}

#[async_trait::async_trait]
impl BackupEngine for PgDumpEngine {
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError> {
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

    async fn restore(&self, options: &RestoreOptions, password: &str) -> Result<(), DbError> {
        let (config, _tunnel) = self.effective_config().await?;
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

        let output = Self::run_command(&mut cmd, config.query_timeout_ms, "PostgreSQL restore").await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DbError::Internal(format!("restore failed: {stderr}")));
        }

        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn external_command_timeout_returns_query_timeout() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 1"]);

        let error = PgDumpEngine::run_command(&mut command, 50, "test command")
            .await
            .expect_err("a sleeping command must exceed the test timeout");

        assert!(matches!(error, DbError::QueryTimeout { timeout_ms: 50 }));
    }
}
