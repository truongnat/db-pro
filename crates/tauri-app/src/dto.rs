use serde::{Deserialize, Serialize};

use db_pro_core::domain::connection::{Connection, ConnectionConfig, DriverType, SshTunnelConfig, SslMode};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::{QueryHistory, SavedQuery};
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
use db_pro_core::domain::schema::{
    Column, ForeignKey, Index, IntrospectResult, PrimaryKey, Schema, Table, TableInfo, View,
};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub error: String,
    pub message: String,
    pub message_id: String,
    pub details: Option<serde_json::Value>,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl From<DbError> for CommandError {
    fn from(err: DbError) -> Self {
        let (error, message_id) = match &err {
            DbError::ConnectionFailed(_) => ("CONNECTION_FAILED", "error.connection.failed"),
            DbError::QueryFailed(_) => ("QUERY_FAILED", "error.query.failed"),
            DbError::NotFound(_) => ("NOT_FOUND", "error.not_found"),
            DbError::AuthFailed(_) => ("AUTH_FAILED", "error.auth.failed"),
            DbError::Timeout(_) => ("TIMEOUT", "error.timeout"),
            DbError::Io(_) => ("IO_ERROR", "error.io"),
            DbError::IntrospectionFailed(_) => ("INTROSPECTION_FAILED", "error.introspection.failed"),
            DbError::EncryptionFailed(_) => ("ENCRYPTION_FAILED", "error.encryption.failed"),
            DbError::Validation(_) => ("VALIDATION", "error.validation"),
            DbError::Internal(_) => ("INTERNAL", "error.internal"),
        };
        Self {
            error: error.into(),
            message: err.to_string(),
            message_id: message_id.into(),
            details: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Connection DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDto {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: DriverTypeDto,
    pub ssl_mode: SslModeDto,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Connection> for ConnectionDto {
    fn from(c: Connection) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.config.name,
            host: c.config.host,
            port: c.config.port,
            database: c.config.database,
            username: c.config.username,
            driver: c.config.driver.into(),
            ssl_mode: c.config.ssl_mode.into(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfigDto {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: DriverTypeDto,
    pub ssl_mode: SslModeDto,
    pub ssh_tunnel: Option<SshTunnelConfigDto>,
    pub query_timeout_ms: u64,
    pub max_rows: u64,
}

impl ConnectionConfigDto {
    pub fn to_domain(&self) -> ConnectionConfig {
        ConnectionConfig {
            name: self.name.clone(),
            host: self.host.clone(),
            port: self.port,
            database: self.database.clone(),
            username: self.username.clone(),
            driver: self.driver.into(),
            ssl_mode: self.ssl_mode.into(),
            ssh_tunnel: self.ssh_tunnel.as_ref().map(|s| SshTunnelConfig {
                host: s.host.clone(),
                port: s.port,
                user: s.user.clone(),
                private_key_path: s.private_key_path.clone(),
            }),
            query_timeout_ms: self.query_timeout_ms,
            max_rows: self.max_rows,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriverTypeDto {
    Postgres,
    Sqlite,
}

impl From<DriverType> for DriverTypeDto {
    fn from(d: DriverType) -> Self {
        match d {
            DriverType::Postgres => Self::Postgres,
            DriverType::SQLite => Self::Sqlite,
        }
    }
}

impl From<DriverTypeDto> for DriverType {
    fn from(d: DriverTypeDto) -> Self {
        match d {
            DriverTypeDto::Postgres => Self::Postgres,
            DriverTypeDto::Sqlite => Self::SQLite,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SslModeDto {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

impl From<SslMode> for SslModeDto {
    fn from(s: SslMode) -> Self {
        match s {
            SslMode::Disable => Self::Disable,
            SslMode::Require => Self::Require,
            SslMode::VerifyCa => Self::VerifyCa,
            SslMode::VerifyFull => Self::VerifyFull,
        }
    }
}

impl From<SslModeDto> for SslMode {
    fn from(s: SslModeDto) -> Self {
        match s {
            SslModeDto::Disable => Self::Disable,
            SslModeDto::Require => Self::Require,
            SslModeDto::VerifyCa => Self::VerifyCa,
            SslModeDto::VerifyFull => Self::VerifyFull,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshTunnelConfigDto {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub private_key_path: String,
}

// ---------------------------------------------------------------------------
// Query DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResultDto {
    pub columns: Vec<ColumnMetaDto>,
    pub rows: Vec<RowDto>,
    pub row_count: u64,
    pub duration_ms: u64,
}

impl From<QueryResult> for QueryResultDto {
    fn from(r: QueryResult) -> Self {
        Self {
            columns: r.columns.into_iter().map(Into::into).collect(),
            rows: r.rows.into_iter().map(Into::into).collect(),
            row_count: r.row_count,
            duration_ms: r.duration_ms,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMetaDto {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl From<ColumnMeta> for ColumnMetaDto {
    fn from(c: ColumnMeta) -> Self {
        Self {
            name: c.name,
            data_type: c.data_type,
            nullable: c.nullable,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RowDto(pub Vec<CellValueDto>);

impl From<Row> for RowDto {
    fn from(r: Row) -> Self {
        Self(r.0.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum CellValueDto {
    Null,
    Bool(bool),
    Int64(i64),
    Float64(f64),
    Text(String),
    Bytes(Vec<u8>),
    Uuid(String),
    Datetime(String),
    Json(serde_json::Value),
}

impl From<CellValue> for CellValueDto {
    fn from(c: CellValue) -> Self {
        match c {
            CellValue::Null => Self::Null,
            CellValue::Bool(v) => Self::Bool(v),
            CellValue::Int64(v) => Self::Int64(v),
            CellValue::Float64(v) => Self::Float64(v),
            CellValue::Text(v) => Self::Text(v),
            CellValue::Bytes(v) => Self::Bytes(v),
            CellValue::Uuid(v) => Self::Uuid(v),
            CellValue::DateTime(v) => Self::Datetime(v),
            CellValue::Json(v) => Self::Json(v),
        }
    }
}

// ---------------------------------------------------------------------------
// Schema DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntrospectResultDto {
    pub schemas: Vec<SchemaDto>,
    pub tables: Vec<TableDto>,
    pub columns: Vec<SchemaColumnDto>,
    pub primary_keys: Vec<PrimaryKeyDto>,
    pub indexes: Vec<SchemaIndexDto>,
    pub foreign_keys: Vec<SchemaForeignKeyDto>,
    pub views: Vec<ViewDto>,
}

impl From<IntrospectResult> for IntrospectResultDto {
    fn from(r: IntrospectResult) -> Self {
        Self {
            schemas: r.schemas.into_iter().map(Into::into).collect(),
            tables: r.tables.into_iter().map(Into::into).collect(),
            columns: r.columns.into_iter().map(Into::into).collect(),
            primary_keys: r.primary_keys.into_iter().map(Into::into).collect(),
            indexes: r.indexes.into_iter().map(Into::into).collect(),
            foreign_keys: r.foreign_keys.into_iter().map(Into::into).collect(),
            views: r.views.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SchemaDto {
    pub name: String,
}

impl From<Schema> for SchemaDto {
    fn from(s: Schema) -> Self {
        Self { name: s.name }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableDto {
    pub name: String,
    pub schema: String,
    pub row_count: Option<u64>,
}

impl From<Table> for TableDto {
    fn from(t: Table) -> Self {
        Self {
            name: t.name,
            schema: t.schema,
            row_count: t.row_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaColumnDto {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub table_name: String,
    pub schema: String,
}

impl From<Column> for SchemaColumnDto {
    fn from(c: Column) -> Self {
        Self {
            name: c.name,
            data_type: c.data_type,
            nullable: c.nullable,
            default_value: c.default,
            is_primary_key: c.is_primary_key,
            table_name: c.table_name,
            schema: c.schema,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryKeyDto {
    pub constraint_name: String,
    pub columns: Vec<String>,
    pub table_name: String,
    pub schema: String,
}

impl From<PrimaryKey> for PrimaryKeyDto {
    fn from(pk: PrimaryKey) -> Self {
        Self {
            constraint_name: pk.constraint_name,
            columns: pk.columns,
            table_name: pk.table_name,
            schema: pk.schema,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaIndexDto {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub table_name: String,
    pub schema: String,
}

impl From<Index> for SchemaIndexDto {
    fn from(i: Index) -> Self {
        Self {
            name: i.name,
            columns: i.columns,
            unique: i.unique,
            table_name: i.table_name,
            schema: i.schema,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaForeignKeyDto {
    pub name: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub schema: String,
    pub to_schema: String,
}

impl From<ForeignKey> for SchemaForeignKeyDto {
    fn from(fk: ForeignKey) -> Self {
        Self {
            name: fk.name,
            from_table: fk.from_table,
            from_column: fk.from_column,
            to_table: fk.to_table,
            to_column: fk.to_column,
            schema: fk.schema,
            to_schema: fk.to_schema,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ViewDto {
    pub name: String,
    pub schema: String,
    pub definition: String,
}

impl From<View> for ViewDto {
    fn from(v: View) -> Self {
        Self {
            name: v.name,
            schema: v.schema,
            definition: v.definition,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfoDto {
    pub table: TableDto,
    pub columns: Vec<SchemaColumnDto>,
    pub primary_key: Option<PrimaryKeyDto>,
    pub indexes: Vec<SchemaIndexDto>,
    pub foreign_keys: Vec<SchemaForeignKeyDto>,
}

impl From<TableInfo> for TableInfoDto {
    fn from(info: TableInfo) -> Self {
        Self {
            table: info.table.into(),
            columns: info.columns.into_iter().map(Into::into).collect(),
            primary_key: info.primary_key.map(Into::into),
            indexes: info.indexes.into_iter().map(Into::into).collect(),
            foreign_keys: info.foreign_keys.into_iter().map(Into::into).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// History DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistoryDto {
    pub id: String,
    pub connection_id: String,
    pub sql: String,
    pub executed_at: String,
    pub duration_ms: u64,
    pub row_count: u64,
}

impl From<QueryHistory> for QueryHistoryDto {
    fn from(h: QueryHistory) -> Self {
        Self {
            id: h.id.to_string(),
            connection_id: h.connection_id.to_string(),
            sql: h.sql,
            executed_at: h.executed_at.to_rfc3339(),
            duration_ms: h.duration_ms,
            row_count: h.row_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQueryDto {
    pub id: String,
    pub connection_id: String,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
    pub created_at: String,
}

impl From<SavedQuery> for SavedQueryDto {
    fn from(q: SavedQuery) -> Self {
        Self {
            id: q.id.to_string(),
            connection_id: q.connection_id.to_string(),
            name: q.name,
            sql: q.sql,
            folder: q.folder,
            created_at: q.created_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Export DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResultDto {
    pub file_content: String,
    pub file_name: String,
    pub mime_type: String,
    pub row_count: u64,
}

impl From<db_pro_core::application::ExportResult> for ExportResultDto {
    fn from(r: db_pro_core::application::ExportResult) -> Self {
        use base64::Engine;
        Self {
            file_content: base64::engine::general_purpose::STANDARD.encode(&r.content),
            file_name: r.filename,
            mime_type: r.mime_type,
            row_count: r.row_count,
        }
    }
}

// ---------------------------------------------------------------------------
// Table Data DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRowsRequest {
    pub schema: String,
    pub table: String,
    pub filters: Vec<FilterDto>,
    pub sorts: Vec<SortDto>,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterDto {
    pub column: String,
    pub op: String,
    pub value: CellValueDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortDto {
    pub column: String,
    pub direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutateRowRequest {
    pub schema: String,
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<CellValueDto>,
    pub pk_columns: Option<Vec<String>>,
    pub pk_values: Option<Vec<CellValueDto>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRowsResultDto {
    pub columns: Vec<ColumnMetaDto>,
    pub rows: Vec<RowDto>,
    pub total_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutateRowResultDto {
    pub affected_rows: u64,
}

impl FilterDto {
    pub fn to_domain(&self) -> Result<db_pro_core::application::sql_builder::TableFilter, CommandError> {
        let op = match self.op.as_str() {
            "eq" => db_pro_core::application::sql_builder::FilterOp::Eq,
            "neq" => db_pro_core::application::sql_builder::FilterOp::Neq,
            "lt" => db_pro_core::application::sql_builder::FilterOp::Lt,
            "lte" => db_pro_core::application::sql_builder::FilterOp::Lte,
            "gt" => db_pro_core::application::sql_builder::FilterOp::Gt,
            "gte" => db_pro_core::application::sql_builder::FilterOp::Gte,
            "like" => db_pro_core::application::sql_builder::FilterOp::Like,
            "isNull" => db_pro_core::application::sql_builder::FilterOp::IsNull,
            "isNotNull" => db_pro_core::application::sql_builder::FilterOp::IsNotNull,
            other => {
                return Err(CommandError {
                    error: "VALIDATION".into(),
                    message: format!("unknown filter operator: {other}"),
                    message_id: "error.validation".into(),
                    details: None,
                })
            }
        };
        Ok(db_pro_core::application::sql_builder::TableFilter {
            column: self.column.clone(),
            op,
            value: self.value.clone().into(),
        })
    }
}

impl SortDto {
    pub fn to_domain(&self) -> db_pro_core::application::sql_builder::SortClause {
        let direction = match self.direction.as_str() {
            "desc" => db_pro_core::application::sql_builder::SortDir::Desc,
            _ => db_pro_core::application::sql_builder::SortDir::Asc,
        };
        db_pro_core::application::sql_builder::SortClause {
            column: self.column.clone(),
            direction,
        }
    }
}

impl From<CellValueDto> for CellValue {
    fn from(d: CellValueDto) -> Self {
        match d {
            CellValueDto::Null => CellValue::Null,
            CellValueDto::Bool(v) => CellValue::Bool(v),
            CellValueDto::Int64(v) => CellValue::Int64(v),
            CellValueDto::Float64(v) => CellValue::Float64(v),
            CellValueDto::Text(v) => CellValue::Text(v),
            CellValueDto::Bytes(v) => CellValue::Bytes(v),
            CellValueDto::Uuid(v) => CellValue::Uuid(v),
            CellValueDto::Datetime(v) => CellValue::DateTime(v),
            CellValueDto::Json(v) => CellValue::Json(v),
        }
    }
}
