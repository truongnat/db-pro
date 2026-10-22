use super::*;

#[tokio::test]
async fn list_connections() {
    let mut repo = MockConnectionRepository::new();
    repo.expect_list().returning(|| Ok(vec![]));

    let svc = build_service(MockDbConnector::new(), repo, MockSecretStore::new());
    let result = svc.list().await;
    assert!(result.is_ok());
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
