use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{DbConnector, SqlDialect};

use crate::postgres::connector::PostgresConnector;
use crate::sqlite::connector::SQLiteConnector;
use crate::ssh::{SshTunnel, SshTunnelConfig, SshTunnelHandle};

struct PostgresDialect;
impl SqlDialect for PostgresDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

struct SqliteDialect;
impl SqlDialect for SqliteDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }
}

pub struct CompositeConnector {
    postgres: Arc<PostgresConnector>,
    sqlite: SQLiteConnector,
    next_id: AtomicU64,
    handle_driver: RwLock<HashMap<u64, DriverType>>,
    inner_handles: RwLock<HashMap<u64, ConnectionHandle>>,
    active_tunnels: RwLock<HashMap<u64, SshTunnelHandle>>,
}

impl CompositeConnector {
    pub fn new() -> Self {
        Self {
            postgres: Arc::new(PostgresConnector::new()),
            sqlite: SQLiteConnector::new(),
            next_id: AtomicU64::new(1),
            handle_driver: RwLock::new(HashMap::new()),
            inner_handles: RwLock::new(HashMap::new()),
            active_tunnels: RwLock::new(HashMap::new()),
        }
    }

    pub fn postgres_connector(&self) -> Arc<PostgresConnector> {
        Arc::clone(&self.postgres)
    }

    pub fn inner_postgres_handle(&self, composite_handle: &ConnectionHandle) -> Result<ConnectionHandle, DbError> {
        let driver = self.driver_of(composite_handle)?;
        if driver != DriverType::Postgres {
            return Err(DbError::Validation("connection is not PostgreSQL".into()));
        }
        self.inner_handle(composite_handle)
    }

    fn inner_handle(&self, handle: &ConnectionHandle) -> Result<ConnectionHandle, DbError> {
        self.inner_handles
            .read()
            .unwrap()
            .get(&handle.0)
            .copied()
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {} is not active", handle.0)))
    }

    fn driver_of(&self, handle: &ConnectionHandle) -> Result<DriverType, DbError> {
        self.handle_driver
            .read()
            .unwrap()
            .get(&handle.0)
            .copied()
            .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", handle.0)))
    }

    pub async fn test_ssh_tunnel(&self, config: &db_pro_core::domain::connection::SshTunnelConfig) -> Result<(), DbError> {
        let tunnel_config = SshTunnelConfig {
            host: config.host.clone(),
            port: config.port,
            user: config.user.clone(),
            private_key_path: config.private_key_path.clone(),
            password: config.password.clone(),
        };
        SshTunnel::test(&tunnel_config).await
    }
}

impl Default for CompositeConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DbConnector for CompositeConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        let mut effective_config = config.clone();

        if let Some(ref ssh_config) = config.ssh_tunnel {
            let tunnel_config = SshTunnelConfig {
                host: ssh_config.host.clone(),
                port: ssh_config.port,
                user: ssh_config.user.clone(),
                private_key_path: ssh_config.private_key_path.clone(),
                password: ssh_config.password.clone(),
            };
            let tunnel = SshTunnel::start(&tunnel_config, &config.host, config.port).await?;
            let local_port = tunnel.local_port();

            effective_config.host = "127.0.0.1".to_string();
            effective_config.port = local_port;

            let (driver, inner_handle) = match config.driver {
                DriverType::Postgres => {
                    let h = self.postgres.connect(&effective_config, password).await?;
                    (DriverType::Postgres, h)
                }
                DriverType::SQLite => {
                    let h = self.sqlite.connect(&effective_config, password).await?;
                    (DriverType::SQLite, h)
                }
            };

            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            self.handle_driver.write().unwrap().insert(id, driver);
            self.inner_handles.write().unwrap().insert(id, inner_handle);
            self.active_tunnels.write().unwrap().insert(id, tunnel);

            return Ok(ConnectionHandle(id));
        }

        let (driver, inner_handle) = match config.driver {
            DriverType::Postgres => {
                let h = self.postgres.connect(config, password).await?;
                (DriverType::Postgres, h)
            }
            DriverType::SQLite => {
                let h = self.sqlite.connect(config, password).await?;
                (DriverType::SQLite, h)
            }
        };

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.handle_driver.write().unwrap().insert(id, driver);
        self.inner_handles.write().unwrap().insert(id, inner_handle);

        Ok(ConnectionHandle(id))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        let driver = self
            .handle_driver
            .read()
            .unwrap()
            .get(&handle.0)
            .copied()
            .ok_or_else(|| DbError::ConnectionFailed(format!("unknown connection handle {}", handle.0)))?;

        let inner = self
            .inner_handles
            .read()
            .unwrap()
            .get(&handle.0)
            .copied()
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {} is not active", handle.0)))?;

        match driver {
            DriverType::Postgres => self.postgres.disconnect(&inner).await?,
            DriverType::SQLite => self.sqlite.disconnect(&inner).await?,
        }

        self.handle_driver.write().unwrap().remove(&handle.0);
        self.inner_handles.write().unwrap().remove(&handle.0);
        self.active_tunnels.write().unwrap().remove(&handle.0);

        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        match config.driver {
            DriverType::Postgres => self.postgres.test_connection(config, password).await,
            DriverType::SQLite => self.sqlite.test_connection(config, password).await,
        }
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        let inner = self.inner_handle(handle)?;
        let driver = self.driver_of(handle)?;
        match driver {
            DriverType::Postgres => self.postgres.query(&inner, sql, params).await,
            DriverType::SQLite => self.sqlite.query(&inner, sql, params).await,
        }
    }

    async fn execute(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<u64, DbError> {
        let inner = self.inner_handle(handle)?;
        let driver = self.driver_of(handle)?;
        match driver {
            DriverType::Postgres => self.postgres.execute(&inner, sql, params).await,
            DriverType::SQLite => self.sqlite.execute(&inner, sql, params).await,
        }
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        let inner = self.inner_handle(handle)?;
        let driver = self.driver_of(handle)?;
        match driver {
            DriverType::Postgres => self.postgres.introspect(&inner).await,
            DriverType::SQLite => self.sqlite.introspect(&inner).await,
        }
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        let inner = self.inner_handle(handle)?;
        let driver = self.driver_of(handle)?;
        match driver {
            DriverType::Postgres => self.postgres.explain(&inner, sql).await,
            DriverType::SQLite => self.sqlite.explain(&inner, sql).await,
        }
    }

    fn dialect(&self, handle: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError> {
        match self.driver_of(handle)? {
            DriverType::Postgres => Ok(Box::new(PostgresDialect)),
            DriverType::SQLite => Ok(Box::new(SqliteDialect)),
        }
    }
}
