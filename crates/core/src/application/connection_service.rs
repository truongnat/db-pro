use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::connection::{Connection, ConnectionConfig, ConnectionHandle, ConnectionId};
use crate::domain::error::DbError;
use crate::ports::{ConnectionRepository, DbConnector, SecretStore};

use super::registry::{ConnectionRegistry, RegisterResult};

pub struct ConnectionService {
    connector: Box<dyn DbConnector>,
    repo: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    registry: Arc<ConnectionRegistry>,
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
            pending_disconnects: Mutex::new(HashMap::new()),
        }
    }

    fn secret_key(id: &ConnectionId) -> String {
        format!("connection/{}/password", id)
    }

    pub async fn create(&self, config: ConnectionConfig, password: &str) -> Result<Connection, DbError> {
        if let Err(errors) = config.validate() {
            let msg = errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(DbError::Validation(msg));
        }

        let mut connection = Connection::new(config);
        let key = Self::secret_key(&connection.id);
        self.secrets.store_secret(&key, password).await?;
        connection = connection.with_secret_ref(key.clone());

        if let Err(e) = self.repo.save(&connection).await {
            if let Err(cleanup_err) = self.secrets.delete_secret(&key).await {
                tracing::error!("failed to clean up orphan secret after repo save failure: {cleanup_err}");
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
        if let Err(errors) = config.validate() {
            let msg = errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(DbError::Validation(msg));
        }

        let mut connection = self
            .repo
            .get(id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {id}")))?;
        let previous = connection.clone();

        connection.config = config;
        connection.updated_at = chrono::Utc::now();

        let key = connection.secret_ref.clone().unwrap_or_else(|| Self::secret_key(id));
        let old_password = if password.is_some() {
            self.secrets.retrieve_secret(&key).await?
        } else {
            None
        };

        if let Some(pw) = password {
            if let Err(error) = self.secrets.store_secret(&key, pw).await {
                self.restore_secret(&key, old_password.as_deref()).await;
                return Err(error);
            }
            connection.secret_ref = Some(key.clone());
        }

        if let Err(e) = self.repo.save(&connection).await {
            if password.is_some() {
                self.restore_secret(&key, old_password.as_deref()).await;
            }
            return Err(e);
        }

        if self.registry.is_active(id) {
            if let Err(error) = self.disconnect(id).await {
                if let Err(rollback_error) = self.repo.save(&previous).await {
                    tracing::error!("failed to restore previous connection after disconnect failure: {rollback_error}");
                }
                if password.is_some() {
                    self.restore_secret(&key, old_password.as_deref()).await;
                }
                return Err(error);
            }
        }

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

    pub async fn delete(&self, id: &ConnectionId) -> Result<(), DbError> {
        if self.registry.is_active(id) || self.has_pending_disconnects(id).await {
            self.disconnect(id).await?;
        }

        let connection = self.repo.get(id).await?;
        let secret = if let Some(conn) = connection.as_ref() {
            if let Some(key) = conn.secret_ref.as_deref() {
                Some((key.to_owned(), self.secrets.retrieve_secret(key).await?))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((key, _)) = secret.as_ref() {
            self.secrets.delete_secret(key).await?;
        }

        if let Err(error) = self.repo.delete(id).await {
            if let Some((key, previous_password)) = secret {
                self.restore_secret(&key, previous_password.as_deref()).await;
            }
            return Err(error);
        }

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

        let secret_key = connection
            .secret_ref
            .as_deref()
            .ok_or_else(|| DbError::AuthFailed("no secret_ref on connection".into()))?;

        let password = self
            .secrets
            .retrieve_secret(secret_key)
            .await?
            .ok_or_else(|| DbError::AuthFailed("password not found in secret store".into()))?;

        let handle = self.connector.connect(&connection.config, &password).await?;

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
        self.connector.test_connection(config, password).await
    }

    pub async fn test_connectivity_with_secret(
        &self,
        id: &ConnectionId,
        config: &ConnectionConfig,
        password: &str,
    ) -> Result<(), DbError> {
        let resolved = if password.is_empty() {
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
        self.connector.test_connection(config, &resolved).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::{ConnectionConfig, DriverType, SslMode};
    use crate::ports::{MockConnectionRepository, MockDbConnector, MockSecretStore};

    fn test_config() -> ConnectionConfig {
        ConnectionConfig {
            name: "test".into(),
            host: "localhost".into(),
            port: 5432,
            database: "testdb".into(),
            username: "user".into(),
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        }
    }

    fn build_service(
        connector: MockDbConnector,
        repo: MockConnectionRepository,
        secrets: MockSecretStore,
    ) -> ConnectionService {
        ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(secrets),
            Arc::new(ConnectionRegistry::new()),
        )
    }

    #[tokio::test]
    async fn create_valid_connection() {
        let config = test_config();
        let mut repo = MockConnectionRepository::new();
        repo.expect_save().returning(|_| Ok(()));

        let mut secrets = MockSecretStore::new();
        secrets.expect_store_secret().returning(|_, _| Ok(()));

        let svc = build_service(MockDbConnector::new(), repo, secrets);
        let result = svc.create(config, "pass").await;
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(conn.secret_ref.is_some());
    }

    #[tokio::test]
    async fn create_invalid_config_rejected() {
        let mut config = test_config();
        config.name = String::new();

        let svc = build_service(
            MockDbConnector::new(),
            MockConnectionRepository::new(),
            MockSecretStore::new(),
        );
        let result = svc.create(config, "pass").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_connections() {
        let mut repo = MockConnectionRepository::new();
        repo.expect_list().returning(|| Ok(vec![]));

        let svc = build_service(MockDbConnector::new(), repo, MockSecretStore::new());
        let result = svc.list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn connect_success() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("key".into());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(conn.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("password".into())));

        let mut connector = MockDbConnector::new();
        connector.expect_connect().returning(|_, _| Ok(ConnectionHandle(1)));

        let registry = Arc::new(ConnectionRegistry::new());
        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(secrets),
            Arc::clone(&registry),
        );

        let handle = svc.connect(&id).await.unwrap();
        assert_eq!(handle, ConnectionHandle(1));
        assert!(registry.is_active(&id));
    }

    #[tokio::test]
    async fn connect_already_active_returns_existing_handle() {
        let id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));

        let svc = ConnectionService::new(
            Box::new(MockDbConnector::new()),
            Box::new(MockConnectionRepository::new()),
            Box::new(MockSecretStore::new()),
            Arc::clone(&registry),
        );

        let result = svc.connect(&id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConnectionHandle(1));
    }

    #[tokio::test]
    async fn connect_duplicate_disconnect_failure_propagates() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("key".into());
        let registry = Arc::new(ConnectionRegistry::new());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(conn.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("password".into())));

        let reg_for_mock = Arc::clone(&registry);
        let mut connector = MockDbConnector::new();
        connector.expect_connect().returning(move |_, _| {
            reg_for_mock.register(id, ConnectionHandle(99));
            Ok(ConnectionHandle(2))
        });
        connector
            .expect_disconnect()
            .returning(|_| Err(DbError::Internal("disconnect failed".into())));

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(secrets),
            Arc::clone(&registry),
        );

        let result = svc.connect(&id).await;
        assert!(result.is_err());
        assert_eq!(registry.get(&id), Some(ConnectionHandle(99)));
    }

    #[tokio::test]
    async fn duplicate_connect_cleanup_handle_can_be_retried() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("key".into());
        let registry = Arc::new(ConnectionRegistry::new());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(conn.clone())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("password".into())));

        let reg_for_mock = Arc::clone(&registry);
        let mut connector = MockDbConnector::new();
        connector.expect_connect().returning(move |_, _| {
            reg_for_mock.register(id, ConnectionHandle(99));
            Ok(ConnectionHandle(2))
        });
        let disconnect_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        connector.expect_disconnect().times(2).returning({
            let disconnect_calls = Arc::clone(&disconnect_calls);
            move |handle| match disconnect_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) {
                0 => {
                    assert_eq!(*handle, ConnectionHandle(2));
                    Err(DbError::Internal("temporary disconnect failure".into()))
                }
                1 => {
                    assert_eq!(*handle, ConnectionHandle(2));
                    Ok(())
                }
                _ => unreachable!("disconnect called more than expected"),
            }
        });

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(secrets),
            Arc::clone(&registry),
        );

        let error = svc
            .connect(&id)
            .await
            .expect_err("duplicate cleanup should report failure");
        assert!(matches!(error, DbError::Internal(message) if message == "temporary disconnect failure"));

        // The active registry entry is intentionally absent in this isolated mock;
        // disconnect still retries the orphaned duplicate handle.
        registry.unregister(&id);
        svc.disconnect(&id).await.expect("orphaned handle should be retried");
    }

    #[tokio::test]
    async fn connect_not_found() {
        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(|_| Ok(None));

        let svc = build_service(MockDbConnector::new(), repo, MockSecretStore::new());
        let result = svc.connect(&ConnectionId::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn disconnect_success() {
        let id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_disconnect().returning(|_| Ok(()));

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(MockConnectionRepository::new()),
            Box::new(MockSecretStore::new()),
            Arc::clone(&registry),
        );

        svc.disconnect(&id).await.unwrap();
        assert!(!registry.is_active(&id));
    }

    #[tokio::test]
    async fn disconnect_failure_keeps_handle_for_retry() {
        let id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector
            .expect_disconnect()
            .returning(|_| Err(DbError::ConnectionLost("close failed".into())));

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(MockConnectionRepository::new()),
            Box::new(MockSecretStore::new()),
            Arc::clone(&registry),
        );

        let error = svc
            .disconnect(&id)
            .await
            .expect_err("connector failure must be returned");
        assert!(matches!(error, DbError::ConnectionLost(_)));
        assert_eq!(registry.get(&id), Some(ConnectionHandle(1)));
    }

    #[tokio::test]
    async fn disconnect_not_active() {
        let svc = build_service(
            MockDbConnector::new(),
            MockConnectionRepository::new(),
            MockSecretStore::new(),
        );
        let result = svc.disconnect(&ConnectionId::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connectivity_success() {
        let mut connector = MockDbConnector::new();
        connector.expect_test_connection().returning(|_, _| Ok(()));

        let svc = build_service(connector, MockConnectionRepository::new(), MockSecretStore::new());
        let result = svc.test_connectivity(&test_config(), "pass").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connectivity_with_secret_uses_saved_password_when_empty() {
        let id = ConnectionId::new();
        let key = ConnectionService::secret_key(&id);

        let mut connector = MockDbConnector::new();
        connector.expect_test_connection().returning(|_, password| {
            assert_eq!(password, "saved-pass");
            Ok(())
        });

        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().returning({
            let key = key.clone();
            move |k| {
                assert_eq!(k, &key);
                Ok(Some("saved-pass".into()))
            }
        });

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(|_| Ok(None));

        let svc = build_service(connector, repo, secrets);
        let result = svc.test_connectivity_with_secret(&id, &test_config(), "").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connectivity_with_secret_uses_persisted_custom_secret_ref() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("custom/key".into());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(conn.clone())));

        let mut connector = MockDbConnector::new();
        connector.expect_test_connection().returning(|_, password| {
            assert_eq!(password, "saved-pass");
            Ok(())
        });

        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().returning(|key| {
            assert_eq!(key, "custom/key");
            Ok(Some("saved-pass".into()))
        });

        let svc = build_service(connector, repo, secrets);
        svc.test_connectivity_with_secret(&id, &test_config(), "")
            .await
            .expect("custom persisted secret should be used");
    }

    #[tokio::test]
    async fn test_connectivity_with_secret_uses_provided_password_when_present() {
        let id = ConnectionId::new();

        let mut connector = MockDbConnector::new();
        connector.expect_test_connection().returning(|_, password| {
            assert_eq!(password, "typed-pass");
            Ok(())
        });

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("saved-pass".into())));

        let svc = build_service(connector, MockConnectionRepository::new(), secrets);
        let result = svc
            .test_connectivity_with_secret(&id, &test_config(), "typed-pass")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_active_disconnects_first() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("key".into());

        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));

        let mut connector = MockDbConnector::new();
        connector.expect_disconnect().returning(|_| Ok(()));

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning({
            let conn = conn.clone();
            move |_| Ok(Some(conn.clone()))
        });
        repo.expect_delete().returning(|_| Ok(()));

        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().returning(|_| Ok(Some("pass".into())));
        secrets.expect_delete_secret().returning(|_| Ok(()));

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(secrets),
            Arc::clone(&registry),
        );

        svc.delete(&id).await.unwrap();
        assert!(!registry.is_active(&id));
    }

    #[tokio::test]
    async fn delete_repo_failure_restores_secret() {
        let id = ConnectionId::new();
        let conn = Connection::new(test_config()).with_secret_ref("key".into());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(conn.clone())));
        repo.expect_delete()
            .returning(|_| Err(DbError::Internal("repo unavailable".into())));

        let mut secrets = MockSecretStore::new();
        secrets
            .expect_retrieve_secret()
            .returning(|_| Ok(Some("saved-pass".into())));
        secrets.expect_delete_secret().returning(|_| Ok(()));
        secrets.expect_store_secret().returning(|key, password| {
            assert_eq!(key, "key");
            assert_eq!(password, "saved-pass");
            Ok(())
        });

        let svc = build_service(MockDbConnector::new(), repo, secrets);
        let error = svc.delete(&id).await.expect_err("repo failure should propagate");
        assert!(matches!(error, DbError::Internal(message) if message == "repo unavailable"));
    }

    #[tokio::test]
    async fn create_cleans_up_secret_on_repo_failure() {
        let config = test_config();

        let mut repo = MockConnectionRepository::new();
        repo.expect_save()
            .returning(|_| Err(DbError::Internal("db error".into())));

        let mut secrets = MockSecretStore::new();
        secrets.expect_store_secret().returning(|_, _| Ok(()));
        secrets.expect_delete_secret().returning(|_| Ok(()));

        let svc = build_service(MockDbConnector::new(), repo, secrets);
        let result = svc.create(config, "pass").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_password_assigns_secret_ref_for_legacy_connection() {
        let id = ConnectionId::new();
        let legacy_connection = Connection::new(test_config());
        let saved = Arc::new(std::sync::Mutex::new(None));

        let mut repo = MockConnectionRepository::new();
        repo.expect_get()
            .returning(move |_| Ok(Some(legacy_connection.clone())));
        repo.expect_save().returning({
            let saved = Arc::clone(&saved);
            move |connection| {
                *saved.lock().expect("saved connection lock") = Some(connection.clone());
                Ok(())
            }
        });

        let key = ConnectionService::secret_key(&id);
        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().returning({
            let key = key.clone();
            move |received| {
                assert_eq!(received, &key);
                Ok(None)
            }
        });
        secrets.expect_store_secret().returning(move |received, value| {
            assert_eq!(received, &ConnectionService::secret_key(&id));
            assert_eq!(value, "new-password");
            Ok(())
        });

        let svc = build_service(MockDbConnector::new(), repo, secrets);
        svc.update(&id, test_config(), Some("new-password")).await.unwrap();

        let saved = saved.lock().expect("saved connection lock");
        assert_eq!(
            saved.as_ref().and_then(|connection| connection.secret_ref.as_deref()),
            Some(key.as_str())
        );
    }

    #[tokio::test]
    async fn update_repo_failure_restores_missing_secret() {
        let id = ConnectionId::new();
        let connection = Connection::new(test_config());

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(connection.clone())));
        repo.expect_save()
            .returning(|_| Err(DbError::Internal("save failed".into())));

        let mut secrets = MockSecretStore::new();
        secrets.expect_retrieve_secret().returning(|_| Ok(None));
        secrets.expect_store_secret().returning(|_, _| Ok(()));
        secrets.expect_delete_secret().returning(move |key| {
            assert_eq!(key, &ConnectionService::secret_key(&id));
            Ok(())
        });

        let svc = build_service(MockDbConnector::new(), repo, secrets);
        assert!(svc.update(&id, test_config(), Some("new-password")).await.is_err());
    }

    #[tokio::test]
    async fn update_repo_failure_keeps_active_connection() {
        let id = ConnectionId::new();
        let connection = Connection::new(test_config());
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(connection.clone())));
        repo.expect_save()
            .returning(|_| Err(DbError::Internal("save failed".into())));

        let svc = ConnectionService::new(
            Box::new(MockDbConnector::new()),
            Box::new(repo),
            Box::new(MockSecretStore::new()),
            Arc::clone(&registry),
        );

        assert!(svc.update(&id, test_config(), None).await.is_err());
        assert_eq!(registry.get(&id), Some(ConnectionHandle(1)));
    }

    #[tokio::test]
    async fn update_disconnect_failure_rolls_back_persisted_connection() {
        let id = ConnectionId::new();
        let connection = Connection::new(test_config());
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, ConnectionHandle(1));
        let save_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut repo = MockConnectionRepository::new();
        repo.expect_get().returning(move |_| Ok(Some(connection.clone())));
        repo.expect_save().times(2).returning({
            let save_count = Arc::clone(&save_count);
            move |saved| {
                let call = save_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if call == 0 {
                    assert_eq!(saved.config.name, "updated");
                } else {
                    assert_eq!(saved.config.name, "test");
                }
                Ok(())
            }
        });

        let mut connector = MockDbConnector::new();
        connector
            .expect_disconnect()
            .returning(|_| Err(DbError::ConnectionLost("close failed".into())));

        let svc = ConnectionService::new(
            Box::new(connector),
            Box::new(repo),
            Box::new(MockSecretStore::new()),
            Arc::clone(&registry),
        );

        let mut updated_config = test_config();
        updated_config.name = "updated".into();
        let error = svc
            .update(&id, updated_config, None)
            .await
            .expect_err("disconnect must fail");
        assert!(matches!(error, DbError::ConnectionLost(_)));
        assert_eq!(save_count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(registry.get(&id), Some(ConnectionHandle(1)));
    }
}
