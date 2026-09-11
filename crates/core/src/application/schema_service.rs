use std::sync::Arc;

use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::domain::schema::{CheckConstraint, ForeignKey, IntrospectResult, TableInfo, Trigger};
use crate::ports::{ConnectionRepository, DbConnector, IntrospectionCache};

use super::registry::ConnectionRegistry;
use super::sql_policy::reject_multi_statement;

pub struct SchemaService {
    connector: Box<dyn DbConnector>,
    cache: Box<dyn IntrospectionCache>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl SchemaService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        cache: Box<dyn IntrospectionCache>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            cache,
            registry,
            connections,
        }
    }

    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self.connection_config(connection_id).await?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    async fn connection_config(&self, connection_id: &ConnectionId) -> Result<ConnectionConfig, DbError> {
        self.connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))
    }

    pub async fn introspect(
        &self,
        connection_id: &ConnectionId,
        force_refresh: bool,
    ) -> Result<IntrospectResult, DbError> {
        if !force_refresh {
            match self.cache.get(connection_id).await {
                Ok(Some(cached)) => return Ok(cached),
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(
                        connection_id = %connection_id,
                        error = %error,
                        "discarding invalid introspection cache"
                    );
                    if let Err(invalidate_error) = self.cache.invalidate(connection_id).await {
                        tracing::warn!(
                            connection_id = %connection_id,
                            error = %invalidate_error,
                            "failed to discard invalid introspection cache"
                        );
                    }
                }
            }
        }

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let result = self.connector.introspect(&handle).await?;

        if let Err(e) = self.cache.save(connection_id, &result).await {
            tracing::warn!("failed to cache introspection: {e}");
        }

        Ok(result)
    }

    pub async fn get_table_info(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<TableInfo, DbError> {
        let introspect = self.introspect(connection_id, false).await?;

        let tbl = introspect
            .tables
            .iter()
            .find(|t| t.schema == schema && t.name == table)
            .ok_or_else(|| DbError::NotFound(format!("table {schema}.{table}")))?
            .clone();

        let columns: Vec<_> = introspect
            .columns
            .into_iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .collect();

        let primary_key = introspect
            .primary_keys
            .into_iter()
            .find(|pk| pk.schema == schema && pk.table_name == table);

        let indexes: Vec<_> = introspect
            .indexes
            .into_iter()
            .filter(|i| i.schema == schema && i.table_name == table)
            .collect();

        let foreign_keys: Vec<_> = introspect
            .foreign_keys
            .into_iter()
            .filter(|fk| fk.from_table == table && fk.schema == schema)
            .collect();

        Ok(TableInfo {
            table: tbl,
            columns,
            primary_key,
            indexes,
            foreign_keys,
        })
    }

    pub async fn get_table_ddl(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<String, DbError> {
        let introspect = self.introspect(connection_id, false).await?;

        // Check if this is a view first — views use their stored definition.
        if let Some(view) = introspect.views.iter().find(|v| v.schema == schema && v.name == table) {
            return Ok(format!("{};\n", view.definition));
        }

        let tbl = introspect
            .tables
            .iter()
            .find(|t| t.schema == schema && t.name == table)
            .ok_or_else(|| DbError::NotFound(format!("table or view {schema}.{table}")))?
            .clone();

        let check_constraints: Vec<_> = introspect
            .check_constraints
            .iter()
            .filter(|constraint| constraint.schema == schema && constraint.table_name == table)
            .cloned()
            .collect();

        let columns: Vec<_> = introspect
            .columns
            .into_iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .collect();

        let primary_key = introspect
            .primary_keys
            .into_iter()
            .find(|pk| pk.schema == schema && pk.table_name == table);

        let indexes: Vec<_> = introspect
            .indexes
            .into_iter()
            .filter(|i| i.schema == schema && i.table_name == table)
            .collect();

        let foreign_keys: Vec<_> = introspect
            .foreign_keys
            .into_iter()
            .filter(|fk| fk.from_table == table && fk.schema == schema)
            .collect();

        let triggers: Vec<_> = introspect
            .triggers
            .into_iter()
            .filter(|tr| tr.table_name == table && tr.schema == schema)
            .collect();

        let info = TableInfo {
            table: tbl,
            columns,
            primary_key,
            indexes,
            foreign_keys,
        };

        let driver = self.connection_config(connection_id).await?.driver;
        let mut ddl = build_create_table_ddl(&info, driver, &check_constraints);
        for trigger in &triggers {
            ddl.push_str(&format_trigger_ddl(trigger));
            ddl.push('\n');
        }

        Ok(ddl)
    }

    pub async fn execute_ddl(&self, connection_id: &ConnectionId, sql: &str) -> Result<u64, DbError> {
        reject_multi_statement(sql)?;

        // Enforce safety policy: readonly connections cannot execute DDL.
        let policy = self.safety_policy_for(connection_id).await?;
        validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let affected = self.connector.execute(&handle, sql, &[]).await?;

        if let Err(e) = self.cache.invalidate(connection_id).await {
            tracing::warn!("failed to invalidate cache after DDL: {e}");
        }

        Ok(affected)
    }

    pub async fn execute_ddl_batch(&self, connection_id: &ConnectionId, statements: &[String]) -> Result<u64, DbError> {
        let policy = self.safety_policy_for(connection_id).await?;
        for sql in statements {
            reject_multi_statement(sql)?;
            validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;
        }

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let affected = self.connector.execute_batch(&handle, statements).await?;

        if let Err(e) = self.cache.invalidate(connection_id).await {
            tracing::warn!("failed to invalidate cache after batch DDL: {e}");
        }

        Ok(affected)
    }

    pub async fn invalidate_cache(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        self.cache.invalidate(connection_id).await
    }
}

struct ForeignKeyDdlGroup<'a> {
    name: &'a str,
    from_columns: Vec<&'a str>,
    to_table: &'a str,
    to_columns: Vec<&'a str>,
    to_schema: &'a str,
}

