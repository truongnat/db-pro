use std::sync::Arc;

use db_pro_core::application::sql_builder::{SortClause, TableFilter};
use db_pro_core::application::{
    BackupService, ConnectionRegistry, ConnectionService, DataDiffService, ExportService, QueryService, SchemaService,
    TableDataService, UserService,
};
use db_pro_core::domain::backup::{BackupOptions, BackupResult, RestoreOptions};
use db_pro_core::domain::connection::{Connection, ConnectionConfig, ConnectionId, DriverType};
use db_pro_core::domain::cross_connection::{DataDiff, SchemaDiff};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::{QueryHistory, SavedQuery, SavedQueryFolder};
use db_pro_core::domain::query::{CellValue, QueryParam, QueryResult};
use db_pro_core::domain::run_config::RunConfig;
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::domain::user::{DatabaseUser, Privilege};
use db_pro_core::ports::ConnectionRepository;
use db_pro_infrastructure::connector::CompositeConnector;
use db_pro_infrastructure::meta::store::SQLiteMetaStore;

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
pub struct SchemaSummary {
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub table_details: Vec<TableSummary>,
    pub views: Vec<ViewSummary>,
    pub triggers: Vec<TriggerSummary>,
    pub functions: Vec<FunctionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSummary {
    pub schema: String,
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<ColumnSummary>,
    pub foreign_keys: Vec<ForeignKeySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSummary {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeySummary {
    pub name: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSummary {
    pub schema: String,
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerSummary {
    pub schema: String,
    pub name: String,
    pub table_name: String,
    pub timing: String,
    pub event: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSummary {
    pub schema: String,
    pub name: String,
    pub routine_type: String,
    pub data_type: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryFolderSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedQuerySummary {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSummary {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
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
            .map(|connections| connections.into_iter().map(summary_from_connection).collect())
            .map_err(Into::into)
    }

    pub async fn list_details(&self) -> Result<Vec<Connection>, DbErrorDto> {
        self.service.list().await.map_err(Into::into)
    }

    pub async fn get(&self, connection_id: &str) -> Result<Option<Connection>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.get(&connection_id).await.map_err(Into::into)
    }

    pub async fn create(
        &self,
        config: db_pro_core::domain::connection::ConnectionConfig,
        password: &str,
    ) -> Result<ConnectionSummary, DbErrorDto> {
        self.service
            .create(config, password)
            .await
            .map(summary_from_connection)
            .map_err(Into::into)
    }

    pub async fn create_detail(&self, config: ConnectionConfig, password: &str) -> Result<Connection, DbErrorDto> {
        self.service.create(config, password).await.map_err(Into::into)
    }

    pub async fn update(
        &self,
        connection_id: &str,
        config: db_pro_core::domain::connection::ConnectionConfig,
        password: Option<&str>,
    ) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .update(&connection_id, config, password)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(&self, connection_id: &str) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.delete(&connection_id).await.map_err(Into::into)
    }

    pub async fn test(
        &self,
        config: &db_pro_core::domain::connection::ConnectionConfig,
        password: &str,
    ) -> Result<(), DbErrorDto> {
        self.service
            .test_connectivity(config, password)
            .await
            .map_err(Into::into)
    }

    pub async fn test_with_secret(
        &self,
        connection_id: &str,
        config: &ConnectionConfig,
        password: &str,
    ) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .test_connectivity_with_secret(&connection_id, config, password)
            .await
            .map_err(Into::into)
    }

    pub async fn connect(&self, connection_id: &str) -> Result<(), DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service
            .connect(&connection_id)
            .await
            .map(|_| ())
            .map_err(Into::into)
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
        self.execute_with_context(connection_id, sql, None, None).await
    }

    pub async fn execute_with_context(
        &self,
        connection_id: &str,
        sql: &str,
        database: Option<&str>,
        schema: Option<&str>,
    ) -> Result<QueryResult, DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service
            .execute(&connection_id, sql, &[] as &[QueryParam], database, schema)
            .await
            .map_err(Into::into)
    }

    pub async fn execute_multi(
        &self,
        connection_id: &str,
        sql: &str,
        database: Option<&str>,
        schema: Option<&str>,
    ) -> Result<db_pro_core::application::MultiQueryResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .execute_multi(&connection_id, sql, database, schema)
            .await
            .map_err(Into::into)
    }

    pub async fn explain(&self, connection_id: &str, sql: &str) -> Result<serde_json::Value, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.explain(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn history(&self, connection_id: &str, limit: u32) -> Result<Vec<QueryHistory>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .get_history(&connection_id, limit)
            .await
            .map_err(Into::into)
    }

    pub async fn save_query(
        &self,
        connection_id: &str,
        name: &str,
        sql: &str,
        folder: Option<&str>,
    ) -> Result<SavedQuery, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .save_query(&connection_id, name, sql, folder)
            .await
            .map_err(Into::into)
    }

