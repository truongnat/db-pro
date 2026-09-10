use std::sync::Arc;

use db_pro_core::application::{ConnectionService, ExportService, QueryService, SchemaService, TableDataService};
use db_pro_core::domain::connection::{ConnectionId, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam, QueryResult};
use db_pro_core::application::sql_builder::{SortClause, TableFilter};
use db_pro_core::domain::schema::IntrospectResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbErrorDto {
    pub code: String,
    pub message: String,
    pub message_id: String,
    pub retryable: bool,
}

impl From<DbError> for DbErrorDto {
    fn from(error: DbError) -> Self {
        Self {
            code: error.code().to_owned(),
            message: error.to_string(),
            message_id: error.message_id().to_owned(),
            retryable: error.retryable(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub readonly: bool,
}

#[derive(Clone)]
pub struct ConnectionApi {
    service: Arc<ConnectionService>,
}

impl ConnectionApi {
    pub(crate) fn new(service: Arc<ConnectionService>) -> Self {
        Self { service }
    }

    pub async fn list(&self) -> Result<Vec<ConnectionSummary>, DbErrorDto> {
        self.service
            .list()
            .await
            .map(|connections| {
                connections
                    .into_iter()
                    .map(|connection| ConnectionSummary {
                        id: connection.id.to_string(),
                        name: connection.config.name,
                        driver: match connection.config.driver {
                            DriverType::Postgres => "PostgreSQL".to_owned(),
                            DriverType::SQLite => "SQLite".to_owned(),
                        },
                        readonly: connection.config.readonly,
                    })
                    .collect()
            })
            .map_err(Into::into)
    }

    pub async fn connect(&self, connection_id: &str) -> Result<(), DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service.connect(&connection_id).await.map(|_| ()).map_err(Into::into)
    }

    pub async fn disconnect(&self, connection_id: &str) -> Result<(), DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service.disconnect(&connection_id).await.map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct QueryApi {
    service: Arc<QueryService>,
}

impl QueryApi {
    pub(crate) fn new(service: Arc<QueryService>) -> Self {
        Self { service }
    }

    pub async fn execute(&self, connection_id: &str, sql: &str) -> Result<QueryResult, DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service
            .execute(&connection_id, sql, &[] as &[QueryParam], None, None)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct SchemaApi {
    service: Arc<SchemaService>,
}

impl SchemaApi {
    pub(crate) fn new(service: Arc<SchemaService>) -> Self {
        Self { service }
    }

    pub async fn introspect(&self, connection_id: &str, force_refresh: bool) -> Result<IntrospectResult, DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service.introspect(&connection_id, force_refresh).await.map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct TableDataApi {
    service: Arc<TableDataService>,
}

impl TableDataApi {
    pub(crate) fn new(service: Arc<TableDataService>) -> Self {
        Self { service }
    }

    pub async fn fetch_rows(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
        sorts: &[SortClause],
        limit: u64,
        offset: u64,
    ) -> Result<(QueryResult, u64), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .fetch_rows(&connection_id, schema, table, filters, sorts, limit, offset)
            .await
            .map_err(Into::into)
    }

    pub async fn update_row(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .update_row(&connection_id, schema, table, columns, values, pk_columns, pk_values)
            .await
            .map_err(Into::into)
    }

    pub async fn delete_row(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        pk_columns: &[String],
        pk_values: &[CellValue],
    ) -> Result<u64, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .delete_row(&connection_id, schema, table, pk_columns, pk_values)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct ExportApi {
    service: Arc<ExportService>,
}

impl ExportApi {
    pub(crate) fn new(service: Arc<ExportService>) -> Self {
        Self { service }
    }

    pub async fn csv(&self, connection_id: &str, sql: &str) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_csv(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn json(&self, connection_id: &str, sql: &str) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_json(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn excel(&self, connection_id: &str, sql: &str) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_excel(&connection_id, sql).await.map_err(Into::into)
    }
}

fn parse_connection_id(value: &str) -> Result<db_pro_core::domain::connection::ConnectionId, DbErrorDto> {
    db_pro_core::domain::connection::ConnectionId::parse(value).map_err(|error| DbErrorDto {
        code: "VALIDATION_ERROR".to_owned(),
        message: format!("invalid connection id: {error}"),
        message_id: "error.validation".to_owned(),
        retryable: false,
    })
}
