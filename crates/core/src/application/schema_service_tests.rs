use super::*;
use crate::domain::connection::ConnectionHandle;
use crate::domain::schema::*;
use crate::ports::MockIntrospectionCache;
use crate::ports::{MockConnectionRepository, MockDbConnector};

fn connections_for(driver: DriverType) -> MockConnectionRepository {
    let mut repo = MockConnectionRepository::new();
    repo.expect_get_config().returning(move |_id| {
        Ok(Some(crate::domain::connection::ConnectionConfig {
            name: "test".into(),
            host: "localhost".into(),
            port: 5432,
            database: "testdb".into(),
            username: "user".into(),
            driver,
            ssl_mode: crate::domain::connection::SslMode::Disable,
            ssh_tunnel: None,
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
            readonly: false,
        }))
    });
    repo
}

fn mock_connections() -> MockConnectionRepository {
    connections_for(DriverType::Postgres)
}

fn sqlite_connections() -> MockConnectionRepository {
    connections_for(DriverType::SQLite)
}

fn test_introspect_result() -> IntrospectResult {
    IntrospectResult {
        schemas: vec![Schema { name: "public".into() }],
        tables: vec![Table {
            name: "users".into(),
            schema: "public".into(),
            row_count: Some(100),
        }],
        columns: vec![
            Column {
                name: "id".into(),
                data_type: "INTEGER".into(),
                nullable: false,
                default: None,
                is_primary_key: true,
                table_name: "users".into(),
                schema: "public".into(),
                ..Default::default()
            },
            Column {
                name: "email".into(),
                data_type: "TEXT".into(),
                nullable: false,
                default: None,
                is_primary_key: false,
                table_name: "users".into(),
                schema: "public".into(),
                ..Default::default()
            },
        ],
        primary_keys: vec![PrimaryKey {
            constraint_name: "users_pk".into(),
            columns: vec!["id".into()],
            table_name: "users".into(),
            schema: "public".into(),
        }],
        indexes: vec![Index {
            name: "idx_email".into(),
            columns: vec!["email".into()],
            unique: true,
            origin: IndexOrigin::User,
            table_name: "users".into(),
            schema: "public".into(),
            ..Default::default()
        }],
        foreign_keys: vec![],
        views: vec![View {
            name: "active_users".into(),
            schema: "public".into(),
            definition: "SELECT id, email FROM users WHERE active = true".into(),
        }],
        triggers: vec![Trigger {
            name: "audit_insert".into(),
            table_name: "users".into(),
            schema: "public".into(),
            timing: "AFTER".into(),
            event: "INSERT".into(),
            definition: "EXECUTE FUNCTION audit_fn()".into(),
            function_def: String::new(),
            enabled: true,
        }],
        check_constraints: vec![],
        functions: vec![],
    }
}

