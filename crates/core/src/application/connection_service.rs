use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::connection::{Connection, ConnectionConfig, ConnectionHandle, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::ports::{ConnectionRepository, DbConnector, IntrospectionCache, SecretStore};

use super::registry::{ConnectionRegistry, RegisterResult};

pub struct ConnectionService {
    connector: Box<dyn DbConnector>,
    repo: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    registry: Arc<ConnectionRegistry>,
    introspection_cache: Option<Box<dyn IntrospectionCache>>,
    pending_disconnects: Mutex<HashMap<ConnectionId, Vec<ConnectionHandle>>>,
}

impl ConnectionService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        repo: Box<dyn ConnectionRepository>,
        secrets: Box<dyn SecretStore>,
        registry: Arc<ConnectionRegistry>,
    ) -> Self {
        Self {
            connector,
            repo,
            secrets,
            registry,
            introspection_cache: None,
            pending_disconnects: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_introspection_cache(mut self, cache: Box<dyn IntrospectionCache>) -> Self {
        self.introspection_cache = Some(cache);
        self
    }

    fn secret_key(id: &ConnectionId) -> String {
        format!("connection/{}/password", id)
    }

    fn ssh_secret_key(id: &ConnectionId) -> String {
        format!("connection/{}/ssh_password", id)
    }

    fn requires_database_secret(config: &ConnectionConfig) -> bool {
        matches!(config.driver, DriverType::Postgres)
    }

    async fn hydrate_ssh_password(&self, id: &ConnectionId, config: &mut ConnectionConfig) -> Result<(), DbError> {
        let Some(ssh_tunnel) = config.ssh_tunnel.as_mut() else {
            return Ok(());
        };
        if ssh_tunnel.password.is_none() {
            ssh_tunnel.password = self.secrets.retrieve_secret(&Self::ssh_secret_key(id)).await?;
        }
        Ok(())
    }

    pub async fn create(&self, config: ConnectionConfig, password: &str) -> Result<Connection, DbError> {
        validate_config(&config)?;

        if Self::requires_database_secret(&config) && password.is_empty() {
            return Err(DbError::AuthFailed(
                "database password is required for PostgreSQL".into(),
            ));
        }

        let ssh_password = config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh_tunnel| ssh_tunnel.password.clone());
        let mut connection = Connection::new(config);
        let key = Self::secret_key(&connection.id);
        let database_secret_stored = if Self::requires_database_secret(&connection.config) {
            self.secrets.store_secret(&key, password).await?;
            connection = connection.with_secret_ref(key.clone());
            true
        } else {
            false
        };

        let ssh_key = Self::ssh_secret_key(&connection.id);
        if let Some(ssh_password) = ssh_password.as_deref() {
            if let Err(error) = self.secrets.store_secret(&ssh_key, ssh_password).await {
                if let Err(cleanup_error) = self.secrets.delete_secret(&key).await {
                    tracing::error!("failed to clean up database secret after SSH secret failure: {cleanup_error}");
                }
                return Err(error);
            }
        }
        if let Some(ssh_tunnel) = connection.config.ssh_tunnel.as_mut() {
            ssh_tunnel.password = None;
        }

        if let Err(e) = self.repo.save(&connection).await {
            if database_secret_stored {
                if let Err(cleanup_err) = self.secrets.delete_secret(&key).await {
                    tracing::error!("failed to clean up orphan secret after repo save failure: {cleanup_err}");
                }
            }
            if ssh_password.is_some() {
                if let Err(cleanup_err) = self.secrets.delete_secret(&ssh_key).await {
                    tracing::error!("failed to clean up orphan SSH secret after repo save failure: {cleanup_err}");
                }
            }
            return Err(e);
        }

        Ok(connection)
    }

    pub async fn list(&self) -> Result<Vec<Connection>, DbError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: &ConnectionId) -> Result<Option<Connection>, DbError> {
        self.repo.get(id).await
    }

    pub async fn update(
        &self,
        id: &ConnectionId,
        config: ConnectionConfig,
        password: Option<&str>,
    ) -> Result<(), DbError> {
        validate_config(&config)?;

        let mut connection = self
            .repo
            .get(id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {id}")))?;
        let previous = connection.clone();
        let previous_requires_secret = Self::requires_database_secret(&previous.config);
        let new_requires_secret = Self::requires_database_secret(&config);
        if new_requires_secret && !previous_requires_secret && password.is_none() && previous.secret_ref.is_none() {
            return Err(DbError::AuthFailed(
                "database password is required when switching to PostgreSQL".into(),
            ));
        }

        let previous_ssh_password = previous
            .config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh_tunnel| ssh_tunnel.password.clone());
        let new_ssh_password = config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh_tunnel| ssh_tunnel.password.clone());
        let previous_has_ssh = previous.config.ssh_tunnel.is_some();
        let new_has_ssh = config.ssh_tunnel.is_some();
        let migrate_legacy_ssh_password = new_has_ssh && new_ssh_password.is_none() && previous_ssh_password.is_some();
        let ssh_secret_changed =
            (!new_has_ssh && previous_has_ssh) || new_ssh_password.is_some() || migrate_legacy_ssh_password;
        let ssh_key = Self::ssh_secret_key(id);
        let previous_ssh_secret = if ssh_secret_changed && previous_has_ssh {
            self.secrets.retrieve_secret(&ssh_key).await?
        } else {
            None
        };

        connection.config = config;
        connection.updated_at = chrono::Utc::now();

        let key = previous.secret_ref.clone().unwrap_or_else(|| Self::secret_key(id));
        let remove_database_secret =
            !new_requires_secret && (previous_requires_secret || previous.secret_ref.is_some());
        let database_secret_changed = (new_requires_secret && password.is_some()) || remove_database_secret;
        let old_password = if database_secret_changed {
            self.secrets.retrieve_secret(&key).await?
        } else {
            None
        };

        if new_requires_secret {
            if let Some(pw) = password {
                if let Err(error) = self.secrets.store_secret(&key, pw).await {
                    self.restore_secret(&key, old_password.as_deref()).await;
                    return Err(error);
                }
                connection.secret_ref = Some(key.clone());
            }
        } else {
            connection.secret_ref = None;
            if remove_database_secret {
                if let Err(error) = self.secrets.delete_secret(&key).await {
                    self.restore_secret(&key, old_password.as_deref()).await;
                    return Err(error);
                }
            }
        }

        let ssh_mutation = if let Some(ssh_password) = new_ssh_password.as_deref() {
            self.secrets.store_secret(&ssh_key, ssh_password).await
        } else if migrate_legacy_ssh_password {
            match previous_ssh_password.as_deref() {
                Some(ssh_password) => self.secrets.store_secret(&ssh_key, ssh_password).await,
                None => Ok(()),
            }
        } else if !new_has_ssh && previous_has_ssh {
            self.secrets.delete_secret(&ssh_key).await
        } else {
            Ok(())
        };
        if let Err(error) = ssh_mutation {
            if database_secret_changed {
                self.restore_secret(&key, old_password.as_deref()).await;
            }
            if ssh_secret_changed {
                self.restore_secret(&ssh_key, previous_ssh_secret.as_deref()).await;
            }
            return Err(error);
        }
        if let Some(ssh_tunnel) = connection.config.ssh_tunnel.as_mut() {
            ssh_tunnel.password = None;
        }

        if let Err(e) = self.repo.save(&connection).await {
            if database_secret_changed {
                self.restore_secret(&key, old_password.as_deref()).await;
            }
            if ssh_secret_changed {
                self.restore_secret(&ssh_key, previous_ssh_secret.as_deref()).await;
            }
            return Err(e);
        }

        if self.registry.is_active(id) {
            if let Err(error) = self.disconnect(id).await {
                if let Err(rollback_error) = self.repo.save(&previous).await {
                    tracing::error!("failed to restore previous connection after disconnect failure: {rollback_error}");
                }
                if database_secret_changed {
                    self.restore_secret(&key, old_password.as_deref()).await;
                }
                if ssh_secret_changed {
                    self.restore_secret(&ssh_key, previous_ssh_secret.as_deref()).await;
                }
                return Err(error);
            }
        }

        self.invalidate_introspection_cache(id).await;

        Ok(())
    }

    async fn restore_secret(&self, key: &str, previous_password: Option<&str>) {
        let result = match previous_password {
            Some(password) => self.secrets.store_secret(key, password).await,
            None => self.secrets.delete_secret(key).await,
        };
        if let Err(error) = result {
            tracing::error!("failed to restore connection secret after update failure: {error}");
        }
    }

    async fn invalidate_introspection_cache(&self, id: &ConnectionId) {
        if let Some(cache) = self.introspection_cache.as_ref() {
            if let Err(error) = cache.invalidate(id).await {
                tracing::warn!(connection_id = %id, %error, "failed to invalidate introspection cache");
            }
        }
    }

    pub async fn delete(&self, id: &ConnectionId) -> Result<(), DbError> {
        if self.registry.is_active(id) || self.has_pending_disconnects(id).await {
            self.disconnect(id).await?;
        }

        let connection = self.repo.get(id).await?;
        let secret = if let Some(conn) = connection.as_ref() {
            if Self::requires_database_secret(&conn.config) || conn.secret_ref.is_some() {
                let key = conn.secret_ref.clone().unwrap_or_else(|| Self::secret_key(id));
                Some((key.clone(), self.secrets.retrieve_secret(&key).await?))
            } else {
                None
            }
        } else {
            None
        };
        let ssh_secret = if connection.as_ref().is_some_and(|conn| conn.config.ssh_tunnel.is_some()) {
            Some((
                Self::ssh_secret_key(id),
                self.secrets.retrieve_secret(&Self::ssh_secret_key(id)).await?,
            ))
        } else {
            None
        };

        if let Some((key, _)) = secret.as_ref() {
            self.secrets.delete_secret(key).await?;
        }
        if let Some((key, _)) = ssh_secret.as_ref() {
            if let Err(error) = self.secrets.delete_secret(key).await {
                if let Some((db_key, previous_password)) = secret.as_ref() {
                    self.restore_secret(db_key, previous_password.as_deref()).await;
                }
                return Err(error);
            }
        }

        if let Err(error) = self.repo.delete(id).await {
            if let Some((key, previous_password)) = secret {
                self.restore_secret(&key, previous_password.as_deref()).await;
            }
            if let Some((key, previous_password)) = ssh_secret {
                self.restore_secret(&key, previous_password.as_deref()).await;
            }
            return Err(error);
        }

        self.invalidate_introspection_cache(id).await;

        Ok(())
    }

    pub async fn connect(&self, id: &ConnectionId) -> Result<ConnectionHandle, DbError> {
        if let Some(handle) = self.registry.get(id) {
            return Ok(handle);
        }

        let connection = self
            .repo
            .get(id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {id}")))?;

        let password = if Self::requires_database_secret(&connection.config) {
            let secret_key = connection.secret_ref.clone().unwrap_or_else(|| Self::secret_key(id));

            self.secrets
                .retrieve_secret(&secret_key)
                .await?
                .ok_or_else(|| DbError::AuthFailed("password not found in secret store".into()))?
        } else {
            String::new()
        };

        let mut config = connection.config.clone();
        self.hydrate_ssh_password(id, &mut config).await?;
        let handle = self.connector.connect(&config, &password).await?;

        match self.registry.register_or_get(*id, handle) {
            RegisterResult::Inserted => Ok(handle),
            RegisterResult::Existing(existing) => {
                if let Err(error) = self.connector.disconnect(&handle).await {
                    self.pending_disconnects
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .entry(*id)
                        .or_default()
                        .push(handle);
                    return Err(error);
                }
                Ok(existing)
            }
        }
    }

    pub async fn disconnect(&self, id: &ConnectionId) -> Result<(), DbError> {
        let handle = self.registry.get(id);
        let has_pending = self.has_pending_disconnects(id).await;
        if handle.is_none() && !has_pending {
            return Err(DbError::NotFound(format!("connection {id} is not active")));
        }

        if let Some(handle) = handle {
            self.connector.disconnect(&handle).await?;
            self.registry.unregister(id);
        }

        self.disconnect_pending_handles(id).await
    }

    async fn has_pending_disconnects(&self, id: &ConnectionId) -> bool {
        self.pending_disconnects
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .contains_key(id)
    }

    async fn disconnect_pending_handles(&self, id: &ConnectionId) -> Result<(), DbError> {
        let handles = self
            .pending_disconnects
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(id)
            .unwrap_or_default();
        let mut remaining = Vec::new();
        let mut first_error = None;

        for handle in handles {
            if let Err(error) = self.connector.disconnect(&handle).await {
                first_error.get_or_insert(error);
                remaining.push(handle);
            }
        }

        if !remaining.is_empty() {
            self.pending_disconnects
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .entry(*id)
                .or_default()
                .extend(remaining);
        }

        first_error.map_or(Ok(()), Err)
    }

    pub async fn test_connectivity(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        validate_config(config)?;
        self.connector.test_connection(config, password).await
    }

    pub async fn test_connectivity_with_secret(
        &self,
        id: &ConnectionId,
        config: &ConnectionConfig,
        password: &str,
    ) -> Result<(), DbError> {
        let mut config = config.clone();
        validate_config(&config)?;
        self.hydrate_ssh_password(id, &mut config).await?;
        let resolved = if !Self::requires_database_secret(&config) {
            String::new()
        } else if password.is_empty() {
            let secret_key = self
                .repo
                .get(id)
                .await?
                .and_then(|connection| connection.secret_ref)
                .unwrap_or_else(|| Self::secret_key(id));
            self.secrets
                .retrieve_secret(&secret_key)
                .await?
                .ok_or_else(|| DbError::AuthFailed("password not found in secret store".into()))?
        } else {
            password.to_string()
        };
        self.connector.test_connection(&config, &resolved).await
    }
}

fn validate_config(config: &ConnectionConfig) -> Result<(), DbError> {
    config.validate().map_err(|errors| {
        let message = errors
            .iter()
            .map(|error| format!("{}: {}", error.field, error.message))
            .collect::<Vec<_>>()
            .join("; ");
        DbError::Validation(message)
    })
}

#[cfg(test)]
#[path = "connection_service/tests.rs"]
mod tests;
