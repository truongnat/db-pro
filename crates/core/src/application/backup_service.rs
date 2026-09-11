use super::registry::ConnectionRegistry;
use crate::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::ports::{BackupEngine, ConnectionRepository, SecretStore};
use std::sync::Arc;

type PgEngineFactory = Box<dyn Fn(&ConnectionConfig) -> Box<dyn BackupEngine> + Send + Sync>;
type SqliteEngineFactory = Box<dyn Fn(&str) -> Box<dyn BackupEngine> + Send + Sync>;

pub struct BackupService {
    connections: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    registry: Arc<ConnectionRegistry>,
    pg_engine_factory: PgEngineFactory,
    sqlite_engine_factory: SqliteEngineFactory,
}

impl BackupService {
    pub fn new(
        connections: Box<dyn ConnectionRepository>,
        secrets: Box<dyn SecretStore>,
        registry: Arc<ConnectionRegistry>,
        pg_engine_factory: PgEngineFactory,
        sqlite_engine_factory: SqliteEngineFactory,
    ) -> Self {
        Self {
            connections,
            secrets,
            registry,
            pg_engine_factory,
            sqlite_engine_factory,
        }
    }

    pub async fn backup(&self, options: &BackupOptions) -> Result<BackupResult, DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let config = self
            .connections
            .get_config(&conn_id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {conn_id} not found")))?;

        let secret_key = format!("connection/{}/password", conn_id);
        let password = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                self.secrets.retrieve_secret(&secret_key).await?.ok_or_else(|| {
                    DbError::Validation("password not found — connect and save credentials before backup".into())
                })?
            }
            crate::domain::connection::DriverType::SQLite => {
                self.secrets.retrieve_secret(&secret_key).await?.unwrap_or_default()
            }
        };

        let engine = match config.driver {
            DriverType::Postgres => (self.pg_engine_factory)(&config),
            DriverType::SQLite => (self.sqlite_engine_factory)(&config.database),
        };

        engine.backup(options, &password).await
    }

    pub async fn restore(&self, options: &RestoreOptions) -> Result<(), DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let config = self
            .connections
            .get_config(&conn_id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {conn_id} not found")))?;

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

        let secret_key = format!("connection/{}/password", conn_id);
        let password = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                self.secrets.retrieve_secret(&secret_key).await?.ok_or_else(|| {
                    DbError::Validation("password not found — connect and save credentials before restore".into())
                })?
            }
            crate::domain::connection::DriverType::SQLite => {
                self.secrets.retrieve_secret(&secret_key).await?.unwrap_or_default()
            }
        };

        let engine = match config.driver {
            DriverType::Postgres => (self.pg_engine_factory)(&config),
            DriverType::SQLite => (self.sqlite_engine_factory)(&config.database),
        };

        engine.restore(options, &password).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::backup::BackupFormat;
    use crate::domain::connection::{DriverType, SshTunnelConfig, SslMode};
    use crate::ports::{MockBackupEngine, MockConnectionRepository, MockSecretStore};
    use std::sync::{Arc, Mutex};

    fn postgres_config_with_ssh() -> ConnectionConfig {
        ConnectionConfig {
            name: "postgres over ssh".into(),
            host: "db.internal".into(),
            port: 5432,
            database: "app".into(),
            username: "app_user".into(),
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: Some(SshTunnelConfig {
                host: "bastion.example".into(),
                port: 22,
                user: "tunnel_user".into(),
                private_key_path: "/tmp/test-key".into(),
                password: None,
            }),
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        }
    }

    #[tokio::test]
    async fn backup_factory_receives_ssh_configuration() {
        let connection_id = ConnectionId::new();
        let mut connections = MockConnectionRepository::new();
        let config = postgres_config_with_ssh();
        connections
            .expect_get_config()
            .returning(move |_| Ok(Some(config.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("password".into())));

        let saw_ssh_config = Arc::new(Mutex::new(false));
        let saw_ssh_config_for_factory = Arc::clone(&saw_ssh_config);
        let pg_factory = Box::new(move |config: &ConnectionConfig| {
            *saw_ssh_config_for_factory.lock().expect("test mutex poisoned") = config.ssh_tunnel.is_some();
            let mut engine = MockBackupEngine::new();
            engine.expect_backup().returning(|_, _| {
                Ok(BackupResult {
                    output_path: "/tmp/backup.dump".into(),
                    size_bytes: 1,
                })
            });
            Box::new(engine) as Box<dyn BackupEngine>
        });
        let sqlite_factory =
            Box::new(|_: &str| -> Box<dyn BackupEngine> { Box::new(MockBackupEngine::new()) as Box<dyn BackupEngine> });
        let service = BackupService::new(
            Box::new(connections),
            Box::new(secrets),
            Arc::new(ConnectionRegistry::new()),
            pg_factory,
            sqlite_factory,
        );

        let result = service
            .backup(&BackupOptions {
                connection_id: connection_id.to_string(),
                output_path: "/tmp/backup.dump".into(),
                format: BackupFormat::Custom,
                schemas: vec![],
                tables: vec![],
            })
            .await;

        assert!(result.is_ok());
        assert!(*saw_ssh_config.lock().expect("test mutex poisoned"));
    }
}