#[tokio::test]
async fn introspect_cache_miss() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(None));
    cache.expect_save().returning(|_, _| Ok(()));

    let mut connector = MockDbConnector::new();
    connector
        .expect_introspect()
        .returning(|_| Ok(test_introspect_result()));

    let svc = SchemaService::new(
        Box::new(connector),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let result = svc.introspect(&conn_id, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn introspect_cache_hit() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

    let connector = MockDbConnector::new();

    let svc = SchemaService::new(
        Box::new(connector),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let result = svc.introspect(&conn_id, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn introspect_discards_sqlite_cache_when_database_file_is_empty() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let empty_db = std::env::temp_dir().join(format!("db-pro-empty-schema-cache-{}.sqlite", std::process::id()));
    std::fs::write(&empty_db, []).expect("create empty sqlite file");

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));
    cache.expect_invalidate().returning(|_| Ok(()));
    cache.expect_save().returning(|_, _| Ok(()));

    let mut connector = MockDbConnector::new();
    connector.expect_introspect().returning(|_| {
        Ok(IntrospectResult {
            schemas: vec![Schema { name: "main".into() }],
            tables: vec![],
            columns: vec![],
            primary_keys: vec![],
            indexes: vec![],
            foreign_keys: vec![],
            views: vec![],
            triggers: vec![],
            check_constraints: vec![],
            functions: vec![],
        })
    });

    let path = empty_db.to_string_lossy().into_owned();
    let mut repo = MockConnectionRepository::new();
    repo.expect_get_config().returning(move |_id| {
        Ok(Some(crate::domain::connection::ConnectionConfig {
            name: "empty-sqlite".into(),
            host: "localhost".into(),
            port: 0,
            database: path.clone(),
            username: String::new(),
            driver: DriverType::SQLite,
            ssl_mode: crate::domain::connection::SslMode::Disable,
            ssh_tunnel: None,
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
            readonly: false,
        }))
    });

    let svc = SchemaService::new(
        Box::new(connector),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(repo),
    );

    let result = svc.introspect(&conn_id, false).await.expect("refresh");
    assert!(result.tables.is_empty(), "live empty DB must replace stale cache");
    let _ = std::fs::remove_file(&empty_db);
}

#[test]
fn empty_sqlite_file_does_not_match_nonempty_cache() {
    let empty_db = std::env::temp_dir().join(format!("db-pro-fingerprint-{}.sqlite", std::process::id()));
    std::fs::write(&empty_db, []).unwrap();
    let cached = test_introspect_result();
    assert!(!sqlite_file_matches_cached_schema(empty_db.to_str().unwrap(), &cached));
    let _ = std::fs::remove_file(&empty_db);
}

#[tokio::test]
async fn introspect_invalid_cache_is_discarded_and_rebuilt() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache
        .expect_get()
        .returning(|_| Err(DbError::Internal("invalid cached schema".into())));
    cache.expect_invalidate().returning(|_| Ok(()));
    cache.expect_save().returning(|_, _| Ok(()));

    let mut connector = MockDbConnector::new();
    connector
        .expect_introspect()
        .returning(|_| Ok(test_introspect_result()));

    let svc = SchemaService::new(
        Box::new(connector),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let result = svc.introspect(&conn_id, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn introspect_force_refresh() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));
    cache.expect_save().returning(|_, _| Ok(()));

    let mut connector = MockDbConnector::new();
    connector
        .expect_introspect()
        .returning(|_| Ok(test_introspect_result()));

    let svc = SchemaService::new(
        Box::new(connector),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let result = svc.introspect(&conn_id, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_table_info_found() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

    let svc = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let info = svc.get_table_info(&conn_id, "public", "users").await;
    assert!(info.is_ok());
    let info = info.unwrap();
    assert_eq!(info.columns.len(), 2);
    assert!(info.primary_key.is_some());
    assert_eq!(info.indexes.len(), 1);
}

#[tokio::test]
async fn get_table_info_not_found() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

    let svc = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let result = svc.get_table_info(&conn_id, "public", "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_table_ddl_view() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

    let svc = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let ddl = svc.get_table_ddl(&conn_id, "public", "active_users").await.unwrap();
    assert!(ddl.contains("SELECT id, email FROM users WHERE active = true"));
}

#[tokio::test]
async fn get_table_ddl_basic() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

    let svc = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let ddl = svc.get_table_ddl(&conn_id, "public", "users").await.unwrap();
    assert!(ddl.contains("CREATE TABLE \"public\".\"users\""));
    assert!(ddl.contains("\"id\" INTEGER NOT NULL"));
    assert!(ddl.contains("PRIMARY KEY (\"id\")"));
    assert!(ddl.contains("CREATE UNIQUE INDEX \"public\".\"idx_email\" ON \"public\".\"users\""));
    assert!(ddl.contains("CREATE TRIGGER"));
    assert!(ddl.contains("audit_insert"));
    assert!(ddl.contains("AFTER INSERT"));
}

#[tokio::test]
async fn get_table_ddl_sqlite_uses_inline_foreign_keys_and_unqualified_names() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut introspection = test_introspect_result();
    introspection.schemas = vec![Schema { name: "main".into() }];
    introspection.tables[0].schema = "main".into();
    for column in &mut introspection.columns {
        column.schema = "main".into();
    }
    introspection.primary_keys[0].schema = "main".into();
    introspection.indexes[0].schema = "main".into();
    introspection.indexes.push(Index {
        name: "sqlite_autoindex_users_1".into(),
        columns: vec!["email".into()],
        unique: true,
        origin: IndexOrigin::UniqueConstraint,
        table_name: "users".into(),
        schema: "main".into(),
        ..Default::default()
    });
    introspection.check_constraints = vec![CheckConstraint {
        name: "users_check_0".into(),
        table_name: "users".into(),
        schema: "main".into(),
        definition: "id > 0".into(),
    }];
    introspection.foreign_keys = vec![ForeignKey {
        name: "users_fk_0".into(),
        from_table: "users".into(),
        from_columns: vec!["id".into()],
        to_table: "parents".into(),
        to_columns: vec!["id".into()],
        schema: "main".into(),
        to_schema: "main".into(),
        ..Default::default()
    }];

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(move |_| Ok(Some(introspection.clone())));

    let service = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(sqlite_connections()),
    );

    let ddl = service.get_table_ddl(&conn_id, "main", "users").await.unwrap();

    assert!(ddl.contains("CREATE TABLE \"users\""));
    assert!(ddl.contains("CONSTRAINT \"users_fk_0\" FOREIGN KEY (\"id\") REFERENCES \"parents\" (\"id\")"));
    assert!(ddl.contains("CONSTRAINT \"users_check_0\" CHECK (id > 0)"));
    assert!(ddl.contains("CREATE UNIQUE INDEX \"idx_email\" ON \"users\""));
    assert!(ddl.contains("UNIQUE (\"email\")"));
    assert!(!ddl.contains("sqlite_autoindex_users_1"));
    assert!(!ddl.contains("ADD CONSTRAINT"));
    assert!(!ddl.contains("\"main\"."));
}

