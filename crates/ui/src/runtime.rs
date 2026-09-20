use serde::{Deserialize, Serialize};

#[path = "runtime_protocol.rs"]
mod protocol;
#[path = "task_bridge.rs"]
mod task_bridge;

pub use protocol::{UiCommand, UiEvent};
pub use task_bridge::TaskBridge;
pub(crate) use task_bridge::MAX_RUNTIME_EVENTS_PER_FRAME;

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiSchemaSummary {
    pub schemas: Vec<String>,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub table_details: Vec<UiTableSummary>,
    pub views: Vec<UiViewSummary>,
    pub triggers: Vec<UiTriggerSummary>,
    pub functions: Vec<UiFunctionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableSummary {
    pub schema: String,
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<UiSchemaColumn>,
    pub foreign_keys: Vec<UiSchemaForeignKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSchemaColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSchemaForeignKey {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiViewSummary {
    pub schema: String,
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTriggerSummary {
    pub schema: String,
    pub name: String,
    pub table_name: String,
    pub timing: String,
    pub event: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRoutineParameter {
    pub name: String,
    pub data_type: String,
    pub mode: String,
    pub has_default: bool,
    pub default_expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFunctionSummary {
    pub schema: String,
    pub name: String,
    pub routine_type: String,
    pub data_type: String,
    pub definition: String,
    pub identity_arguments: String,
    pub language: String,
    pub volatility: String,
    pub security_definer: bool,
    pub parameters: Vec<UiRoutineParameter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableInfo {
    pub schema: String,
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<UiTableColumn>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<UiTableIndex>,
    pub foreign_keys: Vec<UiTableForeignKey>,
    pub check_constraints: Vec<UiCheckConstraint>,
    pub dependencies: Vec<UiTableDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCheckConstraint {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDependencyDirection {
    DependsOn,
    DependedBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDependencyKind {
    Table,
    View,
    ForeignKey,
    Trigger,
    Function,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDependency {
    pub name: String,
    pub schema: String,
    pub kind: UiDependencyKind,
    pub direction: UiDependencyDirection,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UiTableFilterOperator {
    Equals,
    NotEquals,
    #[default]
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDataFilter {
    pub column: String,
    pub data_type: String,
    pub operator: UiTableFilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableDataSort {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTableMutation {
    Update {
        columns: Vec<String>,
        data_types: Vec<String>,
        values: Vec<UiCell>,
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Delete {
        pk_columns: Vec<String>,
        pk_values: Vec<UiCell>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<UiCell>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiTableColumn {
    pub name: String,
    pub data_type: String,
    pub ordinal: usize,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_identity: bool,
    pub is_generated: bool,
    pub collation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub method: String,
    pub primary: bool,
    pub include_columns: Vec<String>,
    pub predicate: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTableForeignKey {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
    pub on_update: String,
    pub on_delete: String,
    pub match_option: String,
    pub deferrable: bool,
    pub initially_deferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiQueryFolderSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSavedQuerySummary {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConnectionSummary {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: String,
    /// The TLS mode stored with the connection. Present so the edit/duplicate
    /// prefill can restore it instead of falling back to a default.
    pub ssl_mode: UiSslMode,
    pub readonly: bool,
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCell {
    Null,
    Boolean(bool),
    Number(String),
    Text(String),
    Json(String),
    Bytes(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiQueryResult {
    pub columns: Vec<UiColumn>,
    pub rows: Vec<Vec<UiCell>>,
    pub row_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiQueryError {
    pub code: String,
    pub message: String,
    pub position: Option<usize>,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiStatementOutput {
    pub statement_index: usize,
    pub result_set: Option<UiQueryResult>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub error: Option<UiQueryError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiQueryExecutionOutput {
    pub statements: Vec<UiStatementOutput>,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiQueryHistoryStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiQueryHistoryEntry {
    pub id: String,
    pub sql: String,
    pub connection_id: Option<String>,
    pub schema: Option<String>,
    pub started_at: String,
    pub duration_ms: u64,
    pub status: UiQueryHistoryStatus,
    pub row_count: Option<u64>,
    pub affected_rows: Option<u64>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    /// Regression guard for the S-1 release-hygiene finding: the developer
    /// connection preset is developer-only. `cargo test --release` exercises the
    /// release half of this assertion.
    #[test]
    fn default_draft_identity_follows_build_profile() {
        let draft = UiConnectionDraft::default();

        if cfg!(debug_assertions) {
            assert_eq!(draft.name, "Xe Lạc Hồng (PostgreSQL)");
            assert_eq!(draft.host, "localhost");
            assert_eq!(draft.database, "fullstack_starter");
            assert_eq!(draft.username, "postgres");
            assert_eq!(draft.password, "postgres");
        } else {
            assert!(
                draft.name.is_empty(),
                "release builds must not pre-fill a connection name"
            );
            assert!(
                draft.host.is_empty(),
                "release builds must not pre-fill a connection host"
            );
            assert!(
                draft.database.is_empty(),
                "release builds must not pre-fill a database name"
            );
            assert!(draft.username.is_empty(), "release builds must not pre-fill a username");
            assert!(draft.password.is_empty(), "release builds must not pre-fill a password");
        }
    }

    /// The neutral parts of the draft are the same in every build profile.
    #[test]
    fn default_draft_neutral_fields_are_profile_independent() {
        let draft = UiConnectionDraft::default();

        assert_eq!(draft.port, "5432");
        assert_eq!(draft.driver, UiDriver::Postgres);
        assert_eq!(draft.ssl_mode, UiSslMode::Require);
        assert!(!draft.readonly);
        assert!(!draft.ssh_tunnel_enabled);
        assert_eq!(draft.ssh_port, "22");
        assert!(draft.ssh_host.is_empty());
        assert!(draft.ssh_user.is_empty());
        assert!(draft.ssh_private_key.is_empty());
    }

    #[test]
    fn request_ids_are_monotonic() {
        let mut bridge = TaskBridge::default();
        assert_eq!(bridge.next_request_id(), RequestId(1));
        assert_eq!(bridge.next_request_id(), RequestId(2));
    }

    #[test]
    fn bridge_delivers_typed_commands() {
        let (command_tx, command_rx) = mpsc::channel();
        let (_event_tx, event_rx) = mpsc::channel();
        let bridge = TaskBridge::new(command_tx, event_rx);
        let command = UiCommand::OpenQuery;
        bridge.send(command.clone()).expect("receiver is alive");
        assert_eq!(command_rx.recv().expect("command expected"), command);
    }

    #[test]
    fn bridge_drains_at_most_the_requested_event_batch() {
        let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
        for request_id in 1..=3 {
            event_tx
                .send(UiEvent::QueryQueued {
                    request_id: RequestId(request_id),
                })
                .expect("event receiver is alive");
        }

        assert_eq!(bridge.drain_events(2).count(), 2);
        assert_eq!(bridge.drain_events(2).count(), 1);
    }
}
