use super::*;

fn tunneled_draft(driver: UiDriver) -> UiConnectionDraft {
    UiConnectionDraft {
        name: "tunnel regression".into(),
        driver,
        host: "database.internal".into(),
        port: "3306".into(),
        database: "app".into(),
        username: "app".into(),
        password: "database-secret".into(),
        ssh_tunnel_enabled: true,
        ssh_host: "bastion.example".into(),
        ssh_port: "2222".into(),
        ssh_user: "deploy".into(),
        ssh_private_key: "/tmp/key".into(),
        ..Default::default()
    }
}

#[test]
fn mysql_test_create_and_update_preserve_tunnel_intent() {
    let draft = tunneled_draft(UiDriver::Mysql);
    let request_id = RequestId(71);
    for command in [
        UiCommand::TestConnection {
            request_id,
            draft: draft.clone(),
        },
        UiCommand::CreateConnection {
            request_id,
            draft: draft.clone(),
        },
        UiCommand::UpdateConnection {
            request_id,
            connection_id: "saved".into(),
            draft,
        },
    ] {
        let config = match translate_command(command).expect("translated connection command") {
            RuntimeCommand::TestConnection { config, .. }
            | RuntimeCommand::CreateConnection { config, .. }
            | RuntimeCommand::UpdateConnection { config, .. } => config,
            _ => panic!("unexpected command family"),
        };
        assert!(config.validate().is_ok());
        let tunnel = config.ssh_tunnel.expect("MySQL must not silently connect directly");
        assert_eq!(tunnel.host, "bastion.example");
        assert_eq!(tunnel.port, 2222);
        assert_eq!(tunnel.user, "deploy");
        assert_eq!(config.host, "database.internal");
    }
}

#[test]
fn sqlserver_tunnel_reaches_explicit_domain_rejection_instead_of_direct_connect() {
    let draft = tunneled_draft(UiDriver::SqlServer);
    let request_id = RequestId(72);
    for command in [
        UiCommand::TestConnection {
            request_id,
            draft: draft.clone(),
        },
        UiCommand::CreateConnection {
            request_id,
            draft: draft.clone(),
        },
        UiCommand::UpdateConnection {
            request_id,
            connection_id: "saved".into(),
            draft,
        },
    ] {
        let config = match translate_command(command).expect("translated connection command") {
            RuntimeCommand::TestConnection { config, .. }
            | RuntimeCommand::CreateConnection { config, .. }
            | RuntimeCommand::UpdateConnection { config, .. } => config,
            _ => panic!("unexpected command family"),
        };
        assert!(config.ssh_tunnel.is_some());
        let errors = config.validate().expect_err("SQL Server SSH is unsupported");
        assert!(errors.iter().any(|error| error.field == "ssh_tunnel"));
    }
}

#[test]
fn disabled_network_tunnel_remains_direct() {
    for driver in [UiDriver::Postgres, UiDriver::Mysql, UiDriver::SqlServer] {
        let mut draft = tunneled_draft(driver);
        draft.ssh_tunnel_enabled = false;
        draft.ssh_port = "unused invalid port".into();
        let (config, _) = draft_to_domain(draft).unwrap();
        assert!(config.ssh_tunnel.is_none());
        assert!(config.validate().is_ok());
    }
}
