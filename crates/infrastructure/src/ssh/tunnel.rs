use std::net::TcpListener;

use db_pro_core::domain::error::DbError;
use tokio::process::{Child, Command};

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
            "-o", "StrictHostKeyChecking=no",
            "-o", "ServerAliveInterval=30",
            "-o", "ServerAliveCountMax=3",
            "-i", &config.private_key_path,
            "-L", &port_forward,
            "-p", &config.port.to_string(),
            &ssh_target,
        ]);

        if let Some(ref password) = config.password {
            cmd.env("SSHPASS", password);
            let mut sshpass = Command::new("sshpass");
            sshpass.args(["-e"]);
            sshpass.arg("ssh").args([
                "-N",
                "-o", "StrictHostKeyChecking=no",
                "-o", "ServerAliveInterval=30",
                "-o", "ServerAliveCountMax=3",
                "-i", &config.private_key_path,
                "-L", &port_forward,
                "-p", &config.port.to_string(),
                &ssh_target,
            ]);
            let child = sshpass
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| DbError::ConnectionFailed(format!("failed to start sshpass: {e}")))?;

            return Ok(SshTunnelHandle {
                local_port,
                child,
            });
        }

        let child = cmd
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| DbError::ConnectionFailed(format!("failed to start ssh: {e}")))?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(SshTunnelHandle {
            local_port,
            child,
        })
    }

    pub async fn test(config: &SshTunnelConfig) -> Result<(), DbError> {
        let ssh_target = format!("{}@{}", config.user, config.host);

        let mut cmd = Command::new("ssh");
        cmd.args([
            "-o", "StrictHostKeyChecking=no",
            "-o", "ConnectTimeout=10",
            "-o", "BatchMode=yes",
            "-i", &config.private_key_path,
            "-p", &config.port.to_string(),
            &ssh_target,
            "echo", "ok",
        ]);

        let output = cmd
            .output()
            .await
            .map_err(|e| DbError::ConnectionFailed(format!("failed to run ssh: {e}")))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DbError::ConnectionFailed(format!(
                "SSH tunnel test failed: {stderr}"
            )))
        }
    }
}

fn find_available_port() -> Result<u16, DbError> {
    TcpListener::bind("127.0.0.1:0")
        .map(|listener| listener.local_addr().unwrap().port())
        .map_err(|e| DbError::ConnectionFailed(format!("failed to find available port: {e}")))
}