    pub async fn list_saved_queries(&self, connection_id: &str) -> Result<Vec<SavedQuery>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .list_saved_queries(&connection_id)
            .await
            .map_err(Into::into)
    }

    pub async fn list_folders(&self, connection_id: &str) -> Result<Vec<SavedQueryFolder>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.list_folders(&connection_id).await.map_err(Into::into)
    }

    pub async fn create_folder(&self, connection_id: &str, name: &str) -> Result<SavedQueryFolder, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .create_folder(&connection_id, name)
            .await
            .map_err(Into::into)
    }

    pub async fn rename_saved_query(&self, id: &uuid::Uuid, name: &str) -> Result<(), DbErrorDto> {
        self.service.rename_saved_query(id, name).await.map_err(Into::into)
    }

    pub async fn delete_saved_query(&self, id: &uuid::Uuid) -> Result<(), DbErrorDto> {
        self.service.delete_saved_query(id).await.map_err(Into::into)
    }

    pub async fn delete_folder(&self, id: &uuid::Uuid) -> Result<(), DbErrorDto> {
        self.service.delete_folder(id).await.map_err(Into::into)
    }

    pub async fn save_run_config(
        &self,
        connection_id: &str,
        name: &str,
        sql: &str,
        timeout_ms: u64,
        max_rows: u64,
    ) -> Result<RunConfig, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .save_run_config(&connection_id, name, sql, timeout_ms, max_rows)
            .await
            .map_err(Into::into)
    }

    pub async fn list_run_configs(&self, connection_id: &str) -> Result<Vec<RunConfig>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.list_run_configs(&connection_id).await.map_err(Into::into)
    }

    pub async fn delete_run_config(&self, id: &uuid::Uuid) -> Result<(), DbErrorDto> {
        self.service.delete_run_config(id).await.map_err(Into::into)
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

    pub async fn introspect_summary(
        &self,
        connection_id: &str,
        force_refresh: bool,
    ) -> Result<SchemaSummary, DbErrorDto> {
        let result = self.introspect(connection_id, force_refresh).await?;
        let table_details = result
            .tables
            .iter()
            .map(|table| TableSummary {
                schema: table.schema.clone(),
                name: table.name.clone(),
                row_count: table.row_count,
                columns: result
                    .columns
                    .iter()
                    .filter(|column| column.schema == table.schema && column.table_name == table.name)
                    .map(|column| ColumnSummary {
                        name: column.name.clone(),
                        data_type: column.data_type.clone(),
                        nullable: column.nullable,
                        is_primary_key: column.is_primary_key,
                    })
                    .collect(),
                foreign_keys: result
                    .foreign_keys
                    .iter()
                    .filter(|foreign_key| foreign_key.schema == table.schema && foreign_key.from_table == table.name)
                    .map(|foreign_key| ForeignKeySummary {
                        name: foreign_key.name.clone(),
                        from_columns: foreign_key.from_columns.clone(),
                        to_schema: foreign_key.to_schema.clone(),
                        to_table: foreign_key.to_table.clone(),
                        to_columns: foreign_key.to_columns.clone(),
                    })
                    .collect(),
            })
            .collect();
        Ok(SchemaSummary {
            tables: result.tables.into_iter().map(|table| table.name).collect(),
            columns: result.columns.into_iter().map(|column| column.name).collect(),
            table_details,
            views: result
                .views
                .into_iter()
                .map(|view| ViewSummary {
                    schema: view.schema,
                    name: view.name,
                    definition: view.definition,
                })
                .collect(),
            triggers: result
                .triggers
                .into_iter()
                .map(|trigger| TriggerSummary {
                    schema: trigger.schema,
                    name: trigger.name,
                    table_name: trigger.table_name,
                    timing: trigger.timing,
                    event: trigger.event,
                    definition: trigger.definition,
                    enabled: trigger.enabled,
                })
                .collect(),
            functions: result
                .functions
                .into_iter()
                .map(|function| FunctionSummary {
                    schema: function.schema,
                    name: function.name,
                    routine_type: function.routine_type,
                    data_type: function.data_type,
                    definition: function.definition,
                })
                .collect(),
        })
    }

    pub async fn introspect(&self, connection_id: &str, force_refresh: bool) -> Result<IntrospectResult, DbErrorDto> {
        let connection_id = ConnectionId::parse(connection_id).map_err(|error| DbErrorDto {
            code: "VALIDATION_ERROR".to_owned(),
            message: format!("invalid connection id: {error}"),
            message_id: "error.validation".to_owned(),
            retryable: false,
        })?;
        self.service
            .introspect(&connection_id, force_refresh)
            .await
            .map_err(Into::into)
    }

    pub async fn diff_schemas(&self, source_id: &str, target_id: &str) -> Result<SchemaDiff, DbErrorDto> {
        let source_id = parse_connection_id(source_id)?;
        let target_id = parse_connection_id(target_id)?;
        self.service
            .diff_schemas(&source_id, &target_id)
            .await
            .map_err(Into::into)
    }

    pub async fn table_info(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<db_pro_core::domain::schema::TableInfo, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .get_table_info(&connection_id, schema, table)
            .await
            .map_err(Into::into)
    }

    pub async fn table_ddl(&self, connection_id: &str, schema: &str, table: &str) -> Result<String, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .get_table_ddl(&connection_id, schema, table)
            .await
            .map_err(Into::into)
    }

    pub async fn execute_ddl(&self, connection_id: &str, sql: &str) -> Result<u64, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.execute_ddl(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn execute_ddl_batch(&self, connection_id: &str, statements: &[String]) -> Result<u64, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .execute_ddl_batch(&connection_id, statements)
            .await
            .map_err(Into::into)
    }

    pub async fn invalidate_cache(&self, connection_id: &str) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.invalidate_cache(&connection_id).await.map_err(Into::into)
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

    #[allow(clippy::too_many_arguments)]
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

    pub async fn insert_row(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        columns: &[String],
        values: &[CellValue],
    ) -> Result<u64, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .insert_row(&connection_id, schema, table, columns, values)
            .await
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
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

    #[allow(clippy::too_many_arguments)]
    pub async fn update_text_row(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        column: &str,
        value: &str,
        pk_column: &str,
        pk_value: &str,
    ) -> Result<u64, DbErrorDto> {
        self.update_row(
            connection_id,
            schema,
            table,
            &[column.to_owned()],
            &[CellValue::Text(value.to_owned())],
            &[pk_column.to_owned()],
            &[CellValue::Text(pk_value.to_owned())],
        )
        .await
    }

    pub async fn delete_text_row(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        pk_column: &str,
        pk_value: &str,
    ) -> Result<u64, DbErrorDto> {
        self.delete_row(
            connection_id,
            schema,
            table,
            &[pk_column.to_owned()],
            &[CellValue::Text(pk_value.to_owned())],
        )
        .await
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

    pub async fn csv(
        &self,
        connection_id: &str,
        sql: &str,
    ) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_csv(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn json(
        &self,
        connection_id: &str,
        sql: &str,
    ) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_json(&connection_id, sql).await.map_err(Into::into)
    }

    pub async fn excel(
        &self,
        connection_id: &str,
        sql: &str,
    ) -> Result<db_pro_core::application::ExportResult, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.export_excel(&connection_id, sql).await.map_err(Into::into)
    }
}

