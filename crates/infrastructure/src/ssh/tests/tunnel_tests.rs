//! Tests for `../tunnel.rs` (moved out of the source file).
use super::{SshTunnel, SshTunnelConfig};
use std::ffi::OsStr;
use tokio::process::Command;

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

#[cfg(unix)]
#[tokio::test]
async fn ssh_test_command_has_a_process_timeout() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 1"]);

    let error = SshTunnel::run_test_command(&mut command, std::time::Duration::from_millis(50))
        .await
        .expect_err("a sleeping SSH test command must exceed its deadline");

    assert!(
        matches!(error, db_pro_core::domain::error::DbError::ConnectionTimeout(message) if message.contains("50ms"))
    );
}
