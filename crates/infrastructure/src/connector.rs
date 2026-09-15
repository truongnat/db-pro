use async_trait::async_trait;
use db_pro_core::domain::capabilities::DatabaseCapabilities;
use db_pro_core::domain::connection::ConnectionConfig;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::{DbConnector, ProviderFactory};
use std::collections::HashMap;
use std::sync::Arc;

use crate::mysql::connector::MySqlConnector;
use crate::postgres::connector::PostgresConnector;
use crate::sqlite::connector::SQLiteConnector;

// ---------------------------------------------------------------------------
// Provider factories — one per driver, registered at startup
// ---------------------------------------------------------------------------

struct PostgresFactory;
#[async_trait]
impl ProviderFactory for PostgresFactory {
    fn driver(&self) -> db_pro_core::domain::connection::DriverType {
        db_pro_core::domain::connection::DriverType::Postgres
    }

    fn capabilities(&self, _config: &ConnectionConfig) -> DatabaseCapabilities {
        DatabaseCapabilities::postgres()
    }

    fn build(&self) -> Box<dyn DbConnector> {
        Box::new(PostgresConnector::new())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        PostgresConnector::new().test_connection(config, password).await
    }
}

struct SqliteFactory;
#[async_trait]
impl ProviderFactory for SqliteFactory {
    fn driver(&self) -> db_pro_core::domain::connection::DriverType {
        db_pro_core::domain::connection::DriverType::SQLite
    }

    fn capabilities(&self, _config: &ConnectionConfig) -> DatabaseCapabilities {
        DatabaseCapabilities::sqlite()
    }

    fn build(&self) -> Box<dyn DbConnector> {
        Box::new(SQLiteConnector::new())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        SQLiteConnector::new().test_connection(config, password).await
    }
}

struct MySqlFactory;
#[async_trait]
impl ProviderFactory for MySqlFactory {
    fn driver(&self) -> db_pro_core::domain::connection::DriverType {
        db_pro_core::domain::connection::DriverType::Mysql
    }

    fn capabilities(&self, _config: &ConnectionConfig) -> DatabaseCapabilities {
        DatabaseCapabilities::mysql()
    }

    fn build(&self) -> Box<dyn DbConnector> {
        Box::new(MySqlConnector::new())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        MySqlConnector::new().test_connection(config, password).await
    }
}

// ---------------------------------------------------------------------------
// SqlDialect implementations
// ---------------------------------------------------------------------------

struct PostgresDialect;
impl db_pro_core::ports::SqlDialect for PostgresDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

struct SqliteDialect;
impl db_pro_core::ports::SqlDialect for SqliteDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

// ---------------------------------------------------------------------------
// CompositeConnector — dispatches via registered factories
// ---------------------------------------------------------------------------

/// A single connection handle and the connector (built by its factory)
/// that owns it. The connector is wrapped in `Arc` so we can clone it
/// and drop the lock before awaiting — `RwLockWriteGuard` is not `Send`.
struct ActiveConnection {
    connector: Arc<dyn DbConnector>,
    inner_handle: db_pro_core::domain::connection::ConnectionHandle,
    driver: db_pro_core::domain::connection::DriverType,
}

pub struct CompositeConnector {
    factories: HashMap<db_pro_core::domain::connection::DriverType, Box<dyn ProviderFactory>>,
    connections: std::sync::RwLock<HashMap<u64, ActiveConnection>>,
    next_id: std::sync::atomic::AtomicU64,
}

