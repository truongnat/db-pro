mod api;
mod worker;

pub use api::{BackupApi, ConnectionApi, ConnectionSummary, DbErrorDto, ExportApi, QueryApi, SavedQuerySummary, SchemaApi, TableDataApi, UserApi};
pub use worker::{spawn_worker, RuntimeCommand, RuntimeEvent, RuntimeRequestId};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use db_pro_core::application::{
    BackupService, ConnectionRegistry, ConnectionService, ExportService, QueryService, SchemaService, TableDataService,
    UserService,
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
}

#[derive(Debug, Error)]
pub enum RuntimeInitError {
    #[error("failed to create application data directory: {0}")]
    CreateDataDir(#[source] std::io::Error),
    #[error("failed to initialize metadata store: {0}")]
    Metadata(#[from] db_pro_core::domain::error::DbError),
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
        let secret_store = Arc::new(KeyringVault::new("com.dbpro.app", secrets_dir));
        let connector = Arc::new(CompositeConnector::new());
        let registry = Arc::new(ConnectionRegistry::new());

        let connections = Arc::new(ConnectionService::new(
            Box::new(Arc::clone(&connector)),
            Box::new(meta_store.clone()),
            Box::new(Arc::clone(&secret_store)),
            Arc::clone(&registry),
        ));

        let queries = Arc::new(QueryService::new(
            Box::new(Arc::clone(&connector)),
            Box::new(meta_store.clone()),
            Box::new(meta_store.clone()),
            Box::new(meta_store.clone()),
            Arc::clone(&registry),
            Box::new(meta_store.clone()),
        ));

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
        ));
        let backup = Arc::new(BackupService::new(
            Box::new(meta_store.clone()),
            Box::new(Arc::clone(&secret_store)),
            Arc::clone(&registry),
            Box::new(|host, port, database, username| {
                Box::new(PgDumpEngine::new(db_config(
                    db_pro_core::domain::connection::DriverType::Postgres,
                    host,
                    port,
                    database,
                    username,
                )))
            }),
            Box::new(|database| Box::new(SqliteBackupEngine::new(db_config(
                db_pro_core::domain::connection::DriverType::SQLite,
                "",
                0,
                database,
                "",
            )))),
        ));
        let users = Arc::new(UserService::new(
            Box::new(PostgresUserManager::new(connector.postgres_connector())),
            Arc::clone(&registry),
            Box::new(meta_store),
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
