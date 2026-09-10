use std::path::{Path, PathBuf};
use std::sync::Arc;

use db_pro_core::application::{ConnectionService, ConnectionRegistry, QueryService, SchemaService};
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;
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
            Box::new(meta_store),
        ));

        Ok(Arc::new(Self {
            data_dir,
            connections,
            queries,
            schema,
        }))
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn connections(&self) -> Arc<ConnectionService> {
        Arc::clone(&self.connections)
    }

    pub fn queries(&self) -> Arc<QueryService> {
        Arc::clone(&self.queries)
    }

    pub fn schema(&self) -> Arc<SchemaService> {
        Arc::clone(&self.schema)
    }
}
