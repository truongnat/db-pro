//! Connection-facing UI runtime models.

/// Stable identity for an async UI operation. Real backend tasks will reuse
/// this identity for cancellation and stale-result protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDriver {
    Postgres,
    Sqlite,
    Mysql,
    SqlServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSslMode {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionDraft {
    pub name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub driver: UiDriver,
    pub ssl_mode: UiSslMode,
    pub readonly: bool,
    pub group: String,
    pub tags: String,
    pub favorite: bool,
    pub environment: String,
    pub ssh_tunnel_enabled: bool,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub ssh_private_key: String,
    pub ssh_profile_id: String,
    pub ssl_root_cert_path: String,
    pub ssl_client_cert_path: String,
    pub ssl_client_key_path: String,
    /// Cloud preset key, e.g. `aws_rds:postgres`. Empty = none.
    pub cloud_preset: String,
    /// `password` or `ephemeral_token`.
    pub auth_kind: String,
    pub cloud_snippet: String,
    pub cloud_guidance: String,
}

/// Developer-convenience identity for a new connection draft.
///
/// Debug builds pre-fill the developer's local PostgreSQL fixture so `cargo run`
/// starts from a usable form. The preset must never reach a release build: it
/// names a private developer database, and a shipped app must not pre-fill the
/// New Connection dialog with it.
#[cfg(debug_assertions)]
fn default_draft_identity() -> (String, String, String, String, String) {
    (
        "Xe Lạc Hồng (PostgreSQL)".to_owned(),
        "localhost".to_owned(),
        "fullstack_starter".to_owned(),
        "postgres".to_owned(),
        "postgres".to_owned(),
    )
}

/// Release builds start from a blank identity — see `default_draft_identity`.
#[cfg(not(debug_assertions))]
fn default_draft_identity() -> (String, String, String, String, String) {
    (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    )
}

impl Default for UiConnectionDraft {
    fn default() -> Self {
        let (name, host, database, username, password) = default_draft_identity();
        Self {
            name,
            host,
            port: "5432".to_owned(),
            database,
            username,
            password,
            driver: UiDriver::Postgres,
            // #144: new PostgreSQL connections default to TLS Require. Do not change
            // the Rust `SslMode::default()` / serde fallback — persisted records that
            // omit the field must keep decoding as Disable.
            ssl_mode: UiSslMode::Require,
            readonly: false,
            group: String::new(),
            tags: String::new(),
            favorite: false,
            environment: "Development".to_owned(),
            ssh_tunnel_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_owned(),
            ssh_user: String::new(),
            ssh_private_key: String::new(),
            ssh_profile_id: String::new(),
            ssl_root_cert_path: String::new(),
            ssl_client_cert_path: String::new(),
            ssl_client_key_path: String::new(),
            cloud_preset: String::new(),
            auth_kind: "password".into(),
            cloud_snippet: String::new(),
            cloud_guidance: String::new(),
        }
    }
}
