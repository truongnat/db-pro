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
            error: r.error,
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
            CellValueDto::Text(v) => CellValue::Text(v),
            CellValueDto::Bytes(v) => CellValue::Bytes(v),
            CellValueDto::Uuid(v) => CellValue::Uuid(v),
            CellValueDto::Datetime(v) => CellValue::DateTime(v),
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

    #[test]
    fn cell_value_dto_int64_boundary_values_serialize_as_strings() {
        let cases: Vec<i64> = vec![0, 1, -1, i64::MAX, i64::MIN, (1i64 << 53) - 1, 1i64 << 53, (1i64 << 53) + 1];

        for &value in &cases {
            let dto = CellValueDto::Int64(value);
            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["type"], "int64");
            assert_eq!(json["value"], serde_json::Value::String(value.to_string()));
        }
    }

    #[test]
    fn cell_value_dto_int64_roundtrip_preserves_exact_value() {
        let cases: Vec<i64> = vec![0, 1, -1, i64::MAX, i64::MIN, (1i64 << 53) - 1, 1i64 << 53, (1i64 << 53) + 1];

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
}
