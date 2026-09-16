//! Staged connection networking diagnostics (#223).
//!
//! Reports DNS → TCP → SSH → TLS → Auth without embedding secrets.

use crate::domain::connection::{ConnectionConfig, DriverType, SslMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStage {
    Dns,
    Tcp,
    Ssh,
    Tls,
    Auth,
}

impl ConnectionStage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dns => "DNS",
            Self::Tcp => "TCP",
            Self::Ssh => "SSH",
            Self::Tls => "TLS",
            Self::Auth => "DB AUTH",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageResult {
    pub stage: ConnectionStage,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionDiagnosticsReport {
    pub stages: Vec<StageResult>,
    pub failed_stage: Option<ConnectionStage>,
}

impl ConnectionDiagnosticsReport {
    pub fn summary(&self) -> String {
        if let Some(stage) = self.failed_stage {
            format!(
                "Connection failed at {} — {}",
                stage.label(),
                self.stages
                    .iter()
                    .find(|s| s.stage == stage)
                    .map(|s| s.message.as_str())
                    .unwrap_or("unknown")
            )
        } else {
            "All connection stages succeeded".to_owned()
        }
    }
}

/// Redact common secret patterns from diagnostic text.
pub fn redact_diag_message(raw: &str) -> String {
    let mut out = raw.to_owned();
    for needle in ["password=", "PASSWORD ", "private_key", "BEGIN RSA", "BEGIN OPENSSH"] {
        if let Some(idx) = out.to_ascii_lowercase().find(&needle.to_ascii_lowercase()) {
            out = format!("{}[redacted]", &out[..idx]);
            break;
        }
    }
    out
}

/// Run pre-auth network stages locally. Auth is reported from the caller after test_connection.
pub fn probe_network_stages(config: &ConnectionConfig) -> ConnectionDiagnosticsReport {
    let mut stages = Vec::new();
    let mut failed_stage = None;

    // SQLite is local-file — skip network stages.
    if config.driver == DriverType::SQLite {
        stages.push(StageResult {
            stage: ConnectionStage::Dns,
            ok: true,
            message: "SQLite uses a local file path — DNS/TCP/SSH/TLS not applicable".into(),
        });
        return ConnectionDiagnosticsReport {
            stages,
            failed_stage: None,
        };
    }

    let host = config.host.trim();
    if host.is_empty() {
        stages.push(StageResult {
            stage: ConnectionStage::Dns,
            ok: false,
            message: "Host is empty".into(),
        });
        return ConnectionDiagnosticsReport {
            stages,
            failed_stage: Some(ConnectionStage::Dns),
        };
    }

    let addr = format!("{host}:{}", config.port);
    match std::net::ToSocketAddrs::to_socket_addrs(&addr) {
        Ok(mut iter) => {
            if let Some(resolved) = iter.next() {
                stages.push(StageResult {
                    stage: ConnectionStage::Dns,
                    ok: true,
                    message: format!("Resolved {host} → {resolved}"),
                });
            } else {
                stages.push(StageResult {
                    stage: ConnectionStage::Dns,
                    ok: false,
                    message: format!("No addresses resolved for {host}"),
                });
                failed_stage = Some(ConnectionStage::Dns);
                return ConnectionDiagnosticsReport { stages, failed_stage };
            }
        }
        Err(err) => {
            stages.push(StageResult {
                stage: ConnectionStage::Dns,
                ok: false,
                message: redact_diag_message(&format!("DNS failed: {err}")),
            });
            failed_stage = Some(ConnectionStage::Dns);
            return ConnectionDiagnosticsReport { stages, failed_stage };
        }
    }

    let tcp_target = format!("{host}:{}", config.port);
    match std::net::TcpStream::connect_timeout(
        &std::net::ToSocketAddrs::to_socket_addrs(&tcp_target)
            .ok()
            .and_then(|mut i| i.next())
            .unwrap_or_else(|| std::net::SocketAddr::from(([127, 0, 0, 1], config.port))),
        std::time::Duration::from_secs(3),
    ) {
        Ok(_) => stages.push(StageResult {
            stage: ConnectionStage::Tcp,
            ok: true,
            message: format!("TCP connect to {tcp_target} succeeded"),
        }),
        Err(err) => {
            stages.push(StageResult {
                stage: ConnectionStage::Tcp,
                ok: false,
                message: redact_diag_message(&format!("TCP failed: {err}")),
            });
            failed_stage = Some(ConnectionStage::Tcp);
            return ConnectionDiagnosticsReport { stages, failed_stage };
        }
    }

    if config.ssh_tunnel.is_some() || config.ssh_profile_id.is_some() {
        stages.push(StageResult {
            stage: ConnectionStage::Ssh,
            ok: true,
            message: "SSH tunnel configured (validated separately via Test SSH when available)".into(),
        });
    } else {
        stages.push(StageResult {
            stage: ConnectionStage::Ssh,
            ok: true,
            message: "SSH not configured — skipped".into(),
        });
    }

    let tls_msg = match config.ssl_mode {
        SslMode::Disable => "TLS disabled".to_owned(),
        SslMode::Require => "TLS required (encrypted, cert not verified)".to_owned(),
        SslMode::VerifyCa => {
            let ca = config
                .ssl_root_cert_path
                .as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or("(system CAs)");
            format!("TLS VerifyCa using root cert path `{ca}`")
        }
        SslMode::VerifyFull => {
            let ca = config
                .ssl_root_cert_path
                .as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or("(system CAs)");
            format!("TLS VerifyFull (hostname check) using root cert path `{ca}`")
        }
    };
    stages.push(StageResult {
        stage: ConnectionStage::Tls,
        ok: true,
        message: tls_msg,
    });

    ConnectionDiagnosticsReport { stages, failed_stage }
}

pub fn with_auth_result(
    mut report: ConnectionDiagnosticsReport,
    auth_ok: bool,
    auth_message: impl Into<String>,
) -> ConnectionDiagnosticsReport {
    report.stages.push(StageResult {
        stage: ConnectionStage::Auth,
        ok: auth_ok,
        message: redact_diag_message(&auth_message.into()),
    });
    if !auth_ok && report.failed_stage.is_none() {
        report.failed_stage = Some(ConnectionStage::Auth);
    }
    report
}

#[cfg(test)]
mod tests {
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
}