fn group_foreign_keys_for_ddl(foreign_keys: &[ForeignKey]) -> Vec<ForeignKeyDdlGroup<'_>> {
    foreign_keys
        .iter()
        .map(|fk| ForeignKeyDdlGroup {
            name: &fk.name,
            from_columns: fk.from_columns.iter().map(|s| s.as_str()).collect(),
            to_table: &fk.to_table,
            to_columns: fk.to_columns.iter().map(|s| s.as_str()).collect(),
            to_schema: &fk.to_schema,
        })
        .collect()
}

fn qualify_name(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        quote_identifier(name)
    } else {
        format!("{}.{}", quote_identifier(schema), quote_identifier(name))
    }
}

fn build_create_table_ddl(info: &TableInfo, driver: DriverType, check_constraints: &[CheckConstraint]) -> String {
    let qualified = qualify_name_for_driver(driver, &info.table.schema, &info.table.name);
    let foreign_keys = group_foreign_keys_for_ddl(&info.foreign_keys);
    let definitions = table_definitions(info, driver, check_constraints, &foreign_keys);
    let mut ddl = format!("CREATE TABLE {qualified} (\n{}\n);\n", definitions.join(",\n"));

    append_index_ddl(&mut ddl, info, driver);
    if driver == DriverType::Postgres {
        append_postgres_foreign_keys(&mut ddl, &qualified, &foreign_keys);
    }

    ddl
}

