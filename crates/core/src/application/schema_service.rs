use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::schema::{IntrospectResult, TableInfo};
use crate::ports::{DbConnector, IntrospectionCache};

use super::registry::ConnectionRegistry;

pub struct SchemaService {
    connector: Box<dyn DbConnector>,
    cache: Box<dyn IntrospectionCache>,
    registry: Arc<ConnectionRegistry>,
}

impl SchemaService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        cache: Box<dyn IntrospectionCache>,
        registry: Arc<ConnectionRegistry>,
    ) -> Self {
        Self {
            connector,
            cache,
            registry,
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
        let info = self.get_table_info(connection_id, schema, table).await?;
        Ok(build_create_table_ddl(&info))
    }

    pub async fn invalidate_cache(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        self.cache.invalidate(connection_id).await
    }
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

    for fk in &info.foreign_keys {
        let to_qualified = format!("{}.{}", quote_identifier(&fk.to_schema), quote_identifier(&fk.to_table));
        ddl.push_str(&format!(
            "ALTER TABLE {qualified} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {to_qualified} ({});\n",
            quote_identifier(&fk.name),
            quote_identifier(&fk.from_column),
            quote_identifier(&fk.to_column)
        ));
    }

    ddl
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
    use crate::ports::MockDbConnector;
    use crate::ports::MockIntrospectionCache;

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
            views: vec![],
            triggers: vec![],
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

        let svc = SchemaService::new(Box::new(connector), Box::new(cache), Arc::clone(&registry));

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

        let svc = SchemaService::new(Box::new(connector), Box::new(cache), Arc::clone(&registry));

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

        let svc = SchemaService::new(Box::new(connector), Box::new(cache), Arc::clone(&registry));

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

        let svc = SchemaService::new(Box::new(MockDbConnector::new()), Box::new(cache), Arc::clone(&registry));

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

        let svc = SchemaService::new(Box::new(MockDbConnector::new()), Box::new(cache), Arc::clone(&registry));

        let result = svc.get_table_info(&conn_id, "public", "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_table_ddl_basic() {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id, ConnectionHandle(1));

        let mut cache = MockIntrospectionCache::new();
        cache.expect_get().returning(|_| Ok(Some(test_introspect_result())));

        let svc = SchemaService::new(Box::new(MockDbConnector::new()), Box::new(cache), Arc::clone(&registry));

        let ddl = svc.get_table_ddl(&conn_id, "public", "users").await.unwrap();
        assert!(ddl.contains("CREATE TABLE \"public\".\"users\""));
        assert!(ddl.contains("\"id\" INTEGER NOT NULL"));
        assert!(ddl.contains("PRIMARY KEY (\"id\")"));
        assert!(ddl.contains("CREATE UNIQUE INDEX \"idx_email\""));
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
                from_column: "user_id".into(),
                to_table: "users".into(),
                to_column: "id".into(),
                schema: "public".into(),
                to_schema: "public".into(),
            }],
        };

        let ddl = build_create_table_ddl(&info);
        assert!(ddl.contains("FOREIGN KEY (\"user_id\") REFERENCES \"public\".\"users\" (\"id\")"));
    }

    #[test]
    fn quote_identifier_handles_special_chars() {
        assert_eq!(quote_identifier("table"), "\"table\"");
        assert_eq!(quote_identifier("my table"), "\"my table\"");
        assert_eq!(quote_identifier("select"), "\"select\"");
        assert_eq!(quote_identifier("has\"quote"), "\"has\"\"quote\"");
    }
}
