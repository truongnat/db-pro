use super::*;
use crate::domain::connection::SshTunnelConfig;
use std::sync::{Arc, Mutex};

fn ssh_tunnel() -> SshTunnelConfig {
    SshTunnelConfig {
        host: "bastion.example".into(),
        port: 22,
        user: "deploy".into(),
        private_key_path: "/tmp/key".into(),
        password: None,
    }
}

#[tokio::test]
async fn create_stores_ssh_password_separately_and_sanitizes_metadata() {
    let mut config = test_config();
    config.ssh_tunnel = Some(SshTunnelConfig {
        password: Some("ssh-secret".into()),
        ..ssh_tunnel()
    });
    let saved = Arc::new(Mutex::new(None));
    let mut repo = MockConnectionRepository::new();
    repo.expect_save().returning({
        let saved = Arc::clone(&saved);
        move |connection| {
            *saved.lock().expect("saved connection lock") = Some(connection.clone());
            Ok(())
        }
    });

    let mut secrets = MockSecretStore::new();
    secrets.expect_store_secret().times(2).returning(|key, value| {
        if key.ends_with("/password") {
            assert_eq!(value, "pass");
        } else {
            assert!(key.ends_with("/ssh_password"));
            assert_eq!(value, "ssh-secret");
        }
        Ok(())
    });

    let svc = build_service(MockDbConnector::new(), repo, secrets);
    let connection = svc.create(config, "pass").await.expect("create should succeed");
    assert_eq!(
        connection
            .config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh| ssh.password.as_deref()),
        None
    );
    assert_eq!(
        saved
            .lock()
            .expect("saved connection lock")
            .as_ref()
            .and_then(|connection| connection.config.ssh_tunnel.as_ref())
            .and_then(|ssh| ssh.password.as_deref()),
        None
    );
}

#[tokio::test]
async fn update_stores_new_ssh_password_and_sanitizes_metadata() {
    let id = ConnectionId::new();
    let previous = Connection::new(test_config());
    let saved = Arc::new(Mutex::new(None));
    let mut repo = MockConnectionRepository::new();
    repo.expect_get().returning(move |_| Ok(Some(previous.clone())));
    repo.expect_save().returning({
        let saved = Arc::clone(&saved);
        move |connection| {
            *saved.lock().expect("saved connection lock") = Some(connection.clone());
            Ok(())
        }
    });

    let mut config = test_config();
    config.ssh_tunnel = Some(SshTunnelConfig {
        password: Some("new-ssh-password".into()),
        ..ssh_tunnel()
    });
    let mut secrets = MockSecretStore::new();
    secrets.expect_retrieve_secret().returning(|key| {
        assert!(key.ends_with("/password"));
        Ok(None)
    });
    secrets.expect_store_secret().times(2).returning(|key, value| {
        if key.ends_with("/password") {
            assert_eq!(value, "new-db-password");
        } else {
            assert!(key.ends_with("/ssh_password"));
            assert_eq!(value, "new-ssh-password");
        }
        Ok(())
    });

    let svc = build_service(MockDbConnector::new(), repo, secrets);
    svc.update(&id, config, Some("new-db-password"))
        .await
        .expect("update should succeed");
    assert_eq!(
        saved
            .lock()
            .expect("saved connection lock")
            .as_ref()
            .and_then(|connection| connection.config.ssh_tunnel.as_ref())
            .and_then(|ssh| ssh.password.as_deref()),
        None
    );
}
