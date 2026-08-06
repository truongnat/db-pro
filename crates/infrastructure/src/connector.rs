use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use async_trait::async_trait;
use db_pro_core::domain::connection::{ConnectionConfig, ConnectionHandle, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::DbConnector;

use crate::postgres::connector::PostgresConnector;
use crate::sqlite::connector::SQLiteConnector;

pub struct CompositeConnector {
    postgres: PostgresConnector,
    sqlite: SQLiteConnector,
    next_id: AtomicU64,
    handle_driver: RwLock<HashMap<u64, DriverType>>,
    inner_handles: RwLock<HashMap<u64, ConnectionHandle>>,
}

impl CompositeConnector {
    pub fn new() -> Self {
        Self {
            postgres: PostgresConnector::new(),
            sqlite: SQLiteConnector::new(),
            next_id: AtomicU64::new(1),
            handle_driver: RwLock::new(HashMap::new()),
            inner_handles: RwLock::new(HashMap::new()),
        }
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
}

impl Default for CompositeConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DbConnector for CompositeConnector {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
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
}