#[test]
fn empty_schema_ddl_generation_omits_prefix() {
    let info = TableInfo {
        table: Table {
            name: "users".into(),
            schema: "".into(),
            row_count: None,
        },
        columns: vec![Column {
            name: "id".into(),
            data_type: "INTEGER".into(),
            nullable: false,
            default: None,
            is_primary_key: true,
            table_name: "users".into(),
            schema: "".into(),
            ..Default::default()
        }],
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![ForeignKey {
            name: "fk_parent".into(),
            from_table: "users".into(),
            from_columns: vec!["parent_id".into()],
            to_table: "parents".into(),
            to_columns: vec!["id".into()],
            schema: "".into(),
            to_schema: "".into(),
            ..Default::default()
        }],
        check_constraints: vec![],
        dependencies: vec![],
    };

    let ddl = build_create_table_ddl(&info, DriverType::Postgres, &[]);
    assert!(ddl.contains("CREATE TABLE \"users\""));
    assert!(ddl.contains("REFERENCES \"parents\""));

    let trigger = Trigger {
        name: "tr_test".into(),
        table_name: "users".into(),
        schema: "".into(),
        timing: "AFTER".into(),
        event: "INSERT".into(),
        definition: "EXECUTE FUNCTION test()".into(),
        function_def: String::new(),
        enabled: true,
    };
    let trigger_ddl = format_trigger_ddl(&trigger);
    assert!(trigger_ddl.contains("AFTER INSERT ON \"users\""));
}

#[test]
fn ddl_generation_with_fk() {
    let info = TableInfo {
        table: Table {
            name: "orders".into(),
            schema: "public".into(),
            row_count: None,
        },
        columns: vec![Column {
            name: "user_id".into(),
            data_type: "INTEGER".into(),
            nullable: false,
            default: None,
            is_primary_key: false,
            table_name: "orders".into(),
            schema: "public".into(),
            ..Default::default()
        }],
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![ForeignKey {
            name: "fk_user".into(),
            from_table: "orders".into(),
            from_columns: vec!["user_id".into()],
            to_table: "users".into(),
            to_columns: vec!["id".into()],
            schema: "public".into(),
            to_schema: "public".into(),
            ..Default::default()
        }],
        check_constraints: vec![],
        dependencies: vec![],
    };

    let ddl = build_create_table_ddl(&info, DriverType::Postgres, &[]);
    assert!(ddl.contains("FOREIGN KEY (\"user_id\") REFERENCES \"public\".\"users\" (\"id\")"));
}

#[test]
fn format_trigger_ddl_reconstructs_from_parts() {
    let trigger = Trigger {
        name: "tr_audit".into(),
        table_name: "users".into(),
        schema: "public".into(),
        timing: "AFTER".into(),
        event: "INSERT".into(),
        definition: "EXECUTE FUNCTION audit_fn()".into(),
        function_def: String::new(),
        enabled: true,
    };
    let ddl = format_trigger_ddl(&trigger);
    assert!(ddl.contains("CREATE TRIGGER"));
    assert!(ddl.contains("\"tr_audit\""));
    assert!(ddl.contains("AFTER INSERT ON"));
    assert!(ddl.contains("\"public\".\"users\""));
    assert!(ddl.contains("EXECUTE FUNCTION audit_fn()"));
}

#[test]
fn format_trigger_ddl_includes_function_def_when_present() {
    let func_body = "CREATE FUNCTION audit_fn() RETURNS trigger AS $$ BEGIN RETURN NEW; END; $$ LANGUAGE plpgsql";
    let trigger = Trigger {
        name: "tr_audit".into(),
        table_name: "users".into(),
        schema: "public".into(),
        timing: "AFTER".into(),
        event: "INSERT".into(),
        definition: "EXECUTE FUNCTION audit_fn()".into(),
        function_def: func_body.into(),
        enabled: true,
    };
    let ddl = format_trigger_ddl(&trigger);
    assert!(ddl.contains("CREATE TRIGGER"));
    assert!(ddl.contains("EXECUTE FUNCTION audit_fn()"));
    assert!(ddl.contains("CREATE FUNCTION audit_fn()"));
    assert!(ddl.contains("RETURN NEW"));
    // Function definition must come BEFORE the CREATE TRIGGER statement.
    let func_pos = ddl.find("CREATE FUNCTION").unwrap();
    let trigger_pos = ddl.find("CREATE TRIGGER").unwrap();
    assert!(func_pos < trigger_pos, "function_def must preced CREATE TRIGGER");
}

