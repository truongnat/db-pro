pub mod agent;
mod agent_executor;
mod agent_orchestrator;
mod api;
mod worker;

pub use agent::{AgentContext, AgentDraft, CodexProvider, CodexProviderError, SqlPredictionContext};
pub use agent_executor::{AgentToolExecutor, AgentToolRunner};
pub use agent_orchestrator::{AgentRunOrchestrator, AgentWorkflowEvent};
pub use api::{
    BackupApi, ColumnSummary, ConnectionApi, ConnectionSummary, DataDiffApi, DbErrorDto, ExportApi, ForeignKeySummary,
    FunctionSummary, PostgresApi, QueryApi, QueryFolderSummary, SavedQuerySummary, SchemaApi, SchemaSummary,
    TableDataApi, TableMutationFailure, TableSummary, TriggerSummary, UserApi, ViewSummary,
};
pub use db_pro_core::domain::agent_workflow::AgentExecutionContext;
pub use worker::{spawn_worker, RuntimeCommand, RuntimeEvent, RuntimeRequestId};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use db_pro_core::application::{
    BackupService, ConnectionRegistry, ConnectionService, DataDiffService, ExportService, QueryService, SchemaService,
    TableDataService, UserService,
};
use db_pro_infrastructure::backup::pg_dump::PgDumpEngine;
use db_pro_infrastructure::backup::sqlite_backup::SqliteBackupEngine;
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;
use db_pro_infrastructure::postgres::user_manager::PostgresUserManager;
use db_pro_infrastructure::secret::keyring_vault::KeyringVault;
use thiserror::Error;

/// The application-owned runtime shared by native UI frontends.
///
/// This type deliberately contains no Tauri or egui types. Tauri commands and
/// the native egui task bridge can both depend on these services without one
/// frontend becoming the owner of application wiring.
pub struct DbProRuntime {
    data_dir: PathBuf,
    connections: Arc<ConnectionService>,
    queries: Arc<QueryService>,
    schema: Arc<SchemaService>,
    table_data: Arc<TableDataService>,
    export: Arc<ExportService>,
    backup: Arc<BackupService>,
    users: Arc<UserService>,
    data_diff: Arc<DataDiffService>,
    connector: Arc<CompositeConnector>,
    registry: Arc<ConnectionRegistry>,
    meta_store: SQLiteMetaStore,
}

