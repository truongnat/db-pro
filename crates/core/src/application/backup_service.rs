use crate::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::ports::{BackupEngine, ConnectionRepository, SecretStore};

pub struct BackupService {
    connections: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    pg_engine_factory: Box<dyn Fn(&str, u16, &str, &str) -> Box<dyn BackupEngine> + Send + Sync>,
    sqlite_engine_factory: Box<dyn Fn(&str) -> Box<dyn BackupEngine> + Send + Sync>,
}

impl BackupService {
    pub fn new(
        connections: Box<dyn ConnectionRepository>,
        secrets: Box<dyn SecretStore>,
        pg_engine_factory: Box<dyn Fn(&str, u16, &str, &str) -> Box<dyn BackupEngine> + Send + Sync>,
        sqlite_engine_factory: Box<dyn Fn(&str) -> Box<dyn BackupEngine> + Send + Sync>,
    ) -> Self {
        Self {
            connections,
            secrets,
            pg_engine_factory,
            sqlite_engine_factory,
        }
    }

    pub async fn backup(&self, options: &BackupOptions) -> Result<BackupResult, DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let config = self.connections.get_config(&conn_id).await?
            .ok_or_else(|| DbError::NotFound(format!("connection {conn_id} not found")))?;

        let secret_key = format!("connection/{}/password", conn_id);
        let password = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                self.secrets.retrieve_secret(&secret_key).await?
                    .ok_or_else(|| DbError::Validation(
                        "password not found — connect and save credentials before backup".into()
                    ))?
            }
            crate::domain::connection::DriverType::SQLite => {
                self.secrets.retrieve_secret(&secret_key).await?
                    .unwrap_or_default()
            }
        };

        let engine = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                (self.pg_engine_factory)(&config.host, config.port, &config.database, &config.username)
            }
            crate::domain::connection::DriverType::SQLite => {
                (self.sqlite_engine_factory)(&config.database)
            }
        };

        engine.backup(options, &password).await
    }

    pub async fn restore(&self, options: &RestoreOptions) -> Result<(), DbError> {
        let conn_id = ConnectionId::parse(&options.connection_id)
            .map_err(|e| DbError::Validation(format!("invalid connection id: {e}")))?;

        let config = self.connections.get_config(&conn_id).await?
            .ok_or_else(|| DbError::NotFound(format!("connection {conn_id} not found")))?;

        let secret_key = format!("connection/{}/password", conn_id);
        let password = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                self.secrets.retrieve_secret(&secret_key).await?
                    .ok_or_else(|| DbError::Validation(
                        "password not found — connect and save credentials before restore".into()
                    ))?
            }
            crate::domain::connection::DriverType::SQLite => {
                self.secrets.retrieve_secret(&secret_key).await?
                    .unwrap_or_default()
            }
        };

        let engine = match config.driver {
            crate::domain::connection::DriverType::Postgres => {
                (self.pg_engine_factory)(&config.host, config.port, &config.database, &config.username)
            }
            crate::domain::connection::DriverType::SQLite => {
                (self.sqlite_engine_factory)(&config.database)
            }
        };

        engine.restore(options, &password).await
    }
}
