//! Tests for `../connection_diagnostics.rs` (moved out of the source file).
use super::*;
use crate::domain::connection::{ConnectionConfig, ConnectionEnvironment, DriverType, SslMode};

fn base_config() -> ConnectionConfig {
    ConnectionConfig {
        name: "t".into(),
        host: "127.0.0.1".into(),
        port: 1,
        database: "db".into(),
        username: "u".into(),
        driver: DriverType::Postgres,
        ssl_mode: SslMode::Require,
        ssh_tunnel: None,
        ssh_profile_id: None,
        ssl_root_cert_path: None,
        ssl_client_cert_path: None,
        ssl_client_key_path: None,
        query_timeout_ms: 30_000,
        max_rows: 1000,
        color: None,
        tags: vec![],
        group: None,
        favorite: false,
        environment: ConnectionEnvironment::Development,
        readonly: false,
    }
}

#[test]
fn redact_strips_password_material() {
    let msg = redact_diag_message("auth failed password=supersecret rest");
    assert!(!msg.contains("supersecret"));
    assert!(msg.contains("[redacted]"));
}

#[test]
fn probe_reports_tcp_failure_without_auth() {
    let report = probe_network_stages(&base_config());
    assert!(report.stages.iter().any(|s| s.stage == ConnectionStage::Dns && s.ok));
    assert!(report.stages.iter().any(|s| s.stage == ConnectionStage::Tcp && !s.ok));
    assert_eq!(report.failed_stage, Some(ConnectionStage::Tcp));
    assert!(!report.summary().to_lowercase().contains("password"));
}

#[test]
fn auth_stage_appended() {
    let report = with_auth_result(probe_network_stages(&base_config()), false, "password=leak");
    let auth = report.stages.iter().find(|s| s.stage == ConnectionStage::Auth).unwrap();
    assert!(!auth.ok);
    assert!(!auth.message.contains("leak"));
}