#[derive(Debug, Error)]
pub enum RuntimeInitError {
    #[error("failed to create application data directory: {0}")]
    CreateDataDir(#[source] std::io::Error),
    #[error("failed to initialize metadata store: {0}")]
    Metadata(#[from] db_pro_core::domain::error::DbError),
}

/// Keyring service name used for every DB Pro secret entry.
pub const KEYRING_SERVICE: &str = "com.dbpro.app";

/// Opt-in that enables the development encrypted-file secret fallback in a release build.
///
/// The file's encryption key is derived from [`KEYRING_SERVICE`], which is not a secret, so the
/// fallback is never enabled in a release build by default (#142).
pub const ALLOW_FILE_SECRET_FALLBACK_ENV: &str = "DB_PRO_ALLOW_FILE_SECRET_FALLBACK";

/// Whether the development encrypted-file secret fallback is enabled for this build.
///
/// Debug builds keep it (development and CI have no OS keyring guarantee); a release build only
/// enables it when [`ALLOW_FILE_SECRET_FALLBACK_ENV`] is set to `1` or `true`. The OS keyring and
/// the in-memory session fallback are always used, in every build.
pub fn file_secret_fallback_enabled() -> bool {
    file_secret_fallback_enabled_for(
        cfg!(debug_assertions),
        std::env::var(ALLOW_FILE_SECRET_FALLBACK_ENV).ok().as_deref(),
    )
}

/// The decision behind [`file_secret_fallback_enabled`], free of build/process-global state.
fn file_secret_fallback_enabled_for(debug_build: bool, opt_in: Option<&str>) -> bool {
    if debug_build {
        return true;
    }
    match opt_in.map(str::trim) {
        Some(value) => value == "1" || value.eq_ignore_ascii_case("true"),
        None => false,
    }
}

/// Build the secret store used by the application.
///
/// The OS keyring is the primary store in every build. The encrypted-file fallback is a
/// development/CI affordance and is only wired in when [`file_secret_fallback_enabled`] allows it;
/// the in-memory session fallback keeps a release build usable when the platform has no keyring
/// item for a key (the password is then asked for again after a restart).
fn build_secret_store(secrets_dir: PathBuf) -> KeyringVault {
    let vault = KeyringVault::new(KEYRING_SERVICE, secrets_dir).with_session_fallback();
    if file_secret_fallback_enabled() {
        tracing::warn!(
            "encrypted-file secret fallback enabled (debug build or {ALLOW_FILE_SECRET_FALLBACK_ENV} set): \
             its key is derived from the service name, so use the OS keyring outside development and CI"
        );
        vault.with_fallback()
    } else {
        vault
    }
}

impl DbProRuntime {
    /// Build the shared service graph once at application startup.
    pub async fn new(data_dir: impl AsRef<Path>) -> Result<Arc<Self>, RuntimeInitError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).map_err(RuntimeInitError::CreateDataDir)?;

        let secrets_dir = data_dir.join("secrets");
        std::fs::create_dir_all(&secrets_dir).map_err(RuntimeInitError::CreateDataDir)?;

        let meta_path = data_dir.join("meta.db");
        let meta_store = SQLiteMetaStore::new(&meta_path.to_string_lossy()).await?;
        let secret_store = Arc::new(build_secret_store(secrets_dir));
        let connector = Arc::new(CompositeConnector::new());
        let registry = Arc::new(ConnectionRegistry::new());

        let connections = Arc::new(
            ConnectionService::new(
                Box::new(Arc::clone(&connector)),
                Box::new(meta_store.clone()),
                Box::new(Arc::clone(&secret_store)),
                Arc::clone(&registry),
            )
            .with_introspection_cache(Box::new(meta_store.clone())),
        );

        let queries = Arc::new(
            QueryService::new(
                Box::new(Arc::clone(&connector)),
                Box::new(meta_store.clone()),
                Box::new(meta_store.clone()),
                Box::new(meta_store.clone()),
                Arc::clone(&registry),
                Box::new(meta_store.clone()),
            )
            .with_introspection_cache(Box::new(meta_store.clone())),
        );

        let schema = Arc::new(SchemaService::new(
            Box::new(Arc::clone(&connector)),
            Box::new(meta_store.clone()),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
        ));

        let table_data = Arc::new(TableDataService::new(
            Box::new(Arc::clone(&connector)),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
        ));
        let export = Arc::new(ExportService::new(
            Box::new(Arc::clone(&connector)),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
        ));
        let backup = Arc::new(BackupService::new(
            Box::new(meta_store.clone()),
            Box::new(Arc::clone(&secret_store)),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
            Box::new(|config| Box::new(PgDumpEngine::new(config.clone()))),
            Box::new(|database| {
                Box::new(SqliteBackupEngine::new(db_config(
                    db_pro_core::domain::connection::DriverType::SQLite,
                    "",
                    0,
                    database,
                    "",
                )))
            }),
        ));
        let users = Arc::new(UserService::new(
            Box::new(PostgresUserManager::new(connector.postgres_connector())),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
        ));
        let data_diff = Arc::new(DataDiffService::new(
            Box::new(Arc::clone(&connector)),
            Arc::clone(&registry),
        ));

        Ok(Arc::new(Self {
            data_dir,
            connections,
            queries,
            schema,
            table_data,
            export,
            backup,
            users,
            data_diff,
            connector,
            registry,
            meta_store,
        }))
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn connections(&self) -> Arc<ConnectionService> {
        Arc::clone(&self.connections)
    }

    pub fn connection_api(&self) -> ConnectionApi {
        ConnectionApi::new(self.connections())
    }

    pub fn queries(&self) -> Arc<QueryService> {
        Arc::clone(&self.queries)
    }

    pub fn query_api(&self) -> QueryApi {
        QueryApi::new(self.queries())
    }

    pub fn schema(&self) -> Arc<SchemaService> {
        Arc::clone(&self.schema)
    }

    pub fn schema_api(&self) -> SchemaApi {
        SchemaApi::new(self.schema())
    }

    pub fn table_data(&self) -> Arc<TableDataService> {
        Arc::clone(&self.table_data)
    }

    pub fn table_data_api(&self) -> TableDataApi {
        TableDataApi::new(self.table_data())
    }

    pub fn export(&self) -> Arc<ExportService> {
        Arc::clone(&self.export)
    }

    pub fn export_api(&self) -> ExportApi {
        ExportApi::new(self.export())
    }

    pub fn backup(&self) -> Arc<BackupService> {
        Arc::clone(&self.backup)
    }

    pub fn backup_api(&self) -> BackupApi {
        BackupApi::new(self.backup())
    }

    pub fn users(&self) -> Arc<UserService> {
        Arc::clone(&self.users)
    }

    pub fn user_api(&self) -> UserApi {
        UserApi::new(self.users())
    }

    pub fn data_diff(&self) -> Arc<DataDiffService> {
        Arc::clone(&self.data_diff)
    }

    pub fn data_diff_api(&self) -> DataDiffApi {
        DataDiffApi::new(self.data_diff())
    }

    pub fn postgres_api(&self) -> PostgresApi {
        PostgresApi::new(self.connector(), self.registry(), self.meta_store.clone())
    }

    pub fn connector(&self) -> Arc<CompositeConnector> {
        Arc::clone(&self.connector)
    }

    pub fn registry(&self) -> Arc<ConnectionRegistry> {
        Arc::clone(&self.registry)
    }

    pub fn meta_store(&self) -> Arc<SQLiteMetaStore> {
        Arc::new(self.meta_store.clone())
    }
}

fn db_config(
    driver: db_pro_core::domain::connection::DriverType,
    host: &str,
    port: u16,
    database: &str,
    username: &str,
) -> db_pro_core::domain::connection::ConnectionConfig {
    db_pro_core::domain::connection::ConnectionConfig {
        name: String::new(),
        host: host.to_owned(),
        port,
        database: database.to_owned(),
        username: username.to_owned(),
        driver,
        ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
        ssh_tunnel: None,
        query_timeout_ms: 30_000,
        max_rows: 500,
        color: None,
        tags: Vec::new(),
        group: None,
        readonly: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_fallback_needs_a_debug_build_or_an_explicit_opt_in() {
        assert!(file_secret_fallback_enabled_for(true, None));
        assert!(file_secret_fallback_enabled_for(true, Some("0")));
        assert!(!file_secret_fallback_enabled_for(false, None));
        assert!(!file_secret_fallback_enabled_for(false, Some("")));
        assert!(!file_secret_fallback_enabled_for(false, Some("0")));
        assert!(!file_secret_fallback_enabled_for(false, Some("yes")));
        assert!(file_secret_fallback_enabled_for(false, Some("1")));
        assert!(file_secret_fallback_enabled_for(false, Some(" true ")));
        assert!(file_secret_fallback_enabled_for(false, Some("TRUE")));
    }

    /// Regression guard for #142: the shipping wiring always selects the OS keyring and the
    /// in-memory session fallback, and selects the encrypted-file fallback only when
    /// [`file_secret_fallback_enabled`] allows it — so the weakly-keyed file can never be
    /// consulted in a release build that did not opt in.
    #[test]
    fn shipping_secret_store_follows_the_build_profile() {
        let secrets_dir = std::env::temp_dir().join(format!("db-pro-runtime-secrets-{}", std::process::id()));
        let vault = build_secret_store(secrets_dir);
        let opt_in = std::env::var(ALLOW_FILE_SECRET_FALLBACK_ENV).ok();

        assert!(
            vault.session_fallback_enabled(),
            "a build without an OS keyring item must degrade to the in-memory session fallback"
        );
        assert_eq!(
            vault.file_fallback_enabled(),
            file_secret_fallback_enabled_for(cfg!(debug_assertions), opt_in.as_deref()),
            "the encrypted-file fallback must follow the build profile and the explicit opt-in only"
        );
    }

    /// The release half of the guard above, compiled only when `debug_assertions` is off, so
    /// `cargo test -p db-pro-runtime --release` is what runs it. It fails deliberately when
    /// [`ALLOW_FILE_SECRET_FALLBACK_ENV`] is set in the environment: the release default is what is
    /// being pinned here, and the gate run does not set the opt-in.
    #[cfg(not(debug_assertions))]
    #[test]
    fn release_secret_store_never_selects_the_file_fallback_by_default() {
        let secrets_dir = std::env::temp_dir().join(format!("db-pro-runtime-release-secrets-{}", std::process::id()));
        let vault = build_secret_store(secrets_dir);

        assert!(
            vault.session_fallback_enabled(),
            "the release store keeps the in-memory session fallback so a missing keyring item is recoverable"
        );
        assert!(
            !vault.file_fallback_enabled(),
            "a release build without {ALLOW_FILE_SECRET_FALLBACK_ENV} must not select the encrypted-file fallback"
        );
    }
}