fn summary_from_connection(connection: db_pro_core::domain::connection::Connection) -> ConnectionSummary {
    ConnectionSummary {
        id: connection.id.to_string(),
        name: connection.config.name,
        host: connection.config.host,
        port: connection.config.port,
        database: connection.config.database,
        username: connection.config.username,
        driver: match connection.config.driver {
            DriverType::Postgres => "PostgreSQL".to_owned(),
            DriverType::SQLite => "SQLite".to_owned(),
        },
        readonly: connection.config.readonly,
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

#[derive(Clone)]
pub struct BackupApi {
    service: Arc<BackupService>,
}

impl BackupApi {
    pub(crate) fn new(service: Arc<BackupService>) -> Self {
        Self { service }
    }

    pub async fn backup(&self, options: &BackupOptions) -> Result<BackupResult, DbErrorDto> {
        self.service.backup(options).await.map_err(Into::into)
    }

    pub async fn restore(&self, options: &RestoreOptions) -> Result<(), DbErrorDto> {
        self.service.restore(options).await.map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct UserApi {
    service: Arc<UserService>,
}

impl UserApi {
    pub(crate) fn new(service: Arc<UserService>) -> Self {
        Self { service }
    }

    pub async fn list_users(&self, connection_id: &str) -> Result<Vec<DatabaseUser>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.list_users(&connection_id).await.map_err(Into::into)
    }

    pub async fn create_role(&self, connection_id: &str, name: &str, login: bool) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .create_role(&connection_id, name, login)
            .await
            .map_err(Into::into)
    }

    pub async fn drop_role(&self, connection_id: &str, name: &str) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service.drop_role(&connection_id, name).await.map_err(Into::into)
    }

    pub async fn list_privileges(&self, connection_id: &str, role_name: &str) -> Result<Vec<Privilege>, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .list_privileges(&connection_id, role_name)
            .await
            .map_err(Into::into)
    }

    pub async fn grant_privilege(
        &self,
        connection_id: &str,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .grant_privilege(&connection_id, role_name, schema, table, privilege)
            .await
            .map_err(Into::into)
    }

    pub async fn revoke_privilege(
        &self,
        connection_id: &str,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.service
            .revoke_privilege(&connection_id, role_name, schema, table, privilege)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct DataDiffApi {
    service: Arc<DataDiffService>,
}

impl DataDiffApi {
    pub(crate) fn new(service: Arc<DataDiffService>) -> Self {
        Self { service }
    }

    pub async fn diff_table_data(
        &self,
        source_id: &str,
        target_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<DataDiff, DbErrorDto> {
        let source_id = parse_connection_id(source_id)?;
        let target_id = parse_connection_id(target_id)?;
        self.service
            .diff_table_data(&source_id, &target_id, schema, table)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct PostgresApi {
    connector: Arc<CompositeConnector>,
    registry: Arc<ConnectionRegistry>,
    meta_store: SQLiteMetaStore,
}

impl PostgresApi {
    pub(crate) fn new(
        connector: Arc<CompositeConnector>,
        registry: Arc<ConnectionRegistry>,
        meta_store: SQLiteMetaStore,
    ) -> Self {
        Self {
            connector,
            registry,
            meta_store,
        }
    }

    pub async fn test_ssh_tunnel(
        &self,
        config: &db_pro_core::domain::connection::SshTunnelConfig,
    ) -> Result<(), DbErrorDto> {
        self.connector.test_ssh_tunnel(config).await.map_err(Into::into)
    }

    pub async fn object_dependencies(
        &self,
        connection_id: &str,
        schema: &str,
        object_name: &str,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::ObjectDependency>, DbErrorDto> {
        let handle = self.postgres_handle(connection_id)?;
        self.connector
            .postgres_connector()
            .get_object_dependencies(&handle, schema, object_name)
            .await
            .map_err(Into::into)
    }

    pub async fn partitions(
        &self,
        connection_id: &str,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::PartitionInfo>, DbErrorDto> {
        let handle = self.postgres_handle(connection_id)?;
        self.connector
            .postgres_connector()
            .list_partitions(&handle)
            .await
            .map_err(Into::into)
    }

    pub async fn tablespaces(
        &self,
        connection_id: &str,
    ) -> Result<Vec<db_pro_core::domain::cross_connection::TablespaceInfo>, DbErrorDto> {
        let handle = self.postgres_handle(connection_id)?;
        self.connector
            .postgres_connector()
            .list_tablespaces(&handle)
            .await
            .map_err(Into::into)
    }

    pub async fn rename_schema_object(
        &self,
        connection_id: &str,
        object_type: &str,
        schema: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        if self
            .meta_store
            .get_config(&connection_id)
            .await
            .map_err(DbErrorDto::from)?
            .is_some_and(|config| config.readonly)
        {
            return Err(DbErrorDto {
                code: "SAFETY".to_owned(),
                message: "connection is read-only — schema rename is not allowed".to_owned(),
                message_id: "error.safety.readonly".to_owned(),
                retryable: false,
            });
        }

        let handle = self.postgres_handle_by_id(connection_id)?;
        self.connector
            .postgres_connector()
            .rename_schema_object(&handle, object_type, schema, old_name, new_name)
            .await
            .map_err(Into::into)
    }

    fn postgres_handle(
        &self,
        connection_id: &str,
    ) -> Result<db_pro_core::domain::connection::ConnectionHandle, DbErrorDto> {
        let connection_id = parse_connection_id(connection_id)?;
        self.postgres_handle_by_id(connection_id)
    }

    fn postgres_handle_by_id(
        &self,
        connection_id: ConnectionId,
    ) -> Result<db_pro_core::domain::connection::ConnectionHandle, DbErrorDto> {
        let composite_handle = self.registry.get(&connection_id).ok_or_else(|| DbErrorDto {
            code: "NOT_CONNECTED".to_owned(),
            message: "connection is not active".to_owned(),
            message_id: "error.connection.failed".to_owned(),
            retryable: false,
        })?;
        self.connector
            .inner_postgres_handle(&composite_handle)
            .map_err(Into::into)
    }
}
