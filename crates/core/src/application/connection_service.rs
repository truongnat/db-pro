use std::sync::Arc;

use crate::domain::connection::{Connection, ConnectionConfig, ConnectionHandle, ConnectionId};
use crate::domain::error::DbError;
use crate::ports::{ConnectionRepository, DbConnector, SecretStore};

use super::registry::ConnectionRegistry;

pub struct ConnectionService {
    connector: Box<dyn DbConnector>,
    repo: Box<dyn ConnectionRepository>,
    secrets: Box<dyn SecretStore>,
    registry: Arc<ConnectionRegistry>,
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

        if self.registry.is_active(id) {
            self.disconnect(id).await?;
        }

        connection.config = config;
        connection.updated_at = chrono::Utc::now();

        self.repo.save(&connection).await?;

        if let Some(pw) = password {
            let key = Self::secret_key(id);
            self.secrets.store_secret(&key, pw).await?;
        }

        Ok(())
    }

    pub async fn delete(&self, id: &ConnectionId) -> Result<(), DbError> {
        if self.registry.is_active(id) {
            self.disconnect(id).await?;
        }

        if let Some(conn) = self.repo.get(id).await? {
            if let Some(ref key) = conn.secret_ref {
                self.secrets.delete_secret(key).await?;
            }
        }

        self.repo.delete(id).await
    }

    pub async fn connect(&self, id: &ConnectionId) -> Result<ConnectionHandle, DbError> {
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
        self.registry.register(*id, handle);
        Ok(handle)
    }

    pub async fn disconnect(&self, id: &ConnectionId) -> Result<(), DbError> {
        let handle = self
            .registry
            .unregister(id)
            .ok_or_else(|| DbError::NotFound(format!("connection {id} is not active")))?;

        self.connector.disconnect(&handle).await
    }

    pub async fn test_connectivity(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        self.connector.test_connection(config, password).await
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
}
