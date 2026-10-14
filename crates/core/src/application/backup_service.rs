use super::registry::ConnectionRegistry;
use crate::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use crate::domain::connection::{Connection, ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::ports::{BackupEngine, ConnectionRepository, IntrospectionCache, SecretStore};
use std::sync::Arc;

type PgEngineFactory = Box<dyn Fn(&ConnectionConfig) -> Box<dyn BackupEngine> + Send + Sync>;
type SqliteEngineFactory = Box<dyn Fn(&str) -> Box<dyn BackupEngine> + Send + Sync>;

pub struct BackupService {
    connections: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    registry: Arc<ConnectionRegistry>,
    introspection_cache: Box<dyn IntrospectionCache>,
    pg_engine_factory: PgEngineFactory,
    sqlite_engine_factory: SqliteEngineFactory,
}

impl BackupService {
    pub fn new(
        connections: Box<dyn ConnectionRepository>,
        secrets: Box<dyn SecretStore>,
        registry: Arc<ConnectionRegistry>,
        introspection_cache: Box<dyn IntrospectionCache>,
        pg_engine_factory: PgEngineFactory,
        sqlite_engine_factory: SqliteEngineFactory,
    ) -> Self {
        Self {
            connections,
            secrets,
            registry,
            introspection_cache,
            pg_engine_factory,
            sqlite_engine_factory,
        }
    }

    pub async fn backup(&self, options: &BackupOptions) -> Result<BackupResult, DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let connection = self.load_connection(&conn_id).await?;
        let mut config = connection.config.clone();
        self.hydrate_ssh_password(&connection, &mut config).await?;
        let password = self.password_for(&connection, "backup").await?;

        let engine = match config.driver {
            DriverType::Postgres => (self.pg_engine_factory)(&config),
            DriverType::SQLite => (self.sqlite_engine_factory)(&config.database),
            DriverType::Mysql => return Err(DbError::Validation("MySQL backup is not yet supported".into())),
            DriverType::SqlServer => {
                return Err(DbError::Unsupported(
                    "SQL Server backup is not available through the shared backup service yet".into(),
                ))
            }
        };

        engine.backup(options, &password).await
    }

    pub async fn restore(&self, options: &RestoreOptions) -> Result<(), DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let connection = self.load_connection(&conn_id).await?;
        let mut config = connection.config.clone();

        // Restore is a mutating operation — block on read-only connections.
        if config.readonly {
            return Err(DbError::QueryFailed(
                "connection is read-only — restore operations are not allowed".into(),
            ));
        }

        if config.driver == DriverType::SQLite && self.registry.is_active(&conn_id) {
            return Err(DbError::Validation(
                "disconnect the SQLite connection before restoring its database".into(),
            ));
        }

        self.hydrate_ssh_password(&connection, &mut config).await?;
        let password = self.password_for(&connection, "restore").await?;

        let engine = match config.driver {
            DriverType::Postgres => (self.pg_engine_factory)(&config),
            DriverType::SQLite => (self.sqlite_engine_factory)(&config.database),
            DriverType::Mysql => return Err(DbError::Validation("MySQL backup is not yet supported".into())),
            DriverType::SqlServer => {
                return Err(DbError::Unsupported(
                    "SQL Server backup is not available through the shared backup service yet".into(),
                ))
            }
        };

        engine.restore(options, &password).await?;

        // The target database now holds the backup's content, so any introspected
        // schema cached for this connection describes a database that no longer
        // exists. Drop it rather than serving it on the next connect.
        if let Err(error) = self.introspection_cache.invalidate(&conn_id).await {
            tracing::warn!(
                connection_id = %conn_id,
                %error,
                "failed to invalidate introspection cache after restore"
            );
        }

        Ok(())
    }

    async fn load_connection(&self, conn_id: &ConnectionId) -> Result<Connection, DbError> {
        self.connections
            .get(conn_id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {conn_id} not found")))
    }

    async fn hydrate_ssh_password(
        &self,
        connection: &Connection,
        config: &mut ConnectionConfig,
    ) -> Result<(), DbError> {
        let Some(ssh_tunnel) = config.ssh_tunnel.as_mut() else {
            return Ok(());
        };
        if ssh_tunnel.password.is_none() {
            let key = format!("connection/{}/ssh_password", connection.id);
            ssh_tunnel.password = self.secrets.retrieve_secret(&key).await?;
        }
        Ok(())
    }

    async fn password_for(&self, connection: &Connection, operation: &str) -> Result<String, DbError> {
        if connection.config.driver == DriverType::SQLite {
            return Ok(String::new());
        }

        let secret_key = connection
            .secret_ref
            .clone()
            .unwrap_or_else(|| format!("connection/{}/password", connection.id));
        self.secrets.retrieve_secret(&secret_key).await?.ok_or_else(|| {
            DbError::Validation(format!(
                "password not found — connect and save credentials before {operation}"
            ))
        })
    }
}

#[cfg(test)]
#[path = "tests/backup_service_tests.rs"]
mod tests;
