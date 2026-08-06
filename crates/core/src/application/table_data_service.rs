use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::ports::DbConnector;

use super::registry::ConnectionRegistry;
use super::sql_builder::{self, SortClause, TableFilter};

pub struct TableDataService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
}

impl TableDataService {
    pub fn new(connector: Box<dyn DbConnector>, registry: Arc<ConnectionRegistry>) -> Self {
        Self { connector, registry }
    }

    pub async fn fetch_rows(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
        sorts: &[SortClause],
        limit: u64,
        offset: u64,
    ) -> Result<(QueryResult, u64), DbError> {
        let handle = self.resolve_handle(connection_id)?;
        let style = self.connector.placeholder_style(&handle);

        let (select_sql, select_params) = sql_builder::build_select(style, schema, table, filters, sorts, limit, offset);
        let (count_sql, count_params) = sql_builder::build_count(style, schema, table, filters);

        let count_result = self.connector.query(&handle, &count_sql, &count_params).await?;
        let data_result = self.connector.query(&handle, &select_sql, &select_params).await?;

        let total_count = count_result
            .rows
            .first()
            .and_then(|row| row.0.first())
            .and_then(|cell| match cell {
                CellValue::Int64(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(0);

        let duration_ms = data_result.duration_ms;
        Ok((
            QueryResult {
                duration_ms,
                ..data_result
            },
            total_count,
        ))
    }

    pub async fn insert_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
    ) -> Result<u64, DbError> {
        let handle = self.resolve_handle(connection_id)?;
        let style = self.connector.placeholder_style(&handle);
        let (sql, params) = sql_builder::build_insert(style, schema, table, columns, values);
        self.connector.execute(&handle, &sql, &params).await
    }

    pub async fn update_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation(
                "update requires a primary key".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let style = self.connector.placeholder_style(&handle);
        let (sql, params) = sql_builder::build_update(style, schema, table, columns, values, pk_columns, pk_values);
        self.connector.execute(&handle, &sql, &params).await
    }

    pub async fn delete_row(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbError> {
        if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
            return Err(DbError::Validation(
                "delete requires a primary key".into(),
            ));
        }
        let handle = self.resolve_handle(connection_id)?;
        let style = self.connector.placeholder_style(&handle);
        let (sql, params) = sql_builder::build_delete(style, schema, table, pk_columns, pk_values);
        self.connector.execute(&handle, &sql, &params).await
    }

    fn resolve_handle(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<crate::domain::connection::ConnectionHandle, DbError> {
        self.registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::ConnectionHandle;
    use crate::domain::query::*;
    use crate::ports::MockDbConnector;

    fn setup() -> (ConnectionId, Arc<ConnectionRegistry>) {
        let conn_id = ConnectionId::new();
        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(conn_id.clone(), ConnectionHandle(1));
        (conn_id, registry)
    }

    #[tokio::test]
    async fn fetch_rows_returns_data_and_count() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector.expect_placeholder_style().returning(|_| PlaceholderStyle::Question);
        connector
            .expect_query()
            .times(2)
            .returning(|_handle, sql, _params| {
                if sql.contains("COUNT(*)") {
                    Ok(QueryResult {
                        columns: vec![ColumnMeta { name: "count".into(), data_type: "INT".into(), nullable: false }],
                        rows: vec![Row(vec![CellValue::Int64(42)])],
                        row_count: 1,
                        duration_ms: 1,
                    })
                } else {
                    Ok(QueryResult {
                        columns: vec![ColumnMeta { name: "id".into(), data_type: "INT".into(), nullable: false }],
                        rows: vec![Row(vec![CellValue::Int64(1)])],
                        row_count: 1,
                        duration_ms: 2,
                    })
                }
            });

        let svc = TableDataService::new(Box::new(connector), registry);
        let (result, total) = svc.fetch_rows(&conn_id, "public", "users", &[], &[], 50, 0).await.unwrap();
        assert_eq!(total, 42);
        assert_eq!(result.rows.len(), 1);
    }

    #[tokio::test]
    async fn insert_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector.expect_placeholder_style().returning(|_| PlaceholderStyle::Question);
        connector.expect_execute().returning(|_, sql, _| {
            assert!(sql.contains("INSERT INTO"));
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry);
        let affected = svc
            .insert_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("alice".into())],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn update_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector.expect_placeholder_style().returning(|_| PlaceholderStyle::Question);
        connector.expect_execute().returning(|_, sql, params| {
            assert!(sql.contains("UPDATE"));
            assert!(sql.contains("WHERE"));
            assert_eq!(params.len(), 2);
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry);
        let affected = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("bob".into())],
                &["id".into()],
                &[CellValue::Int64(1)],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn delete_row_returns_affected() {
        let (conn_id, registry) = setup();

        let mut connector = MockDbConnector::new();
        connector.expect_placeholder_style().returning(|_| PlaceholderStyle::Question);
        connector.expect_execute().returning(|_, sql, params| {
            assert!(sql.contains("DELETE FROM"));
            assert_eq!(params.len(), 1);
            Ok(1)
        });

        let svc = TableDataService::new(Box::new(connector), registry);
        let affected = svc
            .delete_row(&conn_id, "public", "users", &["id".into()], &[CellValue::Int64(1)])
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn fetch_rows_connection_not_active() {
        let registry = Arc::new(ConnectionRegistry::new());
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry);
        let fake_id = ConnectionId::new();
        let result = svc.fetch_rows(&fake_id, "public", "users", &[], &[], 50, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_row_rejects_empty_pk() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry);
        let result = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("x".into())],
                &[],
                &[],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_row_rejects_empty_pk() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry);
        let result = svc
            .delete_row(&conn_id, "public", "users", &[], &[])
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn update_row_rejects_pk_length_mismatch() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry);
        let result = svc
            .update_row(
                &conn_id,
                "public",
                "users",
                &["name".into()],
                &[CellValue::Text("x".into())],
                &["id".into(), "org_id".into()],
                &[CellValue::Int64(1)],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_row_rejects_pk_length_mismatch() {
        let (conn_id, registry) = setup();
        let connector = MockDbConnector::new();
        let svc = TableDataService::new(Box::new(connector), registry);
        let result = svc
            .delete_row(
                &conn_id,
                "public",
                "users",
                &["id".into(), "org_id".into()],
                &[CellValue::Int64(1)],
            )
            .await;
        assert!(matches!(result, Err(DbError::Validation(_))));
    }
}
