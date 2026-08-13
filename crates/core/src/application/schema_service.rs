use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::domain::schema::{ForeignKey, IntrospectResult, TableInfo, Trigger};
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
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    pub async fn introspect(
        &self,
        connection_id: &ConnectionId,
        force_refresh: bool,
    ) -> Result<IntrospectResult, DbError> {
        if !force_refresh {
            if let Some(cached) = self.cache.get(connection_id).await? {
                return Ok(cached);
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

        let mut ddl = build_create_table_ddl(&info);
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

fn build_create_table_ddl(info: &TableInfo) -> String {
    let mut ddl = String::new();
    let qualified = format!(
        "{}.{}",
        quote_identifier(&info.table.schema),
        quote_identifier(&info.table.name)
    );

    ddl.push_str(&format!("CREATE TABLE {qualified} (\n"));

    for (i, col) in info.columns.iter().enumerate() {
        ddl.push_str(&format!("    {}", quote_identifier(&col.name)));
        ddl.push_str(&format!(" {}", col.data_type));
        if !col.nullable {
            ddl.push_str(" NOT NULL");
        }
        if let Some(ref default) = col.default {
            ddl.push_str(&format!(" DEFAULT {default}"));
        }
        if i < info.columns.len() - 1 || info.primary_key.is_some() {
            ddl.push(',');
        }
        ddl.push('\n');
    }

    if let Some(ref pk) = info.primary_key {
        let cols = pk
            .columns
            .iter()
            .map(|c| quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");
        ddl.push_str(&format!("    PRIMARY KEY ({cols})\n"));
    }

    ddl.push_str(");\n");

    for idx in &info.indexes {
        let unique = if idx.unique { "UNIQUE " } else { "" };
        let cols = idx
            .columns
            .iter()
            .map(|c| quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");
        ddl.push_str(&format!(
            "CREATE {unique}INDEX {} ON {qualified} ({cols});\n",
            quote_identifier(&idx.name)
        ));
    }

    for fk in group_foreign_keys_for_ddl(&info.foreign_keys) {
        let to_qualified = format!("{}.{}", quote_identifier(fk.to_schema), quote_identifier(fk.to_table));
        let from_columns = fk
            .from_columns
            .iter()
            .map(|column| quote_identifier(column))
            .collect::<Vec<_>>()
            .join(", ");
        let to_columns = fk
            .to_columns
            .iter()
            .map(|column| quote_identifier(column))
            .collect::<Vec<_>>()
            .join(", ");

        ddl.push_str(&format!(
            "ALTER TABLE {qualified} ADD CONSTRAINT {} FOREIGN KEY ({from_columns}) REFERENCES {to_qualified} ({to_columns});\n",
            quote_identifier(fk.name),
        ));
    }

    ddl
}

fn format_trigger_ddl(trigger: &Trigger) -> String {
    // SQLite: definition is the full CREATE TRIGGER SQL from sqlite_master.
    // PostgreSQL: definition is action_statement (EXECUTE FUNCTION ...),
    //   so emit the function definition first, then the CREATE TRIGGER.
    if trigger.definition.to_ascii_uppercase().starts_with("CREATE TRIGGER") {
        format!("{};\n", trigger.definition)
    } else {
        let qualified = format!(
            "{}.{}",
            quote_identifier(&trigger.schema),
            quote_identifier(&trigger.table_name)
        );
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

    fn mock_connections() -> MockConnectionRepository {
        let mut repo = MockConnectionRepository::new();
        repo.expect_get_config().returning(|_id| {
            Ok(Some(crate::domain::connection::ConnectionConfig {
                name: "test".into(),
                host: "localhost".into(),
                port: 5432,
                database: "testdb".into(),
                username: "user".into(),
                driver: crate::domain::connection::DriverType::Postgres,
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
        assert!(ddl.contains("CREATE UNIQUE INDEX \"idx_email\""));
        assert!(ddl.contains("CREATE TRIGGER"));
        assert!(ddl.contains("audit_insert"));
        assert!(ddl.contains("AFTER INSERT"));
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

        let ddl = build_create_table_ddl(&info);
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

        let ddl = build_create_table_ddl(&info);
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
