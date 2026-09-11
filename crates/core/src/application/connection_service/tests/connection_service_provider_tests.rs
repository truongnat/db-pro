use super::*;

#[tokio::test]
async fn create_sqlite_connection_does_not_store_database_secret() {
    let mut repo = MockConnectionRepository::new();
    repo.expect_save().returning(|_| Ok(()));

    let svc = build_service(MockDbConnector::new(), repo, MockSecretStore::new());
    let connection = svc
        .create(sqlite_config(), "")
        .await
        .expect("SQLite creation must not require a database secret");

    assert!(connection.secret_ref.is_none());
}

#[tokio::test]
async fn create_postgres_rejects_missing_password() {
    let svc = build_service(
        MockDbConnector::new(),
        MockConnectionRepository::new(),
        MockSecretStore::new(),
    );

    let error = svc
        .create(test_config(), "")
        .await
        .expect_err("PostgreSQL creation must require a database password");
    assert!(matches!(error, DbError::AuthFailed(message) if message.contains("required")));
}

#[tokio::test]
async fn connect_sqlite_does_not_read_database_secret() {
    let id = ConnectionId::new();
    let connection = Connection::new(sqlite_config());

    let mut repo = MockConnectionRepository::new();
    repo.expect_get().returning(move |_| Ok(Some(connection.clone())));

    let mut connector = MockDbConnector::new();
    connector.expect_connect().returning(|config, password| {
        assert_eq!(config.driver, DriverType::SQLite);
        assert!(password.is_empty());
        Ok(ConnectionHandle(1))
    });

    let svc = build_service(connector, repo, MockSecretStore::new());
    svc.connect(&id)
        .await
        .expect("SQLite connection must not require a database secret");
}

#[tokio::test]
async fn test_connectivity_with_secret_sqlite_uses_empty_password() {
    let mut connector = MockDbConnector::new();
    connector.expect_test_connection().returning(|config, password| {
        assert_eq!(config.driver, DriverType::SQLite);
        assert!(password.is_empty());
        Ok(())
    });

    let svc = build_service(connector, MockConnectionRepository::new(), MockSecretStore::new());
    svc.test_connectivity_with_secret(&ConnectionId::new(), &sqlite_config(), "typed-pass")
        .await
        .expect("SQLite connectivity must not require a database secret");
}

#[tokio::test]
async fn update_to_sqlite_removes_database_secret_reference() {
    let id = ConnectionId::new();
    let previous = Connection::new(test_config()).with_secret_ref("db-key".into());
    let saved = Arc::new(std::sync::Mutex::new(None));

    let mut repo = MockConnectionRepository::new();
    repo.expect_get().returning(move |_| Ok(Some(previous.clone())));
    repo.expect_save().returning({
        let saved = Arc::clone(&saved);
        move |connection| {
            *saved.lock().expect("saved connection lock") = Some(connection.clone());
            Ok(())
        }
    });

    let mut secrets = MockSecretStore::new();
    secrets
        .expect_retrieve_secret()
        .withf(|key| key == "db-key")
        .returning(|_| Ok(Some("db-password".into())));
    secrets
        .expect_delete_secret()
        .withf(|key| key == "db-key")
        .returning(|_| Ok(()));

    let svc = build_service(MockDbConnector::new(), repo, secrets);
    svc.update(&id, sqlite_config(), None)
        .await
        .expect("switching to SQLite should remove the unused secret");

    let saved = saved.lock().expect("saved connection lock");
    assert!(saved.as_ref().is_some_and(|connection| connection.secret_ref.is_none()));
}

#[tokio::test]
async fn update_sqlite_to_postgres_requires_a_new_password() {
    let id = ConnectionId::new();
    let previous = Connection::new(sqlite_config());

    let mut repo = MockConnectionRepository::new();
    repo.expect_get().returning(move |_| Ok(Some(previous.clone())));

    let svc = build_service(MockDbConnector::new(), repo, MockSecretStore::new());
    let error = svc
        .update(&id, test_config(), None)
        .await
        .expect_err("switching to PostgreSQL must require a password");
    assert!(matches!(error, DbError::AuthFailed(message) if message.contains("switching")));
}
