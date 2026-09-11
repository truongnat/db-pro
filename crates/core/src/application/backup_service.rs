use super::registry::ConnectionRegistry;
use crate::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use crate::domain::connection::{Connection, ConnectionConfig, ConnectionId, DriverType};
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

        let connection = self.load_connection(&conn_id).await?;
        let mut config = connection.config.clone();
        self.hydrate_ssh_password(&connection, &mut config).await?;
        let password = self.password_for(&connection, "backup").await?;

        let engine = match config.driver {
            DriverType::Postgres => (self.pg_engine_factory)(&config),
            DriverType::SQLite => (self.sqlite_engine_factory)(&config.database),
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
        };

        engine.restore(options, &password).await
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
        let secret_key = connection
            .secret_ref
            .clone()
            .unwrap_or_else(|| format!("connection/{}/password", connection.id));
        match connection.config.driver {
            DriverType::Postgres => self.secrets.retrieve_secret(&secret_key).await?.ok_or_else(|| {
                DbError::Validation(format!(
                    "password not found — connect and save credentials before {operation}"
                ))
            }),
            DriverType::SQLite => Ok(self.secrets.retrieve_secret(&secret_key).await?.unwrap_or_default()),
        }
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

    fn postgres_config_without_ssh() -> ConnectionConfig {
        let mut config = postgres_config_with_ssh();
        config.ssh_tunnel = None;
        config
    }

    #[tokio::test]
    async fn backup_factory_receives_ssh_configuration() {
        let connection_id = ConnectionId::new();
        let mut connections = MockConnectionRepository::new();
        let config = postgres_config_with_ssh();
        let connection = Connection::new(config.clone()).with_secret_ref("custom/key".into());
        connections
            .expect_get()
            .returning(move |_| Ok(Some(connection.clone())));

        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().times(2).returning(|key| {
            if key == "custom/key" {
                Ok(Some("password".into()))
            } else {
                assert!(key.ends_with("/ssh_password"));
                Ok(Some("ssh-password".into()))
            }
        });

        let saw_ssh_config = Arc::new(Mutex::new(false));
        let saw_ssh_config_for_factory = Arc::clone(&saw_ssh_config);
        let pg_factory = Box::new(move |config: &ConnectionConfig| {
            let ssh_tunnel = config.ssh_tunnel.as_ref().expect("SSH config should be present");
            assert_eq!(ssh_tunnel.password.as_deref(), Some("ssh-password"));
            *saw_ssh_config_for_factory.lock().expect("test mutex poisoned") = true;
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

    #[tokio::test]
    async fn backup_uses_persisted_custom_secret_reference() {
        let connection_id = ConnectionId::new();
        let connection = Connection::new(postgres_config_without_ssh()).with_secret_ref("migrated/password".into());
        let mut connections = MockConnectionRepository::new();
        connections
            .expect_get()
            .returning(move |_| Ok(Some(connection.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .withf(|key| key == "migrated/password")
            .returning(|_| Ok(Some("custom".into())));

        let pg_factory = Box::new(|_: &ConnectionConfig| {
            let mut engine = MockBackupEngine::new();
            engine
                .expect_backup()
                .withf(|_, password| password == "custom")
                .returning(|_, _| {
                    Ok(BackupResult {
                        output_path: "/tmp/backup.dump".into(),
                        size_bytes: 1,
                    })
                });
            Box::new(engine) as Box<dyn BackupEngine>
        });
        let sqlite_factory = Box::new(|_: &str| -> Box<dyn BackupEngine> { Box::new(MockBackupEngine::new()) });
        let service = BackupService::new(
            Box::new(connections),
            Box::new(secrets),
            Arc::new(ConnectionRegistry::new()),
            pg_factory,
            sqlite_factory,
        );

        service
            .backup(&BackupOptions {
                connection_id: connection_id.to_string(),
                output_path: "/tmp/backup.dump".into(),
                format: BackupFormat::Custom,
                schemas: vec![],
                tables: vec![],
            })
            .await
            .expect("backup should use the persisted custom secret");
    }

    #[tokio::test]
    async fn restore_uses_persisted_custom_secret_reference() {
        let connection_id = ConnectionId::new();
        let connection = Connection::new(postgres_config_without_ssh()).with_secret_ref("migrated/password".into());
        let mut connections = MockConnectionRepository::new();
        connections
            .expect_get()
            .returning(move |_| Ok(Some(connection.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .withf(|key| key == "migrated/password")
            .returning(|_| Ok(Some("custom".into())));

        let pg_factory = Box::new(|_: &ConnectionConfig| {
            let mut engine = MockBackupEngine::new();
            engine
                .expect_restore()
                .withf(|_, password| password == "custom")
                .returning(|_, _| Ok(()));
            Box::new(engine) as Box<dyn BackupEngine>
        });
        let sqlite_factory = Box::new(|_: &str| -> Box<dyn BackupEngine> { Box::new(MockBackupEngine::new()) });
        let service = BackupService::new(
            Box::new(connections),
            Box::new(secrets),
            Arc::new(ConnectionRegistry::new()),
            pg_factory,
            sqlite_factory,
        );

        service
            .restore(&RestoreOptions {
                connection_id: connection_id.to_string(),
                input_path: "/tmp/backup.dump".into(),
                format: BackupFormat::Custom,
            })
            .await
            .expect("restore should use the persisted custom secret");
    }
}