#[test]
fn format_trigger_ddl_uses_full_definition_when_available() {
    let full_sql = "CREATE TRIGGER tr_audit AFTER INSERT ON users BEGIN SELECT 1; END";
    let trigger = Trigger {
        name: "tr_audit".into(),
        table_name: "users".into(),
        schema: "main".into(),
        timing: "AFTER".into(),
        event: "INSERT".into(),
        definition: full_sql.into(),
        function_def: String::new(),
        enabled: true,
    };
    let ddl = format_trigger_ddl(&trigger);
    assert!(ddl.starts_with(full_sql));
    assert!(ddl.ends_with(";\n"));
}

#[test]
fn ddl_generation_groups_composite_fk_rows_into_one_constraint() {
    let info = TableInfo {
        table: Table {
            name: "child".into(),
            schema: "public".into(),
            row_count: None,
        },
        columns: vec![
            Column {
                name: "tenant_id".into(),
                data_type: "INTEGER".into(),
                nullable: false,
                default: None,
                is_primary_key: false,
                table_name: "child".into(),
                schema: "public".into(),
                ..Default::default()
            },
            Column {
                name: "parent_id".into(),
                data_type: "INTEGER".into(),
                nullable: false,
                default: None,
                is_primary_key: false,
                table_name: "child".into(),
                schema: "public".into(),
                ..Default::default()
            },
        ],
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![ForeignKey {
            name: "fk_parent".into(),
            from_table: "child".into(),
            from_columns: vec!["tenant_id".into(), "parent_id".into()],
            to_table: "parent".into(),
            to_columns: vec!["tenant_id".into(), "id".into()],
            schema: "public".into(),
            to_schema: "public".into(),
            ..Default::default()
        }],
        check_constraints: vec![],
        dependencies: vec![],
    };

    let ddl = build_create_table_ddl(&info, DriverType::Postgres, &[]);
    assert_eq!(ddl.matches("ADD CONSTRAINT \"fk_parent\"").count(), 1);
    assert!(ddl.contains(
        "FOREIGN KEY (\"tenant_id\", \"parent_id\") REFERENCES \"public\".\"parent\" (\"tenant_id\", \"id\")"
    ));
}

#[test]
fn quote_identifier_handles_special_chars() {
    assert_eq!(quote_identifier("table"), "\"table\"");
    assert_eq!(quote_identifier("my table"), "\"my table\"");
    assert_eq!(quote_identifier("select"), "\"select\"");
    assert_eq!(quote_identifier("has\"quote"), "\"has\"\"quote\"");
}

#[tokio::test]
async fn test_dependency_deduplication() {
    let conn_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(conn_id, ConnectionHandle(1));

    let mut introspection = test_introspect_result();
    // Add two foreign keys referencing the same parent table
    introspection.foreign_keys.push(ForeignKey {
        name: "fk_users_parent1".into(),
        from_table: "users".into(),
        from_columns: vec!["parent_id".into()],
        to_table: "parents".into(),
        to_columns: vec!["id".into()],
        schema: "public".into(),
        to_schema: "public".into(),
        ..Default::default()
    });
    introspection.foreign_keys.push(ForeignKey {
        name: "fk_users_parent2".into(),
        from_table: "users".into(),
        from_columns: vec!["alt_parent_id".into()],
        to_table: "parents".into(),
        to_columns: vec!["id".into()],
        schema: "public".into(),
        to_schema: "public".into(),
        ..Default::default()
    });

    let mut cache = MockIntrospectionCache::new();
    cache.expect_get().returning(move |_| Ok(Some(introspection.clone())));

    let svc = SchemaService::new(
        Box::new(MockDbConnector::new()),
        Box::new(cache),
        Arc::clone(&registry),
        Box::new(mock_connections()),
    );

    let info = svc.get_table_info(&conn_id, "public", "users").await.unwrap();
    let parent_table_deps: Vec<_> = info
        .dependencies
        .iter()
        .filter(|d| d.name == "parents" && d.kind == DependencyKind::Table)
        .collect();
    assert_eq!(
        parent_table_deps.len(),
        1,
        "Duplicate table dependency should be deduplicated"
    );
}
