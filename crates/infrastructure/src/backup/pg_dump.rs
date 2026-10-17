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

    /// Uniquely-named sibling file `pg_dump -f` writes into before publish (#431).
    ///
    /// `pg_dump` resolves `-f` itself, so handing it the caller-visible `output_path` would reopen a
    /// name that was only *reserved*, letting a symlink planted between reserve and open redirect the
    /// dump into an unrelated file. Writing to an unpredictable name in the same directory and then
    /// renaming it over the destination removes that window: a rename replaces whatever sits at
    /// `output_path` (including a planted symlink) rather than following it.
    fn backup_staging_path(output_path: &Path) -> std::path::PathBuf {
        let name = output_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "backup".to_owned());
        output_path.with_file_name(format!(".{name}.pgdump-tmp-{}", uuid::Uuid::new_v4()))
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
    ///
    /// `pgpass_path` is the `.pgpass` file handed over via `PGPASSFILE`; the password itself never
    /// enters the child's environment (#430).
    fn restore_command(options: &RestoreOptions, config: &ConnectionConfig, pgpass_path: &Path) -> Command {
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
            .env("PGPASSFILE", pgpass_path)
            .stdin(Stdio::null());
        cmd
    }
}

/// `.pgpass`-format credential file handed to the child via `PGPASSFILE` (#430).
///
/// `PGPASSWORD` keeps the database password readable through `/proc/<pid>/environ` (and `ps eww`)
/// for the whole dump — minutes for a large database — which is why PostgreSQL's own docs advise
/// against it. libpq also honours `PGPASSFILE`, so the password rides in a 0600 file with an
/// unpredictable name in the system temp dir instead of the child environment; the file is removed
/// when this guard drops, i.e. as soon as the spawned tool exits or the run errors.
///
/// Residual risk: on non-Unix platforms there is no permission check to exploit, so the file relies
/// on its unpredictable name plus the short lifetime.
struct PgPassFile {
    path: std::path::PathBuf,
}

impl PgPassFile {
    /// The pgpass host/database/user fields must match the *effective* connection target — after an
    /// SSH tunnel that is `127.0.0.1`, not the real server — so callers pass the effective config.
    async fn create(config: &ConnectionConfig, password: &str) -> Result<Self, DbError> {
        fn field(value: &str) -> String {
            // `.pgpass` fields are colon-separated; `\` and `:` inside a field must be
            // backslash-escaped.
            value.replace('\\', "\\\\").replace(':', "\\:")
        }
        let contents = format!(
            "{}:{}:{}:{}:{}\n",
            field(&config.host),
            config.port,
            field(&config.database),
            field(&config.username),
            field(password),
        );
        let path = std::env::temp_dir().join(format!("db-pro-pgpass-{}", uuid::Uuid::new_v4()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        // libpq ignores a group/world-accessible pgpass file; create it owner-only.
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        // A few hundred bytes: synchronous write keeps the guard simple. The file must not
        // outlive a failed write — `Self` does not exist yet, so Drop cannot clean it (#483).
        let file = options
            .open(&path)
            .map_err(|error| DbError::Internal(format!("failed to create pgpass file {}: {error}", path.display())))?;
        write_pgpass(file, &path, contents.as_bytes())?;
        Ok(Self { path })
    }

    fn apply(&self, command: &mut Command) {
        command.env("PGPASSFILE", &self.path);
    }
}

/// Writes `contents` to the freshly opened pgpass sink; on failure the partial file is
/// removed because no `PgPassFile` guard exists yet to drop it. The sink is consumed so the
/// handle closes before unlink (Windows refuses to remove an open file).
fn write_pgpass(mut sink: impl std::io::Write, path: &Path, contents: &[u8]) -> Result<(), DbError> {
    if let Err(error) = sink.write_all(contents) {
        drop(sink);
        let _ = std::fs::remove_file(path);
        return Err(DbError::Internal(format!("failed to write pgpass file: {error}")));
    }
    Ok(())
}

impl Drop for PgPassFile {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path) {
            tracing::warn!(path = %self.path.display(), %error, "failed to remove temporary pgpass file");
        }
    }
}

#[async_trait::async_trait]
impl BackupEngine for PgDumpEngine {
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    async fn backup(&self, options: &BackupOptions, password: &str) -> Result<BackupResult, DbError> {
        Self::ensure_tool_available("pg_dump")?;
        let output_path = Path::new(&options.output_path);
        Self::reserve_backup_output(output_path).await?;

        // `pg_dump -f` resolves its output path itself, so it must never see the caller-visible
        // path: the reservation file only claims the name, and a symlink planted between the
        // reservation and pg_dump's open would redirect the dump into an unrelated file (#431).
        // pg_dump writes a uniquely-named sibling instead; a rename then publishes it over the
        // reserved path, replacing (not following) whatever sits there.
        let staging_path = Self::backup_staging_path(output_path);

        let result = async {
            let (config, _tunnel) = self.effective_config().await?;
            let pgpass = PgPassFile::create(&config, password).await?;
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
                .arg(&staging_path);
            pgpass.apply(&mut cmd);

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

            tokio::fs::rename(&staging_path, output_path).await.map_err(|error| {
                DbError::Internal(format!(
                    "pg_dump wrote {} but publishing it to {} failed: {error}",
                    staging_path.display(),
                    output_path.display()
                ))
            })?;

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
            Self::remove_failed_output(&staging_path).await;
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
        // `PgPassFile` must outlive `run_command` — the file is deleted when the guard drops.
        let pgpass = PgPassFile::create(&config, password).await?;
        let mut cmd = Self::restore_command(options, &config, &pgpass.path);

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
