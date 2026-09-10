use std::sync::Arc;

use db_pro_core::application::{ConnectionService, QueryService, SchemaService};
use db_pro_core::domain::connection::{ConnectionId, DriverType};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{QueryParam, QueryResult};
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
