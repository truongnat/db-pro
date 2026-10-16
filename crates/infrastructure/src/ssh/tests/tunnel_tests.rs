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

#[cfg(unix)]
#[tokio::test]
async fn password_tunnel_process_receives_environment_without_password_or_empty_identity_arguments() {
    use std::os::unix::fs::PermissionsExt;

    let directory = std::env::temp_dir().join(format!("db-pro-ssh-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let sshpass = directory.join("sshpass");
    std::fs::write(
        &sshpass,
        "#!/bin/sh\n[ \"$SSHPASS\" = 'local-fixture-secret' ] || exit 41\n[ \"$1\" = '-e' ] || exit 42\nshift\nexec \"$@\"\n",
    )
    .unwrap();
    let ssh = directory.join("ssh");
    std::fs::write(
        &ssh,
        "#!/bin/sh\nfor arg do\n  [ \"$arg\" != 'local-fixture-secret' ] || exit 43\n  [ \"$arg\" != '-i' ] || exit 44\ndone\nexit 0\n",
    )
    .unwrap();
    for path in [&sshpass, &ssh] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let config = SshTunnelConfig {
        host: "bastion.example".into(),
        port: 22,
        user: "deploy".into(),
        private_key_path: String::new(),
        password: Some("local-fixture-secret".into()),
    };
    let mut command = SshTunnel::build_start_command(&config, "database.internal", 5432, 15432);
    command.env("PATH", &directory);
    let output = command.output().await.unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        output.status.success(),
        "fixture rejected tunnel process: {:?}",
        output.status
    );
}

#[test]
fn key_tunnel_preserves_identity_and_forward_target() {
    let config = SshTunnelConfig {
        host: "bastion.example".into(),
        port: 2222,
        user: "deploy".into(),
        private_key_path: "/tmp/key with spaces".into(),
        password: None,
    };
    let command = SshTunnel::build_start_command(&config, "database.internal", 3306, 13306);
    let command = command.as_std();
    assert_eq!(command.get_program(), OsStr::new("ssh"));
    let args: Vec<_> = command.get_args().collect();
    assert!(args
        .windows(2)
        .any(|pair| pair == [OsStr::new("-i"), OsStr::new("/tmp/key with spaces")]));
    assert!(args
        .windows(2)
        .any(|pair| pair == [OsStr::new("-L"), OsStr::new("13306:database.internal:3306")]));
}
