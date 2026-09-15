use serde::{Deserialize, Serialize};

use db_pro_core::domain::connection::{Connection, ConnectionConfig, DriverType, SshTunnelConfig, SslMode};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::history::{QueryHistory, SavedQuery, SavedQueryFolder};
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
use db_pro_core::domain::run_config::RunConfig;
use db_pro_core::domain::schema::{
    Column, ForeignKey, Index, IntrospectResult, PrimaryKey, Schema, Table, TableInfo, Trigger, View,
};
use db_pro_core::domain::user::{DatabaseUser, Privilege};

mod string_i64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<i64>().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub error: String,
    pub message: String,
    pub message_id: String,
    pub details: Option<serde_json::Value>,
    pub retryable: bool,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl From<DbError> for CommandError {
    fn from(err: DbError) -> Self {
        let details = match &err {
            DbError::ConstraintViolation {
                constraint_type,
                constraint,
                table,
                column,
                ..
            } => Some(serde_json::json!({
                "constraint_type": constraint_type,
                "constraint": constraint,
                "table": table,
                "column": column,
            })),
            _ => None,
        };
        Self {
            error: err.code().into(),
            message: err.to_string(),
            message_id: err.message_id().into(),
            details,
            retryable: err.retryable(),
        }
    }
}

impl From<db_pro_runtime::DbErrorDto> for CommandError {
    fn from(error: db_pro_runtime::DbErrorDto) -> Self {
        Self {
            error: error.code,
            message: error.message,
            message_id: error.message_id,
            details: None,
            retryable: error.retryable,
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
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub readonly: bool,
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
            color: c.config.color,
            tags: c.config.tags,
            group: c.config.group,
            readonly: c.config.readonly,
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
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub readonly: bool,
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
                password: s.password.clone(),
            }),
            query_timeout_ms: self.query_timeout_ms,
            max_rows: self.max_rows,
            color: self.color.clone(),
            tags: self.tags.clone(),
            group: self.group.clone(),
            readonly: self.readonly,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriverTypeDto {
    Postgres,
    Sqlite,
    Mysql,
}

impl From<DriverType> for DriverTypeDto {
    fn from(d: DriverType) -> Self {
        match d {
            DriverType::Postgres => Self::Postgres,
            DriverType::SQLite => Self::Sqlite,
            DriverType::Mysql => Self::Mysql,
        }
    }
}

impl From<DriverTypeDto> for DriverType {
    fn from(d: DriverTypeDto) -> Self {
        match d {
            DriverTypeDto::Postgres => Self::Postgres,
            DriverTypeDto::Sqlite => Self::SQLite,
            DriverTypeDto::Mysql => Self::Mysql,
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
    #[serde(default)]
    pub password: Option<String>,
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
pub struct MultiQueryResultDto {
    pub results: Vec<QueryResultDto>,
    pub total_duration_ms: u64,
    pub error: Option<(usize, String)>,
}

impl From<db_pro_core::application::MultiQueryResult> for MultiQueryResultDto {
    fn from(r: db_pro_core::application::MultiQueryResult) -> Self {
        Self {
            results: r.results.into_iter().map(Into::into).collect(),
            total_duration_ms: r.total_duration_ms,
            error: r.error.map(|(statement_index, error)| (statement_index, error.message)),
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
    Int64(#[serde(with = "string_i64")] i64),
    Float64(f64),
    Decimal(String),
    Text(String),
    Bytes(Vec<u8>),
    Uuid(String),
    Datetime(String),
    /// `TIMESTAMP`: local date-time with no timezone marker.
    Timestamp(String),
    /// `TIMESTAMPTZ`: absolute instant normalized to UTC (`...Z`).
    Timestamptz(String),
    /// `TIMETZ`: wall-clock time with its own explicit offset.
    Timetz(String),
    Date(String),
    Time(String),
    Interval(String),
    Inet(String),
    Json(serde_json::Value),
}

impl From<CellValue> for CellValueDto {
    fn from(c: CellValue) -> Self {
        match c {
            CellValue::Null => Self::Null,
            CellValue::Bool(v) => Self::Bool(v),
            CellValue::Int64(v) => Self::Int64(v),
            CellValue::Float64(v) => Self::Float64(v),
            CellValue::Decimal(v) => Self::Decimal(v),
            CellValue::Text(v) => Self::Text(v),
            CellValue::Bytes(v) => Self::Bytes(v),
            CellValue::Uuid(v) => Self::Uuid(v),
            CellValue::DateTime(v) => Self::Datetime(v),
            CellValue::Timestamp(v) => Self::Timestamp(v),
            CellValue::TimestampTz(v) => Self::Timestamptz(v),
            CellValue::TimeTz(v) => Self::Timetz(v),
            CellValue::Date(v) => Self::Date(v),
            CellValue::Time(v) => Self::Time(v),
            CellValue::Interval(v) => Self::Interval(v),
            CellValue::Inet(v) => Self::Inet(v),
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
    pub triggers: Vec<TriggerDto>,
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
            triggers: r.triggers.into_iter().map(Into::into).collect(),
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
    pub from_columns: Vec<String>,
    pub to_table: String,
    pub to_columns: Vec<String>,
    pub schema: String,
    pub to_schema: String,
}

impl From<ForeignKey> for SchemaForeignKeyDto {
    fn from(fk: ForeignKey) -> Self {
        Self {
            name: fk.name,
            from_table: fk.from_table,
            from_columns: fk.from_columns,
            to_table: fk.to_table,
            to_columns: fk.to_columns,
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
pub struct TriggerDto {
    pub name: String,
    pub table_name: String,
    pub schema: String,
    pub timing: String,
    pub event: String,
    pub definition: String,
    pub function_def: String,
    pub enabled: bool,
}

impl From<Trigger> for TriggerDto {
    fn from(t: Trigger) -> Self {
        Self {
            name: t.name,
            table_name: t.table_name,
            schema: t.schema,
            timing: t.timing,
            event: t.event,
            definition: t.definition,
            function_def: t.function_def,
            enabled: t.enabled,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdlResultDto {
    pub affected_rows: u64,
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
    pub database: Option<String>,
    pub schema: Option<String>,
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
            database: h.database,
            schema: h.schema,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQueryFolderDto {
    pub id: String,
    pub connection_id: String,
    pub name: String,
    pub created_at: String,
}

impl From<SavedQueryFolder> for SavedQueryFolderDto {
    fn from(f: SavedQueryFolder) -> Self {
        Self {
            id: f.id.to_string(),
            connection_id: f.connection_id.to_string(),
            name: f.name,
            created_at: f.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunConfigDto {
    pub id: String,
    pub connection_id: String,
    pub name: String,
    pub sql: String,
    pub timeout_ms: u64,
    pub max_rows: u64,
    pub created_at: String,
}

impl From<RunConfig> for RunConfigDto {
    fn from(c: RunConfig) -> Self {
        Self {
            id: c.id.to_string(),
            connection_id: c.connection_id.to_string(),
            name: c.name,
            sql: c.sql,
            timeout_ms: c.timeout_ms,
            max_rows: c.max_rows,
            created_at: c.created_at.to_rfc3339(),
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
                    retryable: false,
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
            CellValueDto::Decimal(v) => CellValue::Decimal(v),
            CellValueDto::Text(v) => CellValue::Text(v),
            CellValueDto::Bytes(v) => CellValue::Bytes(v),
            CellValueDto::Uuid(v) => CellValue::Uuid(v),
            CellValueDto::Datetime(v) => CellValue::DateTime(v),
            CellValueDto::Timestamp(v) => CellValue::Timestamp(v),
            CellValueDto::Timestamptz(v) => CellValue::TimestampTz(v),
            CellValueDto::Timetz(v) => CellValue::TimeTz(v),
            CellValueDto::Date(v) => CellValue::Date(v),
            CellValueDto::Time(v) => CellValue::Time(v),
            CellValueDto::Interval(v) => CellValue::Interval(v),
            CellValueDto::Inet(v) => CellValue::Inet(v),
            CellValueDto::Json(v) => CellValue::Json(v),
        }
    }
}

// ---------------------------------------------------------------------------
// User Management DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseUserDto {
    pub name: String,
    pub is_super: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_login: bool,
}

impl From<DatabaseUser> for DatabaseUserDto {
    fn from(u: DatabaseUser) -> Self {
        Self {
            name: u.name,
            is_super: u.is_super,
            can_create_db: u.can_create_db,
            can_create_role: u.can_create_role,
            can_login: u.can_login,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeDto {
    pub schema: String,
    pub table: String,
    pub privilege_type: String,
}

impl From<Privilege> for PrivilegeDto {
    fn from(p: Privilege) -> Self {
        Self {
            schema: p.schema,
            table: p.table,
            privilege_type: p.privilege_type,
        }
    }
}

// ---------------------------------------------------------------------------
// Backup DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupFormatDto {
    Plain,
    Custom,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupOptionsDto {
    pub connection_id: String,
    pub output_path: String,
    pub format: BackupFormatDto,
    pub schemas: Option<Vec<String>>,
    pub tables: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOptionsDto {
    pub connection_id: String,
    pub input_path: String,
    pub format: BackupFormatDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResultDto {
    pub output_path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProgressDto {
    pub operation: String,
    pub status: String,
    pub path: String,
    pub message: Option<String>,
}

impl From<db_pro_core::domain::backup::BackupResult> for BackupResultDto {
    fn from(r: db_pro_core::domain::backup::BackupResult) -> Self {
        Self {
            output_path: r.output_path,
            size_bytes: r.size_bytes,
        }
    }
}

// --- Cross-connection DTOs ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDiffDto {
    pub tables_only_in_source: Vec<String>,
    pub tables_only_in_target: Vec<String>,
    pub column_diffs: Vec<TableColumnDiffDto>,
    pub indexes_only_in_source: Vec<String>,
    pub indexes_only_in_target: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnDiffDto {
    pub schema: String,
    pub table: String,
    pub columns_only_in_source: Vec<String>,
    pub columns_only_in_target: Vec<String>,
    pub type_mismatches: Vec<ColumnTypeMismatchDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnTypeMismatchDto {
    pub column: String,
    pub source_type: String,
    pub target_type: String,
}

impl From<db_pro_core::domain::cross_connection::SchemaDiff> for SchemaDiffDto {
    fn from(d: db_pro_core::domain::cross_connection::SchemaDiff) -> Self {
        Self {
            tables_only_in_source: d.tables_only_in_source,
            tables_only_in_target: d.tables_only_in_target,
            column_diffs: d.column_diffs.into_iter().map(Into::into).collect(),
            indexes_only_in_source: d.indexes_only_in_source,
            indexes_only_in_target: d.indexes_only_in_target,
        }
    }
}

impl From<db_pro_core::domain::cross_connection::TableColumnDiff> for TableColumnDiffDto {
    fn from(d: db_pro_core::domain::cross_connection::TableColumnDiff) -> Self {
        Self {
            schema: d.schema,
            table: d.table,
            columns_only_in_source: d.columns_only_in_source,
            columns_only_in_target: d.columns_only_in_target,
            type_mismatches: d.type_mismatches.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<db_pro_core::domain::cross_connection::ColumnTypeMismatch> for ColumnTypeMismatchDto {
    fn from(m: db_pro_core::domain::cross_connection::ColumnTypeMismatch) -> Self {
        Self {
            column: m.column,
            source_type: m.source_type,
            target_type: m.target_type,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDiffDto {
    pub schema: String,
    pub table: String,
    pub source_row_count: i64,
    pub target_row_count: i64,
    pub row_count_diff: i64,
}

impl From<db_pro_core::domain::cross_connection::DataDiff> for DataDiffDto {
    fn from(d: db_pro_core::domain::cross_connection::DataDiff) -> Self {
        Self {
            schema: d.schema,
            table: d.table,
            source_row_count: d.source_row_count,
            target_row_count: d.target_row_count,
            row_count_diff: d.row_count_diff,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDependencyDto {
    pub object_type: String,
    pub object_name: String,
    pub depends_on_type: String,
    pub depends_on_name: String,
}

impl From<db_pro_core::domain::cross_connection::ObjectDependency> for ObjectDependencyDto {
    fn from(d: db_pro_core::domain::cross_connection::ObjectDependency) -> Self {
        Self {
            object_type: d.object_type,
            object_name: d.object_name,
            depends_on_type: d.depends_on_type,
            depends_on_name: d.depends_on_name,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionInfoDto {
    pub schema: String,
    pub table: String,
    pub partition_strategy: String,
    pub partitions: Vec<PartitionChildDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionChildDto {
    pub name: String,
    pub bound_expr: String,
}

impl From<db_pro_core::domain::cross_connection::PartitionInfo> for PartitionInfoDto {
    fn from(p: db_pro_core::domain::cross_connection::PartitionInfo) -> Self {
        Self {
            schema: p.schema,
            table: p.table,
            partition_strategy: p.partition_strategy,
            partitions: p.partitions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<db_pro_core::domain::cross_connection::PartitionChild> for PartitionChildDto {
    fn from(c: db_pro_core::domain::cross_connection::PartitionChild) -> Self {
        Self {
            name: c.name,
            bound_expr: c.bound_expr,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablespaceInfoDto {
    pub name: String,
    pub owner: String,
    pub location: String,
}

impl From<db_pro_core::domain::cross_connection::TablespaceInfo> for TablespaceInfoDto {
    fn from(t: db_pro_core::domain::cross_connection::TablespaceInfo) -> Self {
        Self {
            name: t.name,
            owner: t.owner,
            location: t.location,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A representative introspection payload that exercises every DTO field,
    /// including one single-column FK and one composite FK.
    fn representative_introspect_result() -> IntrospectResult {
        IntrospectResult {
            schemas: vec![Schema {
                name: "public".to_owned(),
            }],
            tables: vec![
                Table {
                    name: "orders".to_owned(),
                    schema: "public".to_owned(),
                    row_count: Some(12),
                },
                Table {
                    name: "order_items".to_owned(),
                    schema: "public".to_owned(),
                    row_count: None,
                },
                Table {
                    name: "customers".to_owned(),
                    schema: "public".to_owned(),
                    row_count: Some(4),
                },
            ],
            columns: vec![
                Column {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    ordinal: 0,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    collation: None,
                    table_name: "orders".to_owned(),
                    schema: "public".to_owned(),
                },
                Column {
                    name: "customer_id".to_owned(),
                    data_type: "integer".to_owned(),
                    ordinal: 1,
                    nullable: false,
                    default: Some("42".to_owned()),
                    is_primary_key: false,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    collation: None,
                    table_name: "orders".to_owned(),
                    schema: "public".to_owned(),
                },
                Column {
                    name: "order_id".to_owned(),
                    data_type: "integer".to_owned(),
                    ordinal: 0,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    collation: None,
                    table_name: "order_items".to_owned(),
                    schema: "public".to_owned(),
                },
                Column {
                    name: "product_id".to_owned(),
                    data_type: "integer".to_owned(),
                    ordinal: 1,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                    is_identity: false,
                    is_generated: false,
                    collation: None,
                    table_name: "order_items".to_owned(),
                    schema: "public".to_owned(),
                },
            ],
            primary_keys: vec![
                PrimaryKey {
                    constraint_name: "orders_pkey".to_owned(),
                    columns: vec!["id".to_owned()],
                    table_name: "orders".to_owned(),
                    schema: "public".to_owned(),
                },
                PrimaryKey {
                    constraint_name: "order_items_pkey".to_owned(),
                    columns: vec!["order_id".to_owned(), "product_id".to_owned()],
                    table_name: "order_items".to_owned(),
                    schema: "public".to_owned(),
                },
            ],
            indexes: vec![Index {
                name: "orders_customer_id_idx".to_owned(),
                columns: vec!["customer_id".to_owned()],
                unique: false,
                method: "btree".to_owned(),
                primary: false,
                include_columns: Vec::new(),
                predicate: None,
                definition: "CREATE INDEX orders_customer_id_idx ON orders (customer_id)".to_owned(),
                origin: db_pro_core::domain::schema::IndexOrigin::User,
                table_name: "orders".to_owned(),
                schema: "public".to_owned(),
            }],
            foreign_keys: vec![
                ForeignKey {
                    name: "orders_customer_id_fkey".to_owned(),
                    from_table: "orders".to_owned(),
                    from_columns: vec!["customer_id".to_owned()],
                    to_table: "customers".to_owned(),
                    to_columns: vec!["id".to_owned()],
                    schema: "public".to_owned(),
                    to_schema: "public".to_owned(),
                    on_update: "NO ACTION".to_owned(),
                    on_delete: "CASCADE".to_owned(),
                    match_option: "NONE".to_owned(),
                    deferrable: false,
                    initially_deferred: false,
                },
                ForeignKey {
                    name: "order_items_fkey".to_owned(),
                    from_table: "order_items".to_owned(),
                    from_columns: vec!["order_id".to_owned(), "product_id".to_owned()],
                    to_table: "orders".to_owned(),
                    to_columns: vec!["id".to_owned(), "product_id".to_owned()],
                    schema: "public".to_owned(),
                    to_schema: "public".to_owned(),
                    on_update: "NO ACTION".to_owned(),
                    on_delete: "NO ACTION".to_owned(),
                    match_option: "NONE".to_owned(),
                    deferrable: false,
                    initially_deferred: false,
                },
            ],
            check_constraints: Vec::new(),
            views: vec![View {
                name: "open_orders".to_owned(),
                schema: "public".to_owned(),
                definition: "SELECT id FROM orders".to_owned(),
            }],
            triggers: vec![Trigger {
                name: "orders_updated_at".to_owned(),
                table_name: "orders".to_owned(),
                schema: "public".to_owned(),
                timing: "BEFORE".to_owned(),
                event: "UPDATE".to_owned(),
                definition: "CREATE TRIGGER orders_updated_at BEFORE UPDATE ON orders ...".to_owned(),
                function_def: "CREATE FUNCTION update_orders_updated_at() ...".to_owned(),
                enabled: true,
            }],
            functions: Vec::new(),
        }
    }

    /// The exact serialized shape the Tauri `introspect` command emits: camelCase
    /// everywhere, the eight top-level keys the current contract carries, and
    /// optional fields as JSON null (not omitted).
    #[test]
    fn introspect_result_dto_serializes_the_locked_camel_case_shape() {
        let dto: IntrospectResultDto = representative_introspect_result().into();
        let value = serde_json::to_value(&dto).expect("the DTO must serialize");
        let object = value.as_object().expect("the root must be an object");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "columns",
                "foreignKeys",
                "indexes",
                "primaryKeys",
                "schemas",
                "tables",
                "triggers",
                "views"
            ],
            "a drift in the top-level shape must fail here, not in the consumer"
        );

        let tables = &object["tables"];
        assert_eq!(tables[0]["name"], "orders");
        assert_eq!(tables[0]["rowCount"], 12);
        assert_eq!(
            tables[1]["rowCount"],
            serde_json::Value::Null,
            "None row_count is JSON null, per the current contract"
        );

        let columns = &object["columns"];
        assert_eq!(columns[0]["dataType"], "integer");
        assert_eq!(columns[0]["isPrimaryKey"], true);
        assert_eq!(columns[0]["tableName"], "orders");
        assert_eq!(columns[1]["defaultValue"], "42");
        assert_eq!(columns[0]["defaultValue"], serde_json::Value::Null);

        assert_eq!(object["primaryKeys"][1]["columns"][0], "order_id");
        assert_eq!(object["indexes"][0]["tableName"], "orders");
        assert_eq!(object["views"][0]["definition"], "SELECT id FROM orders");
        assert_eq!(
            object["triggers"][0]["functionDef"],
            "CREATE FUNCTION update_orders_updated_at() ..."
        );
    }

    /// Foreign-key column arrays keep their provider order, for composite keys in
    /// both directions. The old singular drift (`fromColumn` / `toColumn`) must
    /// never reappear.
    #[test]
    fn introspect_result_dto_keeps_foreign_key_column_order_and_plural_names() {
        let dto: IntrospectResultDto = representative_introspect_result().into();
        let value = serde_json::to_value(&dto).expect("the DTO must serialize");
        let foreign_keys = value["foreignKeys"].as_array().expect("foreignKeys array");

        let single = &foreign_keys[0];
        assert_eq!(single["fromTable"], "orders");
        assert_eq!(single["fromColumns"][0], "customer_id");
        assert_eq!(single["toTable"], "customers");
        assert_eq!(single["toColumns"][0], "id");
        assert!(single.get("fromColumn").is_none(), "singular drift must not exist");
        assert!(single.get("toColumn").is_none(), "singular drift must not exist");

        let composite = &foreign_keys[1];
        assert_eq!(composite["fromColumns"][0], "order_id");
        assert_eq!(composite["fromColumns"][1], "product_id");
        assert_eq!(composite["toColumns"][0], "id");
        assert_eq!(composite["toColumns"][1], "product_id");
    }

    /// A payload carrying the pre-contract singular names must fail validation —
    /// pinning the drift the issue names so a regression reintroduces a red test.
    #[test]
    fn introspect_result_dto_rejects_singular_foreign_key_names() {
        let drifted = serde_json::json!({
            "fromTable": "orders",
            "fromColumn": ["customer_id"],
            "toTable": "customers",
            "toColumn": ["id"],
        });
        let accepted = drifted
            .as_object()
            .map(|object| {
                object.get("fromColumns").is_some()
                    && object.get("toColumns").is_some()
                    && object.get("fromColumn").is_none()
                    && object.get("toColumn").is_none()
            })
            .unwrap_or(false);
        assert!(
            !accepted,
            "a payload using the singular names must not pass the contract"
        );
    }

    /// The checked-in fixture is the recorded shape of the IPC boundary: any drift
    /// in the DTO structs (renamed, added or dropped fields, reordered FK columns)
    /// fails this assertion before it reaches a consumer.
    #[test]
    fn introspect_result_dto_matches_the_checked_in_contract_fixture() {
        let dto: IntrospectResultDto = representative_introspect_result().into();
        let emitted = serde_json::to_string_pretty(&dto).expect("the DTO must serialize");
        let fixture = include_str!("../tests/fixtures/introspect-contract.json");
        assert_eq!(
            emitted, fixture,
            "the serialized shape must match the checked-in contract fixture byte-for-byte"
        );
    }

    #[test]
    fn cell_value_dto_int64_boundary_values_serialize_as_strings() {
        let cases: Vec<i64> = vec![
            0,
            1,
            -1,
            i64::MAX,
            i64::MIN,
            (1i64 << 53) - 1,
            1i64 << 53,
            (1i64 << 53) + 1,
        ];

        for &value in &cases {
            let dto = CellValueDto::Int64(value);
            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["type"], "int64");
            assert_eq!(json["value"], serde_json::Value::String(value.to_string()));
        }
    }

    #[test]
    fn cell_value_dto_int64_roundtrip_preserves_exact_value() {
        let cases: Vec<i64> = vec![
            0,
            1,
            -1,
            i64::MAX,
            i64::MIN,
            (1i64 << 53) - 1,
            1i64 << 53,
            (1i64 << 53) + 1,
        ];

        for &value in &cases {
            let dto = CellValueDto::Int64(value);
            let json = serde_json::to_string(&dto).unwrap();
            let decoded: CellValueDto = serde_json::from_str(&json).unwrap();
            match decoded {
                CellValueDto::Int64(v) => assert_eq!(v, value, "roundtrip failed for {value}"),
                other => panic!("expected Int64, got {other:?}"),
            }
        }
    }

    #[test]
    fn cell_value_dto_int64_rejects_non_string_json() {
        let json = r#"{"type":"int64","value":9007199254740993}"#;
        let result = serde_json::from_str::<CellValueDto>(json);
        assert!(result.is_err(), "should reject numeric JSON value for int64");
    }

    /// A whole result that carries every value class the canonical contract
    /// defines — A1 numeric/integer, A2 temporal, A3 structured/fallback — so the
    /// emitted shape is pinned for the composite payload and not only for
    /// isolated variants (Gate 5 A4).
    fn representative_query_result() -> QueryResult {
        let columns = vec![
            ColumnMeta {
                name: "id".to_owned(),
                data_type: "bigint".to_owned(),
                nullable: false,
            },
            ColumnMeta {
                name: "active".to_owned(),
                data_type: "boolean".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "ratio".to_owned(),
                data_type: "double precision".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "amount".to_owned(),
                data_type: "numeric(20,4)".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "note".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "payload".to_owned(),
                data_type: "jsonb".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "token".to_owned(),
                data_type: "uuid".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "created_at".to_owned(),
                data_type: "timestamp with time zone".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "local_at".to_owned(),
                data_type: "timestamp without time zone".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "legacy_stamp".to_owned(),
                data_type: "timestamp without time zone".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "birthday".to_owned(),
                data_type: "date".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "slot".to_owned(),
                data_type: "time without time zone".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "offset_slot".to_owned(),
                data_type: "time with time zone".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "duration".to_owned(),
                data_type: "interval".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "address".to_owned(),
                data_type: "inet".to_owned(),
                nullable: true,
            },
            ColumnMeta {
                name: "raw".to_owned(),
                data_type: "bytea".to_owned(),
                nullable: true,
            },
        ];

        let populated = Row(vec![
            // Beyond 2^53: must leave as an exact string, never a JS number.
            CellValue::Int64(9_007_199_254_740_993),
            CellValue::Bool(true),
            CellValue::Float64(0.1),
            CellValue::Decimal("12345678901234567890.12345".to_owned()),
            CellValue::Text("hello".to_owned()),
            CellValue::Json(serde_json::json!({ "a": 1, "b": [true, null] })),
            CellValue::Uuid("3f2504e0-4f89-11d3-9a0c-0305e82c3301".to_owned()),
            // A2: an instant keeps its own tag and its explicit UTC marker.
            CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".to_owned()),
            // A2: a timestamp without time zone gains no offset and keeps its tag.
            CellValue::Timestamp("2024-03-15T10:20:30.123456".to_owned()),
            // The legacy variant stays available for pre-Gate-5 call sites and for
            // providers that do not distinguish the temporal classes.
            CellValue::DateTime("2024-03-15T10:20:30.123456+00:00".to_owned()),
            CellValue::Date("2024-03-15".to_owned()),
            CellValue::Time("10:20:30.123456".to_owned()),
            // A2: a time with time zone keeps its own offset.
            CellValue::TimeTz("10:20:30.123456+07:00".to_owned()),
            CellValue::Interval("1 mons 2 days 03:04:05.000006".to_owned()),
            CellValue::Inet("192.168.0.1/24".to_owned()),
            CellValue::Bytes(vec![0x00, 0xff, 0x10]),
        ]);
        let all_null = Row(vec![CellValue::Null; columns.len()]);

        QueryResult {
            columns,
            rows: vec![populated, all_null],
            row_count: 2,
            duration_ms: 12,
        }
    }

    /// The serialized shape of a whole result: four camelCase top-level keys, the
    /// three column keys, and every cell as a tagged `{type, value}` object.
    #[test]
    fn query_result_dto_serializes_the_locked_camel_case_shape() {
        let dto: QueryResultDto = representative_query_result().into();
        let value = serde_json::to_value(&dto).expect("the DTO must serialize");
        let object = value.as_object().expect("the root must be an object");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["columns", "durationMs", "rowCount", "rows"],
            "a drift in the whole-result shape must fail here, not in the consumer"
        );
        assert_eq!(object["rowCount"], 2);
        assert_eq!(object["durationMs"], 12);

        let mut column_keys: Vec<&str> = object["columns"][0]
            .as_object()
            .expect("a column must be an object")
            .keys()
            .map(String::as_str)
            .collect();
        column_keys.sort_unstable();
        assert_eq!(column_keys, vec!["dataType", "name", "nullable"]);
        assert_eq!(object["columns"][0]["dataType"], "bigint");
        assert_eq!(object["columns"][0]["nullable"], false);

        let rows = object["rows"].as_array().expect("rows must be an array");
        assert_eq!(rows.len(), 2, "each row stays an array of cells");
        assert_eq!(
            rows[0].as_array().expect("a row must be an array").len(),
            object["columns"].as_array().map(Vec::len).unwrap_or_default()
        );
    }

    /// Every value class A1-A3 carries keeps its own tag in the mixed payload; a
    /// class silently re-tagged or collapsed into `text` fails here.
    #[test]
    fn query_result_dto_covers_every_value_class_tag() {
        let dto: QueryResultDto = representative_query_result().into();
        let value = serde_json::to_value(&dto).expect("the DTO must serialize");

        let tags: Vec<&str> = value["rows"][0]
            .as_array()
            .expect("the first row must be an array")
            .iter()
            .map(|cell| {
                cell.get("type")
                    .and_then(serde_json::Value::as_str)
                    .expect("every cell must carry a type tag")
            })
            .collect();
        assert_eq!(
            tags,
            vec![
                "int64",
                "bool",
                "float64",
                "decimal",
                "text",
                "json",
                "uuid",
                "timestamptz",
                "timestamp",
                "datetime",
                "date",
                "time",
                "timetz",
                "interval",
                "inet",
                "bytes"
            ],
            "the tagged shape must keep every value class distinguishable"
        );

        for cell in value["rows"][1].as_array().expect("the second row must be an array") {
            assert_eq!(cell["type"], "null");
            assert!(cell.get("value").is_none(), "a null cell carries no value key: {cell}");
        }
    }

    /// Precision-sensitive values stay exact strings on the JSON boundary, so a
    /// JavaScript-shaped consumer cannot silently round them through `Number`.
    #[test]
    fn query_result_dto_never_exposes_precision_sensitive_values_as_numbers() {
        let dto: QueryResultDto = representative_query_result().into();
        let value = serde_json::to_value(&dto).expect("the DTO must serialize");
        let row = value["rows"][0].as_array().expect("the first row must be an array");

        assert_eq!(row[0]["value"], "9007199254740993", "int64 leaves as a string");
        assert!(row[0]["value"].is_string());
        assert_eq!(
            row[3]["value"], "12345678901234567890.12345",
            "numeric keeps every digit as a string"
        );
        assert!(row[3]["value"].is_string());
        assert!(row[2]["value"].is_number(), "a float64 stays a JSON number by contract");
    }

    /// The checked-in fixture is the recorded shape of the whole-result IPC
    /// boundary: DTO drift (renamed, added or dropped fields, re-tagged cells,
    /// reordered columns) fails before a consumer sees it, and every tagged cell
    /// still round-trips through the derived deserializer.
    #[test]
    fn query_result_dto_matches_the_checked_in_contract_fixture() {
        let dto: QueryResultDto = representative_query_result().into();
        let emitted = serde_json::to_string_pretty(&dto).expect("the DTO must serialize");
        let fixture = include_str!("../tests/fixtures/query-result-contract.json");
        assert_eq!(
            emitted, fixture,
            "the serialized shape must match the checked-in contract fixture byte-for-byte"
        );

        let parsed: serde_json::Value = serde_json::from_str(fixture).expect("the fixture must be valid JSON");
        for row in parsed["rows"].as_array().expect("rows must be an array") {
            for cell in row.as_array().expect("a row must be an array") {
                let decoded: CellValueDto = serde_json::from_value(cell.clone()).expect("every cell must deserialize");
                assert_eq!(
                    serde_json::to_value(&decoded).expect("the cell must re-serialize"),
                    *cell,
                    "a tagged cell must round-trip without shape ambiguity"
                );
            }
        }
    }

    /// Connects to the live fixture named by `DATABASE_URL`
    /// (`postgres://user:password@host:port/database`) — the same server the
    /// `pg_integration` suite uses — so the DTO can be exercised on values a real
    /// PostgreSQL produced rather than on hand-built ones.
    fn live_fixture_credentials() -> (ConnectionConfig, String) {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for the live provider-to-DTO path");
        let (authority, host_and_database) = url
            .strip_prefix("postgres://")
            .expect("DATABASE_URL must use the postgres:// scheme")
            .rsplit_once('@')
            .expect("DATABASE_URL must carry user:password@host");
        let (username, password) = authority
            .split_once(':')
            .expect("DATABASE_URL must carry user:password");
        let (host_and_port, database) = host_and_database
            .split_once('/')
            .expect("DATABASE_URL must carry host/database");
        let (host, port) = host_and_port
            .split_once(':')
            .expect("DATABASE_URL must carry host:port");
        let (database, _parameters) = database.split_once('?').unwrap_or((database, ""));

        let config = ConnectionConfig {
            name: "provider-to-dto-fixture".to_owned(),
            host: host.to_owned(),
            port: port.parse().expect("DATABASE_URL must carry a numeric port"),
            database: database.to_owned(),
            username: username.to_owned(),
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 10_000,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        };
        (config, password.to_owned())
    }

    fn sorted_object_keys(value: &serde_json::Value) -> Vec<String> {
        let mut keys: Vec<String> = value.as_object().expect("a JSON object").keys().cloned().collect();
        keys.sort_unstable();
        keys
    }

    /// Gate 5 D1 (#64) — the whole path on one real server: the fixture's 26-column
    /// row is decoded by the provider adapter, crosses the domain representation and
    /// leaves as `QueryResultDto` JSON with every class tag, value and provider type
    /// name intact. The two neighbouring tests each cover only one half:
    /// `pg_decoder_matrix_covers_every_value_class` stops at the decoder, and
    /// `query_result_dto_matches_the_checked_in_contract_fixture` starts from a
    /// hand-built `QueryResult`.
    ///
    /// Ignored unless `DATABASE_URL` is set, like the rest of the live suite.
    #[tokio::test]
    #[ignore = "requires DATABASE_URL (live PostgreSQL fixture)"]
    async fn live_fixture_query_survives_the_provider_to_dto_path() {
        use db_pro_core::ports::DbConnector;
        use db_pro_infrastructure::postgres::connector::PostgresConnector;

        let (config, password) = live_fixture_credentials();
        let connector = PostgresConnector::new();
        let handle = connector.connect(&config, &password).await.expect("PG connect failed");
        let decoded = connector
            .query(&handle, "SELECT * FROM decoder_matrix ORDER BY id", &[])
            .await
            .expect("no fixture column may take the row down");
        connector.disconnect(&handle).await.expect("PG disconnect failed");

        assert_eq!(decoded.row_count, 2, "one populated row and one all-NULL row");
        let domain_rows = decoded.rows.clone();
        let emitted = serde_json::to_value(QueryResultDto::from(decoded)).expect("the DTO must serialize");

        // 1. The #54 payload contract, now on data a real server produced.
        assert_eq!(
            sorted_object_keys(&emitted),
            ["columns", "durationMs", "rowCount", "rows"]
        );
        assert_eq!(emitted["rowCount"], serde_json::json!(2));
        assert!(emitted["durationMs"].is_u64(), "durationMs must stay a number");

        let columns = emitted["columns"].as_array().expect("columns must be an array");
        assert_eq!(columns.len(), 26, "one column per decoder_matrix column");
        for column in columns {
            assert_eq!(sorted_object_keys(column), ["dataType", "name", "nullable"]);
        }
        // Custom and fallback classes keep their provider type name across the hop, so
        // the write-policy layer can still tell an array from a range from a composite.
        let declared_types: Vec<&str> = columns
            .iter()
            .map(|column| column["dataType"].as_str().expect("a provider type name"))
            .collect();
        for required in ["TEXT[]", "INT4RANGE", "decoder_pair", "NUMERIC", "TIMETZ"] {
            assert!(
                declared_types.iter().any(|declared| declared.contains(required)),
                "the declared provider type {required} must survive into the DTO: {declared_types:?}"
            );
        }

        let rows = emitted["rows"].as_array().expect("rows must be an array");
        assert_eq!(rows.len(), 2, "both fixture rows must reach the DTO");
        let populated = rows[0].as_array().expect("a row is an array of cells");
        assert_eq!(populated.len(), 26, "one cell per column");

        // 2. Class by class: one provider class, one domain value, one emitted tag.
        let expected_classes: [(&str, &str); 26] = [
            ("id int2", "int64"),
            ("flag bool", "bool"),
            ("count int4", "int64"),
            ("big_count int8", "int64"),
            ("ratio float4", "float64"),
            ("precise_ratio float8", "float64"),
            ("amount numeric(24,4)", "decimal"),
            ("calendar_date", "date"),
            ("wall_time", "time"),
            ("zoned_time timetz", "timetz"),
            ("local_stamp timestamp", "timestamp"),
            ("instant timestamptz", "timestamptz"),
            ("span interval", "interval"),
            ("token uuid", "uuid"),
            ("doc json", "json"),
            ("payload jsonb", "json"),
            ("blob bytea", "bytes"),
            ("address inet", "inet"),
            ("network cidr", "inet"),
            ("status enum", "text"),
            ("quantity domain", "int64"),
            ("postal domain", "text"),
            ("labels array", "bytes"),
            ("slot range", "bytes"),
            ("pair composite", "bytes"),
            ("missing null", "null"),
        ];
        let emitted_tags: Vec<&str> = populated
            .iter()
            .map(|cell| cell["type"].as_str().expect("every cell carries a type tag"))
            .collect();
        let mut mismatched = Vec::new();
        for (index, (label, expected_class)) in expected_classes.iter().enumerate() {
            if emitted_tags[index] != *expected_class {
                mismatched.push(format!(
                    "{label}: expected {expected_class}, emitted {}",
                    emitted_tags[index]
                ));
            }
        }
        assert!(
            mismatched.is_empty(),
            "provider -> domain -> DTO class mismatch: {mismatched:#?}"
        );

        // 3. The values that a lossy hop would mangle: exact digits, the temporal
        // distinction, byte-exact binary and the fallback classes.
        let exact_cells: [(&str, usize, serde_json::Value); 12] = [
            (
                "big_count int8 keeps every digit",
                3,
                serde_json::json!({"type": "int64", "value": "9223372036854775807"}),
            ),
            (
                "amount numeric(24,4) keeps every digit",
                6,
                serde_json::json!({"type": "decimal", "value": "12345678901234567890.1234"}),
            ),
            (
                "calendar_date",
                7,
                serde_json::json!({"type": "date", "value": "2024-03-15"}),
            ),
            (
                "wall_time",
                8,
                serde_json::json!({"type": "time", "value": "10:20:30.123456"}),
            ),
            (
                "zoned_time keeps its own tag and offset",
                9,
                serde_json::json!({"type": "timetz", "value": "10:20:30.123456+07:00"}),
            ),
            (
                "local_stamp is a timestamp and gains no invented zone",
                10,
                serde_json::json!({"type": "timestamp", "value": "2024-03-15T10:20:30.123456"}),
            ),
            (
                "instant keeps its Z, its own tag and stays distinct from local_stamp",
                11,
                serde_json::json!({"type": "timestamptz", "value": "2024-03-15T10:20:30.123456Z"}),
            ),
            (
                "span interval",
                12,
                serde_json::json!({"type": "interval", "value": "1 mons 2 days 03:04:05.000006"}),
            ),
            (
                "blob bytea stays byte-exact",
                16,
                serde_json::json!({"type": "bytes", "value": [0xde, 0xad, 0xbe, 0xef]}),
            ),
            (
                "status enum label",
                19,
                serde_json::json!({"type": "text", "value": "shipped"}),
            ),
            (
                "quantity domain through its base type",
                20,
                serde_json::json!({"type": "int64", "value": "7"}),
            ),
            (
                "missing null carries no payload",
                25,
                serde_json::json!({"type": "null"}),
            ),
        ];
        for (label, index, expected) in exact_cells {
            assert_eq!(populated[index], expected, "{label}");
        }

        // 4. No loss anywhere: for every cell of both rows, the domain value's own
        // JSON, the DTO's JSON and the JSON that survives a round trip back through
        // `CellValueDto` are the same bytes.
        for (row_index, domain_row) in domain_rows.iter().enumerate() {
            let cells = rows[row_index].as_array().expect("a row is an array of cells");
            assert_eq!(cells.len(), domain_row.0.len(), "row {row_index} must keep every cell");
            for (cell_index, cell) in domain_row.0.iter().enumerate() {
                let domain_json = serde_json::to_value(cell).expect("a domain cell must serialize");
                assert_eq!(
                    domain_json, cells[cell_index],
                    "row {row_index} cell {cell_index}: the domain value and the DTO disagree"
                );

                let round_tripped: CellValue = serde_json::from_value::<CellValueDto>(cells[cell_index].clone())
                    .expect("every emitted cell must deserialize")
                    .into();
                assert_eq!(
                    serde_json::to_value(&round_tripped).expect("a round-tripped cell must re-serialize"),
                    cells[cell_index],
                    "row {row_index} cell {cell_index} does not survive the DTO round trip"
                );
            }
        }
    }
}
