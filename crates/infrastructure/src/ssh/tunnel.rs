use std::net::TcpListener;
use std::time::Duration;

use db_pro_core::domain::error::DbError;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};

const TUNNEL_READY_TIMEOUT: Duration = Duration::from_secs(10);
const TUNNEL_READY_RETRY: Duration = Duration::from_millis(50);

#[derive(Debug, Clone)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub private_key_path: String,
    pub password: Option<String>,
}

pub struct SshTunnelHandle {
    local_port: u16,
    child: Child,
}

impl SshTunnelHandle {
    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for SshTunnelHandle {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

pub struct SshTunnel;

impl SshTunnel {
    pub async fn start(
        config: &SshTunnelConfig,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<SshTunnelHandle, DbError> {
        let local_port = find_available_port()?;

        let ssh_target = format!("{}@{}", config.user, config.host);
        let port_forward = format!("{local_port}:{remote_host}:{remote_port}");

        let mut cmd = Command::new("ssh");
        cmd.args([
            "-N",
            "-o",
            "ExitOnForwardFailure=yes",
            "-o",
            "ServerAliveInterval=30",
            "-o",
            "ServerAliveCountMax=3",
            "-i",
            &config.private_key_path,
            "-L",
            &port_forward,
            "-p",
            &config.port.to_string(),
            &ssh_target,
        ]);

        if let Some(ref password) = config.password {
            cmd.env("SSHPASS", password);
            let mut sshpass = Command::new("sshpass");
            sshpass.args(["-e"]);
            sshpass.arg("ssh").args([
                "-N",
                "-o",
                "ExitOnForwardFailure=yes",
                "-o",
                "ServerAliveInterval=30",
                "-o",
                "ServerAliveCountMax=3",
                "-i",
                &config.private_key_path,
                "-L",
                &port_forward,
                "-p",
                &config.port.to_string(),
                &ssh_target,
            ]);
            let child = sshpass
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| DbError::ConnectionFailed(format!("failed to start sshpass: {e}")))?;

            return Self::wait_until_ready(local_port, child).await;
        }

        let child = cmd
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| DbError::ConnectionFailed(format!("failed to start ssh: {e}")))?;

        Self::wait_until_ready(local_port, child).await
    }

    pub async fn test(config: &SshTunnelConfig) -> Result<(), DbError> {
        let mut cmd = Self::build_test_command(config);

        let output = cmd
            .output()
            .await
            .map_err(|e| DbError::ConnectionFailed(format!("failed to run ssh: {e}")))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DbError::ConnectionFailed(format!("SSH tunnel test failed: {stderr}")))
        }
    }

    fn build_test_command(config: &SshTunnelConfig) -> Command {
        let ssh_target = format!("{}@{}", config.user, config.host);
        let mut cmd = if let Some(password) = config.password.as_deref() {
            let mut command = Command::new("sshpass");
            command.args(["-e", "ssh"]).env("SSHPASS", password);
            command
        } else {
            Command::new("ssh")
        };
        cmd.args(["-o", "ConnectTimeout=10"]);
        if config.password.is_none() {
            cmd.args(["-o", "BatchMode=yes"]);
        }
        cmd.args([
            "-i",
            &config.private_key_path,
            "-p",
            &config.port.to_string(),
            &ssh_target,
            "echo",
            "ok",
        ]);
        cmd
    }

    async fn wait_until_ready(local_port: u16, mut child: Child) -> Result<SshTunnelHandle, DbError> {
        let deadline = tokio::time::Instant::now() + TUNNEL_READY_TIMEOUT;

        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| DbError::ConnectionFailed(format!("failed to inspect SSH process: {error}")))?
            {
                return Err(DbError::ConnectionFailed(format!(
                    "SSH tunnel exited before becoming ready with status {status}"
                )));
            }

            if TcpStream::connect(("127.0.0.1", local_port)).await.is_ok() {
                return Ok(SshTunnelHandle { local_port, child });
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(DbError::ConnectionTimeout(format!(
                    "SSH tunnel did not open local port {local_port} within {}ms",
                    TUNNEL_READY_TIMEOUT.as_millis()
                )));
            }

            tokio::time::sleep(TUNNEL_READY_RETRY).await;
        }
    }
}

fn find_available_port() -> Result<u16, DbError> {
    TcpListener::bind("127.0.0.1:0")
        .map_err(|e| DbError::ConnectionFailed(format!("failed to find available port: {e}")))
        .and_then(|listener| {
            listener
                .local_addr()
                .map(|addr| addr.port())
                .map_err(|e| DbError::ConnectionFailed(format!("failed to read local addr: {e}")))
        })
}

#[cfg(test)]
mod tests {
    use super::{SshTunnel, SshTunnelConfig};
    use std::ffi::OsStr;

    #[test]
    fn password_ssh_test_uses_sshpass_and_keeps_password_out_of_arguments() {
        let config = SshTunnelConfig {
            host: "bastion.example".into(),
            port: 22,
            user: "deploy".into(),
            private_key_path: "/tmp/key".into(),
            password: Some("secret-value".into()),
        };
        let command = SshTunnel::build_test_command(&config);
        let std_command = command.as_std();
        assert_eq!(std_command.get_program(), OsStr::new("sshpass"));
        let args: Vec<_> = std_command.get_args().collect();
        assert_eq!(args.first(), Some(&OsStr::new("-e")));
        assert_eq!(args.get(1), Some(&OsStr::new("ssh")));
        assert!(!args.iter().any(|arg| *arg == OsStr::new("secret-value")));
        assert!(!args.iter().any(|arg| *arg == OsStr::new("BatchMode=yes")));
        assert!(std_command
            .get_envs()
            .any(|(key, value)| key == OsStr::new("SSHPASS") && value == Some(OsStr::new("secret-value"))));
    }

    #[test]
    fn key_ssh_test_invokes_ssh_directly() {
        let config = SshTunnelConfig {
            host: "bastion.example".into(),
            port: 22,
            user: "deploy".into(),
            private_key_path: "/tmp/key".into(),
            password: None,
        };
        assert_eq!(
            SshTunnel::build_test_command(&config).as_std().get_program(),
            OsStr::new("ssh")
        );
    }
}