fn table_definitions(
    info: &TableInfo,
    driver: DriverType,
    check_constraints: &[CheckConstraint],
    foreign_keys: &[ForeignKeyDdlGroup<'_>],
) -> Vec<String> {
    let mut definitions: Vec<_> = info.columns.iter().map(format_column_definition).collect();

    if let Some(ref pk) = info.primary_key {
        definitions.push(format!(
            "    PRIMARY KEY ({})",
            quote_columns(&pk.columns.iter().map(String::as_str).collect::<Vec<_>>())
        ));
    }
    definitions.extend(check_constraints.iter().map(format_check_constraint));

    if driver == DriverType::SQLite {
        definitions.extend(foreign_keys.iter().map(format_sqlite_foreign_key));
    }

    definitions
}

fn format_column_definition(column: &crate::domain::schema::Column) -> String {
    let mut definition = format!("    {} {}", quote_identifier(&column.name), column.data_type);
    if !column.nullable {
        definition.push_str(" NOT NULL");
    }
    if let Some(ref default) = column.default {
        definition.push_str(&format!(" DEFAULT {default}"));
    }
    definition
}

fn format_check_constraint(constraint: &CheckConstraint) -> String {
    format!(
        "    CONSTRAINT {} {}",
        quote_identifier(&constraint.name),
        check_constraint_definition(constraint),
    )
}

fn format_sqlite_foreign_key(foreign_key: &ForeignKeyDdlGroup<'_>) -> String {
    let to_qualified = qualify_name_for_driver(DriverType::SQLite, foreign_key.to_schema, foreign_key.to_table);
    let from_columns = quote_columns(&foreign_key.from_columns);
    let to_columns = quote_columns(&foreign_key.to_columns);
    format!(
        "    CONSTRAINT {} FOREIGN KEY ({from_columns}) REFERENCES {to_qualified} ({to_columns})",
        quote_identifier(foreign_key.name),
    )
}

fn append_index_ddl(ddl: &mut String, info: &TableInfo, driver: DriverType) {
    let index_target = qualify_name_for_driver(driver, &info.table.schema, &info.table.name);
    for index in &info.indexes {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let index_name = qualify_name_for_driver(driver, &index.schema, &index.name);
        let cols = quote_columns(&index.columns.iter().map(String::as_str).collect::<Vec<_>>());
        ddl.push_str(&format!(
            "CREATE {unique}INDEX {index_name} ON {index_target} ({cols});\n"
        ));
    }
}

fn append_postgres_foreign_keys(ddl: &mut String, qualified_table: &str, foreign_keys: &[ForeignKeyDdlGroup<'_>]) {
    for foreign_key in foreign_keys {
        let to_qualified = qualify_name(foreign_key.to_schema, foreign_key.to_table);
        let from_columns = quote_columns(&foreign_key.from_columns);
        let to_columns = quote_columns(&foreign_key.to_columns);
        ddl.push_str(&format!(
            "ALTER TABLE {qualified_table} ADD CONSTRAINT {} FOREIGN KEY ({from_columns}) REFERENCES {to_qualified} ({to_columns});\n",
            quote_identifier(foreign_key.name),
        ));
    }
}

fn qualify_name_for_driver(driver: DriverType, schema: &str, name: &str) -> String {
    if driver == DriverType::SQLite {
        quote_identifier(name)
    } else {
        qualify_name(schema, name)
    }
}

fn quote_columns(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ")
}

fn check_constraint_definition(constraint: &CheckConstraint) -> String {
    if constraint
        .definition
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("CHECK")
    {
        constraint.definition.clone()
    } else {
        format!("CHECK ({})", constraint.definition)
    }
}

fn format_trigger_ddl(trigger: &Trigger) -> String {
    // SQLite: definition is the full CREATE TRIGGER SQL from sqlite_master.
    // PostgreSQL: definition is action_statement (EXECUTE FUNCTION ...),
    //   so emit the function definition first, then the CREATE TRIGGER.
    if trigger.definition.to_ascii_uppercase().starts_with("CREATE TRIGGER") {
        format!("{};\n", trigger.definition)
    } else {
        let qualified = qualify_name(&trigger.schema, &trigger.table_name);
        let trigger_stmt = format!(
            "CREATE TRIGGER {}\n  {} {} ON {}\n  {};\n",
            quote_identifier(&trigger.name),
            trigger.timing,
            trigger.event,
            qualified,
            trigger.definition
        );
        if !trigger.function_def.is_empty() {
            let mut ddl = String::new();
            ddl.push_str(&trigger.function_def);
            if !trigger.function_def.ends_with('\n') {
                ddl.push('\n');
            }
            ddl.push('\n');
            ddl.push_str(&trigger_stmt);
            ddl
        } else {
            trigger_stmt
        }
    }
}

fn quote_identifier(name: &str) -> String {
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
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
                query_timeout_ms: 30_000,
                max_rows: 500,
                color: None,
                tags: vec![],
                group: None,
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
                },
                Column {
                    name: "email".into(),
                    data_type: "TEXT".into(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    table_name: "users".into(),
                    schema: "public".into(),
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
                table_name: "users".into(),
                schema: "public".into(),
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
            }],
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
            }],
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
                },
                Column {
                    name: "parent_id".into(),
                    data_type: "INTEGER".into(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    table_name: "child".into(),
                    schema: "public".into(),
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
            }],
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
}