impl CompositeConnector {
    pub fn new() -> Self {
        let mut factories: HashMap<db_pro_core::domain::connection::DriverType, Box<dyn ProviderFactory>> =
            HashMap::new();
        factories.insert(
            db_pro_core::domain::connection::DriverType::Postgres,
            Box::new(PostgresFactory),
        );
        factories.insert(
            db_pro_core::domain::connection::DriverType::SQLite,
            Box::new(SqliteFactory),
        );
        factories.insert(
            db_pro_core::domain::connection::DriverType::Mysql,
            Box::new(MySqlFactory),
        );

        Self {
            factories,
            connections: std::sync::RwLock::new(HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Register a new provider factory. Returns the previous factory for
    /// the same driver, if any — so a caller that registers a stub for
    /// testing can restore the original afterwards.
    pub fn register_factory(&mut self, factory: Box<dyn ProviderFactory>) -> Option<Box<dyn ProviderFactory>> {
        self.factories.insert(factory.driver(), factory)
    }

    /// The capabilities advertised for the given connection config.
    /// Branches on capability, not driver type: `composite.capabilities(config)`.
    pub fn capabilities(&self, config: &ConnectionConfig) -> DatabaseCapabilities {
        self.factories
            .get(&config.driver)
            .map(|factory| factory.capabilities(config))
            .unwrap_or_else(|| DatabaseCapabilities::for_driver(config.driver))
    }

    pub fn postgres_connector(&self) -> Arc<PostgresConnector> {
        Arc::new(PostgresConnector::new())
    }

    pub fn inner_postgres_handle(
        &self,
        composite_handle: &db_pro_core::domain::connection::ConnectionHandle,
    ) -> Result<db_pro_core::domain::connection::ConnectionHandle, DbError> {
        let guard = self.connections.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard
            .get(&composite_handle.0)
            .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", composite_handle.0)))?;
        if conn.driver != db_pro_core::domain::connection::DriverType::Postgres {
            return Err(DbError::Validation("connection is not PostgreSQL".into()));
        }
        Ok(conn.inner_handle)
    }

    pub async fn test_ssh_tunnel(
        &self,
        config: &db_pro_core::domain::connection::SshTunnelConfig,
    ) -> Result<(), DbError> {
        use crate::ssh::{SshTunnel, SshTunnelConfig};
        let tunnel_config = SshTunnelConfig {
            host: config.host.clone(),
            port: config.port,
            user: config.user.clone(),
            private_key_path: config.private_key_path.clone(),
            password: config.password.clone(),
        };
        SshTunnel::test(&tunnel_config).await
    }

    /// Clone the connector and inner handle for a connection, so we can
    /// drop the lock before calling an async method — `RwLockWriteGuard`
    /// is not `Send` and cannot be held across an `.await`.
    fn clone_connection(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
    ) -> Result<(Arc<dyn DbConnector>, db_pro_core::domain::connection::ConnectionHandle), DbError> {
        let guard = self.connections.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", handle.0)))?;
        Ok((Arc::clone(&conn.connector), conn.inner_handle))
    }
}

impl Default for CompositeConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DbConnector for CompositeConnector {
    async fn connect(
        &self,
        config: &ConnectionConfig,
        password: &str,
    ) -> Result<db_pro_core::domain::connection::ConnectionHandle, DbError> {
        let factory = self
            .factories
            .get(&config.driver)
            .ok_or_else(|| DbError::Validation(format!("unsupported driver: {:?}", config.driver)))?;
        let connector = factory.build();
        let inner_handle = connector.connect(config, password).await?;

        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.connections.write().unwrap_or_else(|e| e.into_inner()).insert(
            id,
            ActiveConnection {
                connector: Arc::from(connector),
                inner_handle,
                driver: config.driver,
            },
        );

        Ok(db_pro_core::domain::connection::ConnectionHandle(id))
    }

    async fn disconnect(&self, handle: &db_pro_core::domain::connection::ConnectionHandle) -> Result<(), DbError> {
        let (connector, inner) = {
            let mut guard = self.connections.write().unwrap_or_else(|e| e.into_inner());
            let conn = guard
                .remove(&handle.0)
                .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", handle.0)))?;
            (conn.connector, conn.inner_handle)
        };
        connector.disconnect(&inner).await
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        let factory = self
            .factories
            .get(&config.driver)
            .ok_or_else(|| DbError::Validation(format!("unsupported driver: {:?}", config.driver)))?;
        factory.test_connection(config, password).await
    }

    async fn query(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        sql: &str,
        params: &[db_pro_core::domain::query::QueryParam],
    ) -> Result<db_pro_core::domain::query::QueryResult, DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.query(&inner, sql, params).await
    }

    async fn execute(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        sql: &str,
        params: &[db_pro_core::domain::query::QueryParam],
    ) -> Result<u64, DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.execute(&inner, sql, params).await
    }

    async fn cancel(&self, handle: &db_pro_core::domain::connection::ConnectionHandle) -> Result<(), DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.cancel(&inner).await
    }

    async fn execute_batch(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        statements: &[String],
    ) -> Result<u64, DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.execute_batch(&inner, statements).await
    }

    async fn execute_transaction(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        statements: &[String],
        read_statements: &[bool],
    ) -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
        let (connector, inner) =
            self.clone_connection(handle)
                .map_err(|error| db_pro_core::ports::TransactionFailure {
                    phase: db_pro_core::ports::TransactionFailurePhase::Validation,
                    statement_index: 0,
                    outcome: db_pro_core::ports::TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                })?;
        connector.execute_transaction(&inner, statements, read_statements).await
    }

    async fn execute_parameterized_transaction(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        statements: &[db_pro_core::ports::ParameterizedTransactionStatement],
    ) -> Result<Vec<db_pro_core::ports::TransactionStatementResult>, db_pro_core::ports::TransactionFailure> {
        let (connector, inner) =
            self.clone_connection(handle)
                .map_err(|error| db_pro_core::ports::TransactionFailure {
                    phase: db_pro_core::ports::TransactionFailurePhase::Validation,
                    statement_index: 0,
                    outcome: db_pro_core::ports::TransactionFailureOutcome::NotStarted,
                    results: Vec::new(),
                    error,
                })?;
        connector.execute_parameterized_transaction(&inner, statements).await
    }

    async fn introspect(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
    ) -> Result<db_pro_core::domain::schema::IntrospectResult, DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.introspect(&inner).await
    }

    async fn explain(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
        sql: &str,
    ) -> Result<serde_json::Value, DbError> {
        let (connector, inner) = self.clone_connection(handle)?;
        connector.explain(&inner, sql).await
    }

    fn dialect(
        &self,
        handle: &db_pro_core::domain::connection::ConnectionHandle,
    ) -> Result<Box<dyn db_pro_core::ports::SqlDialect>, DbError> {
        let guard = self.connections.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard
            .get(&handle.0)
            .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", handle.0)))?;
        Ok(match conn.driver {
            db_pro_core::domain::connection::DriverType::Postgres => Box::new(PostgresDialect),
            db_pro_core::domain::connection::DriverType::SQLite => Box::new(SqliteDialect),
            db_pro_core::domain::connection::DriverType::Mysql => {
                return Err(DbError::Validation("MySQL dialect not yet implemented".into()))
            }
        })
    }
}
