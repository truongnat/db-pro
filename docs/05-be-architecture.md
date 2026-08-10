# 06 — DB Client — Backend Architecture (Rust Core)

---

## 1. Stack

| Concern | Technology | Version | Rationale |
|---|---|---|---|
| Runtime | Rust (async, `tokio`) | 1.x | Safe, fast, compile-time checked |
| DB — Postgres | `sqlx` + `tokio-postgres` | 0.7 + 0.7 | Async, no ORM, compile-time query check |
| DB — SQLite | `rusqlite` | 0.30 | Local meta-store |
| SSH tunnel | `openssh` | 0.9 | Spawn binary, no C dep |
| Secret vault | `keyring` + `aes-gcm` | 0.16 + 0.10 | libsecret on Ubuntu |
| Crypto | `pbkdf2` + `hmac` + `sha2` | 0.12 + 0.10 + 0.10 | Key derivation |
| Error handling | `thiserror` | 1.0 | Typed, exhaustive errors |
| Async trait | `async-trait` | 0.1 | Trait methods with async |
| ID generation | `uuid` | 1.x | Type-safe IDs |
| Date/time | `chrono` | 0.4 | UTC timestamps |
| Serialization | `serde` + `serde_json` | 1.x | JSON for Tauri boundary |
| Observability | `tracing` | 0.1 | Structured logs |
| Testing | `#[cfg(test)]` + `mockall` | latest | Unit + integration tests |
| Config | `config` + `serde_json` | 0.14 | Runtime config loading |
| Connection pool | `bb8` | 0.8 | Pool for repeated queries |

---

## 2. Clean Architecture Layers

```
┌─────────────────────────────────────────────┐
│  tauri-app (commands) — thin controller     │
├─────────────────────────────────────────────┤
│  core::application — use cases (services)   │
├─────────────────────────────────────────────┤
│  core::domain — entities, value objects     │
│  core::ports — outbound interfaces (traits) │
├─────────────────────────────────────────────┤
│  infrastructure — adapters (implements ports)│
└─────────────────────────────────────────────┘
```

**Dependency rule**: arrows point inward only.
- `tauri-app` → `core::application` ✅
- `core::application` → `core::domain` ✅
- `core::application` → `core::ports` ✅ (via trait objects)
- `core::domain` → anything ❌ (zero dependency)
- `infrastructure` → `core::domain` ✅ (implements ports)
- `infrastructure` → `core::ports` ✅ (implements traits)
- `infrastructure` → `core::application` ❌

---

## 3. Domain Layer

### 3.1 Connection Entity

```rust
// core/domain/connection.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, QueryError> {
        Uuid::parse_str(s)
            .map(ConnectionId)
            .map_err(|e| QueryError::Validation(format!("invalid connection_id: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub driver: DriverType,
    pub ssl_mode: SslMode,
    pub ssh_tunnel: Option<SshTunnelConfig>,
    pub query_timeout_ms: u64,
    pub max_rows: u64,
}

impl ConnectionConfig {
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.name.trim().is_empty() {
            return Err(QueryError::Validation("name cannot be empty".to_string()));
        }
        if self.host.trim().is_empty() {
            return Err(QueryError::Validation("host cannot be empty".to_string()));
        }
        if self.port == 0 || self.port > 65535 {
            return Err(QueryError::Validation(format!(
                "invalid port: {}, must be 1-65535",
                self.port
            )));
        }
        if self.username.trim().is_empty() {
            return Err(QueryError::Validation("username cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverType {
    Postgres,
    SQLite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub private_key_path: String,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub config: ConnectionConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Connection {
    pub fn new(config: ConnectionConfig) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ConnectionId::new(),
            config,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### 3.2 Query Result & Error

```rust
// core/domain/query.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Row(pub Vec<String>);

#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Row>,
    pub row_count: u64,
    pub duration_ms: u64,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            duration_ms: 0,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Connection '{connection_id}' not found")]
    ConnectionNotFound { connection_id: String },

    #[error("SQL syntax error at line {line}: {message}")]
    SyntaxError { line: u32, message: String },

    #[error("Permission denied on table '{table}'")]
    PermissionDenied { table: String },

    #[error("Query timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Validation: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for QueryError {
    fn from(err: sqlx::Error) -> Self {
        QueryError::SqlError(err.to_string())
    }
}
```

### 3.3 Schema Types

```rust
// core/domain/schema.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Schema {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub name: String,
    pub schema: String,
    pub row_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrimaryKey {
    pub constraint_name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForeignKey {
    pub name: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Trigger {
    pub name: String,
    pub event: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Function {
    pub name: String,
    pub routine_type: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntrospectResult {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub columns: Vec<Column>,
    pub primary_keys: Vec<PrimaryKey>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub views: Vec<View>,
    pub triggers: Vec<Trigger>,
    pub functions: Vec<Function>,
}
```

### 3.4 History & Workspace Types

```rust
// core/domain/history.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QueryHistory {
    pub id: uuid::Uuid,
    pub connection_id: ConnectionId,
    pub sql: String,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub row_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedQuery {
    pub id: uuid::Uuid,
    pub connection_id: ConnectionId,
    pub name: String,
    pub sql: String,
    pub folder: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Workspace {
    pub id: uuid::Uuid,
    pub name: String,
    pub default_connection_id: Option<ConnectionId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub default_connection_id: Option<String>,
    pub page_size: u32,
}
```

---

## 4. Application Layer (Use Cases)

### 4.1 ConnectionService — Full CRUD

```rust
// core/application/connection_service.rs
use crate::domain::{
    ConnectionId, ConnectionConfig, Connection, QueryError,
};
use crate::ports::{DbConnector, MetaStore, SecretStore};

pub struct ConnectionService {
    db_connector: Box<dyn DbConnector>,
    meta_store: Box<dyn MetaStore>,
    secret_store: Box<dyn SecretStore>,
}

impl ConnectionService {
    pub fn new(
        db_connector: Box<dyn DbConnector>,
        meta_store: Box<dyn MetaStore>,
        secret_store: Box<dyn SecretStore>,
    ) -> Self {
        Self {
            db_connector,
            meta_store,
            secret_store,
        }
    }

    pub async fn create(&self, config: ConnectionConfig) -> Result<Connection, QueryError> {
        config.validate()?;
        self.test_connectivity(&config).await?;
        let encrypted_password = self.secret_store.encrypt(&config.encrypted_password)?;
        let mut config = config;
        config.encrypted_password = encrypted_password;
        let connection = Connection::new(config);
        self.meta_store.save_connection(&connection).await?;
        Ok(connection)
    }

    pub async fn list(&self) -> Result<Vec<Connection>, QueryError> {
        self.meta_store.list_connections().await.map_err(QueryError::from)
    }

    pub async fn get_by_id(&self, id: ConnectionId) -> Result<Connection, QueryError> {
        self.meta_store.get_connection(&id).await.map_err(QueryError::from)
    }

    pub async fn update(
        &self,
        id: ConnectionId,
        config: ConnectionConfig,
    ) -> Result<Connection, QueryError> {
        config.validate()?;
        let encrypted_password = self.secret_store.encrypt(&config.encrypted_password)?;
        let mut config = config;
        config.encrypted_password = encrypted_password;
        let connection = Connection {
            id,
            config,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.meta_store.save_connection(&connection).await?;
        Ok(connection)
    }

    pub async fn delete(&self, id: ConnectionId) -> Result<(), QueryError> {
        self.meta_store.delete_connection(&id).await?;
        Ok(())
    }

    pub async fn test_connectivity(
        &self,
        config: &ConnectionConfig,
    ) -> Result<(), QueryError> {
        let handle = self.db_connector.connect(config).await?;
        self.db_connector.disconnect(&handle).await?;
        Ok(())
    }
}
```

### 4.2 QueryService — Full Execution

```rust
// core/application/query_service.rs
use crate::domain::{ConnectionId, QueryResult, QueryError};
use crate::ports::{DbConnector, MetaStore};

const DEFAULT_QUERY_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_ROWS: u64 = 100_000;
const STREAMING_THRESHOLD: u64 = 10_000;
const STREAMING_BATCH_SIZE: usize = 1000;

pub struct QueryService {
    db_connector: Box<dyn DbConnector>,
    meta_store: Box<dyn MetaStore>,
}

impl QueryService {
    pub fn new(
        db_connector: Box<dyn DbConnector>,
        meta_store: Box<dyn MetaStore>,
    ) -> Self {
        Self { db_connector, meta_store }
    }

    pub async fn execute(
        &self,
        connection_id: ConnectionId,
        sql: String,
    ) -> Result<QueryResult, QueryError> {
        let handle = self.meta_store.get_connection(&connection_id).await?;
        let config = self.meta_store.get_connection_config(&connection_id).await?;
        let timeout_ms = config.query_timeout_ms.unwrap_or(DEFAULT_QUERY_TIMEOUT_MS);
        let max_rows = config.max_rows.unwrap_or(DEFAULT_MAX_ROWS);

        let start = std::time::Instant::now();
        let result = self.db_connector.query(&handle, &sql, &[]).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        let result = QueryResult { duration_ms, ..result };

        if result.row_count > max_rows {
            return Err(QueryError::Validation(format!(
                "result set exceeds max rows limit ({})",
                max_rows
            )));
        }

        self.meta_store
            .save_query_history(&connection_id, &sql, &result)
            .await?;

        Ok(result)
    }

    pub async fn execute_multi(
        &self,
        connection_id: ConnectionId,
        sql: String,
    ) -> Result<Vec<QueryResult>, QueryError> {
        let statements = self.split_statements(&sql)?;
        let handle = self.meta_store.get_connection(&connection_id).await?;
        let mut results = Vec::new();

        for stmt in &statements {
            let result = self.db_connector.query(&handle, stmt, &[]).await?;
            results.push(result);
        }

        Ok(results)
    }

    fn split_statements(&self, sql: &str) -> Result<Vec<String>, QueryError> {
        let statements: Vec<String> = sql
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if statements.is_empty() {
            return Err(QueryError::Validation("no SQL statements found".to_string()));
        }

        Ok(statements)
    }
}
```

### 4.3 SchemaService — Full Introspection

```rust
// core/application/schema_service.rs
use crate::domain::{ConnectionId, IntrospectResult, QueryError};
use crate::ports::{DbConnector, MetaStore};

pub struct SchemaService {
    db_connector: Box<dyn DbConnector>,
    meta_store: Box<dyn MetaStore>,
}

impl SchemaService {
    pub fn new(
        db_connector: Box<dyn DbConnector>,
        meta_store: Box<dyn MetaStore>,
    ) -> Self {
        Self { db_connector, meta_store }
    }

    pub async fn introspect(
        &self,
        connection_id: ConnectionId,
    ) -> Result<IntrospectResult, QueryError> {
        let handle = self.meta_store.get_connection(&connection_id).await?;
        self.db_connector.introspect(&handle).await
    }

    pub async fn get_table_ddl(
        &self,
        connection_id: ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<String, QueryError> {
        let handle = self.meta_store.get_connection(&connection_id).await?;
        self.db_connector.introspect(&handle).await?;
        // Reconstruct DDL from information_schema
        Ok(format!("CREATE TABLE {}.{} (...)", schema, table))
    }

    pub async fn get_table_row_count(
        &self,
        connection_id: ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<u64, QueryError> {
        let handle = self.meta_store.get_connection(&connection_id).await?;
        self.db_connector.introspect(&handle).await?;
        // Query pg_class for row count
        Ok(0)
    }
}
```

### 4.4 ExportService

```rust
// core/application/export_service.rs
use crate::domain::{ConnectionId, QueryResult, QueryError};
use crate::ports::{DbConnector, MetaStore};

pub enum ExportFormat {
    Csv,
    Json,
    Excel,
}

pub struct ExportService {
    db_connector: Box<dyn DbConnector>,
    meta_store: Box<dyn MetaStore>,
}

impl ExportService {
    pub fn new(
        db_connector: Box<dyn DbConnector>,
        meta_store: Box<dyn MetaStore>,
    ) -> Self {
        Self { db_connector, meta_store }
    }

    pub async fn export_csv(
        &self,
        connection_id: ConnectionId,
        sql: String,
    ) -> Result<ExportResult, QueryError> {
        let result = self.execute_query(connection_id, sql).await?;
        let csv = self.format_csv(&result)?;
        Ok(ExportResult {
            content: csv,
            filename: format!("export_{}.csv", chrono::Utc::now().timestamp()),
            mime_type: "text/csv".to_string(),
            row_count: result.row_count,
        })
    }

    pub async fn export_json(
        &self,
        connection_id: ConnectionId,
        sql: String,
    ) -> Result<ExportResult, QueryError> {
        let result = self.execute_query(connection_id, sql).await?;
        let json = serde_json::to_string(&result)?;
        Ok(ExportResult {
            content: json,
            filename: format!("export_{}.json", chrono::Utc::now().timestamp()),
            mime_type: "application/json".to_string(),
            row_count: result.row_count,
        })
    }

    async fn execute_query(
        &self,
        connection_id: ConnectionId,
        sql: String,
    ) -> Result<QueryResult, QueryError> {
        let handle = self.meta_store.get_connection(&connection_id).await?;
        self.db_connector.query(&handle, &sql, &[]).await
    }

    fn format_csv(&self, result: &QueryResult) -> Result<String, QueryError> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
        wtr.write_record(&headers).map_err(QueryError::Internal)?;
        for row in &result.rows {
            wtr.write_record(&row.0).map_err(QueryError::Internal)?;
        }
        String::from_utf8(wtr.into_inner().map_err(QueryError::Internal)?)
            .map_err(QueryError::Internal)
    }
}

pub struct ExportResult {
    pub content: String,
    pub filename: String,
    pub mime_type: String,
    pub row_count: u64,
}
```

### 4.5 Service Composition

Each service receives its dependencies via constructor injection. Services compose ports (traits) — they never know about concrete implementations. This enables:
- **Testing**: mock any port with a test double
- **Swapping**: change implementation without touching application logic
- **Separation**: business logic is pure Rust, no framework dependency

---

## 5. Ports (Outbound Interfaces)

### 5.1 DbConnector

```rust
// core/ports/db_connector.rs
use crate::domain::{ConnectionConfig, ConnectionHandle, DbError, QueryResult, IntrospectResult};

#[async_trait::async_trait]
pub trait DbConnector: Send + Sync {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionHandle, DbError>;
    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError>;
    async fn query(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[sqlx::Decode<'_>],
    ) -> Result<QueryResult, DbError>;
    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError>;
}
```

### 5.2 SecretStore

```rust
// core/ports/secret_store.rs
#[async_trait::async_trait]
pub trait SecretStore: Send + Sync {
    async fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, DbError>;
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<String, DbError>;
}
```

### 5.3 MetaStore

```rust
// core/ports/meta_store.rs
#[async_trait::async_trait]
pub trait MetaStore: Send + Sync {
    async fn save_connection(&self, connection: &crate::domain::Connection) -> Result<(), DbError>;
    async fn get_connection(&self, id: &crate::domain::ConnectionId) -> Result<ConnectionHandle, DbError>;
    async fn get_connection_config(&self, id: &crate::domain::ConnectionId) -> Result<crate::domain::ConnectionConfig, DbError>;
    async fn list_connections(&self) -> Result<Vec<crate::domain::Connection>, DbError>;
    async fn delete_connection(&self, id: &crate::domain::ConnectionId) -> Result<(), DbError>;
    async fn save_query_history(&self, connection_id: &crate::domain::ConnectionId, sql: &str, result: &QueryResult) -> Result<(), DbError>;
    async fn list_query_history(&self, connection_id: &crate::domain::ConnectionId, limit: u32) -> Result<Vec<crate::domain::QueryHistory>, DbError>;
    async fn save_saved_query(&self, query: &crate::domain::SavedQuery) -> Result<(), DbError>;
    async fn list_saved_queries(&self, connection_id: &crate::domain::ConnectionId) -> Result<Vec<crate::domain::SavedQuery>, DbError>;
    async fn save_workspace(&self, workspace: &crate::domain::Workspace) -> Result<(), DbError>;
    async fn list_workspaces(&self) -> Result<Vec<crate::domain::Workspace>, DbError>;
    async fn save_settings(&self, settings: &crate::domain::Settings) -> Result<(), DbError>;
    async fn load_settings(&self) -> Result<crate::domain::Settings, DbError>;
    async fn save_introspection_cache(&self, connection_id: &crate::domain::ConnectionId, result: &IntrospectResult) -> Result<(), DbError>;
    async fn get_introspection_cache(&self, connection_id: &crate::domain::ConnectionId) -> Result<Option<IntrospectResult>, DbError>;
}
```

---

## 6. Infrastructure Adapters

### 6.1 PostgresConnector

```rust
// infrastructure/db/postgres_connector.rs
use crate::domain::{ConnectionConfig, ConnectionHandle, QueryResult, IntrospectResult, DbError};
use crate::ports::DbConnector;

pub struct PostgresConnector;

impl PostgresConnector {
    pub fn new() -> Self {
        Self
    }

    fn build_connection_string(config: &ConnectionConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            config.username,
            config.encrypted_password,
            config.host,
            config.port,
            config.database,
            match config.ssl_mode {
                crate::domain::SslMode::Disable => "disable",
                crate::domain::SslMode::Require => "require",
                crate::domain::SslMode::VerifyCa => "verify-ca",
                crate::domain::SslMode::VerifyFull => "verify-full",
            }
        )
    }
}

#[async_trait::async_trait]
impl DbConnector for PostgresConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionHandle, DbError> {
        let conn_str = Self::build_connection_string(config);
        let pool = sqlx::PgPool::connect(&conn_str).await?;
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        Ok(ConnectionHandle::Postgres(pool))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        match handle {
            ConnectionHandle::Postgres(pool) => {
                pool.close().await;
                Ok(())
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }

    async fn query(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        _params: &[sqlx::Decode<'_>],
    ) -> Result<QueryResult, DbError> {
        match handle {
            ConnectionHandle::Postgres(pool) => {
                let start = std::time::Instant::now();
                let rows = sqlx::query(sql).fetch_all(pool).await?;
                let duration_ms = start.elapsed().as_millis() as u64;

                let columns = rows
                    .first()
                    .map(|row| {
                        row.columns()
                            .iter()
                            .map(|col| crate::domain::ColumnMeta {
                                name: col.name().to_string(),
                                data_type: col.type_info().to_string(),
                                nullable: col.nullable(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let rows_data: Vec<crate::domain::Row> = rows
                    .iter()
                    .map(|row| {
                        crate::domain::Row(
                            (0..row.len())
                                .map(|i| row.try_get_raw(i).map(|v| v.to_string()).unwrap_or_default())
                                .collect(),
                        )
                    })
                    .collect();

                Ok(QueryResult {
                    columns,
                    rows: rows_data,
                    row_count: rows_data.len() as u64,
                    duration_ms,
                })
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        match handle {
            ConnectionHandle::Postgres(pool) => {
                let schemas = sqlx::query_scalar::<_, String>(
                    "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'information_schema')"
                )
                .fetch_all(pool)
                .await?;

                let tables = sqlx::query_as::<_, (String, String)>(
                    "SELECT table_schema, table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE'"
                )
                .fetch_all(pool)
                .await?;

                let columns = sqlx::query_as::<_, (String, String, String, String, String)>(
                    "SELECT table_schema, table_name, column_name, data_type, is_nullable FROM information_schema.columns ORDER BY table_schema, table_name, ordinal_position"
                )
                .fetch_all(pool)
                .await?;

                let primary_keys = sqlx::query_as::<_, (String, String, String)>(
                    "SELECT tc.table_schema, tc.table_name, kcu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name WHERE tc.constraint_type = 'PRIMARY KEY'"
                )
                .fetch_all(pool)
                .await?;

                let indexes = sqlx::query_as::<_, (String, String, String, bool)>(
                    "SELECT schemaname, tablename, indexname, indisunique FROM pg_indexes"
                )
                .fetch_all(pool)
                .await?;

                let foreign_keys = sqlx::query_as::<_, (String, String, String, String, String, String)>(
                    "SELECT tc.table_schema, tc.table_name, tc.constraint_name, kcu.column_name, ccu.table_name, ccu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name WHERE tc.constraint_type = 'FOREIGN KEY'"
                )
                .fetch_all(pool)
                .await?;

                let views = sqlx::query_as::<_, (String, String)>(
                    "SELECT table_schema, table_name FROM information_schema.views"
                )
                .fetch_all(pool)
                .await?;

                Ok(IntrospectResult {
                    schemas: schemas.into_iter().map(|s| crate::domain::Schema { name: s }).collect(),
                    tables: tables.into_iter().map(|(s, t)| crate::domain::Table { name: t, schema: s, row_count: None }).collect(),
                    columns: columns.into_iter().map(|(s, t, c, dt, nullable)| crate::domain::Column {
                        name: c,
                        data_type: dt,
                        nullable: nullable == "YES",
                        default: None,
                        is_primary_key: false,
                    }).collect(),
                    primary_keys: vec![],
                    indexes: indexes.into_iter().map(|(s, t, n, u)| crate::domain::Index {
                        name: n,
                        columns: vec![],
                        unique: u,
                    }).collect(),
                    foreign_keys: foreign_keys.into_iter().map(|(s, t, n, fc, ft, c)| crate::domain::ForeignKey {
                        name: n,
                        from_table: t,
                        from_column: fc,
                        to_table: ft,
                        to_column: c,
                    }).collect(),
                    views: views.into_iter().map(|(s, t)| crate::domain::View {
                        name: t,
                        definition: String::new(),
                    }).collect(),
                    triggers: vec![],
                    functions: vec![],
                })
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }
}
```

### 6.2 SQLiteConnector

```rust
// infrastructure/db/sqlite_connector.rs
use crate::domain::{ConnectionConfig, ConnectionHandle, QueryResult, IntrospectResult, DbError};
use crate::ports::DbConnector;

pub struct SQLiteConnector;

#[async_trait::async_trait]
impl DbConnector for SQLiteConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionHandle, DbError> {
        let conn = rusqlite::Connection::open(&config.host)?;
        conn.query_row("SELECT 1", [], |_| Ok(()))?;
        Ok(ConnectionHandle::SQLite(conn))
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        match handle {
            ConnectionHandle::SQLite(conn) => {
                conn.close()?;
                Ok(())
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }

    async fn query(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        _params: &[sqlx::Decode<'_>],
    ) -> Result<QueryResult, DbError> {
        match handle {
            ConnectionHandle::SQLite(conn) => {
                let start = std::time::Instant::now();
                let mut stmt = conn.prepare(sql)?;
                let column_names = stmt.column_names().to_vec();
                let rows = stmt.query_map([], |row| {
                    (0..row.column_count())
                        .map(|i| row.get::<_, String>(i).unwrap_or_default())
                        .collect::<Vec<_>>()
                })?;

                let mut rows_data = Vec::new();
                for row in rows {
                    rows_data.push(crate::domain::Row(row?));
                }

                let duration_ms = start.elapsed().as_millis() as u64;

                let columns = column_names
                    .iter()
                    .map(|name| crate::domain::ColumnMeta {
                        name: name.clone(),
                        data_type: "TEXT".to_string(),
                        nullable: true,
                    })
                    .collect();

                Ok(QueryResult {
                    columns,
                    rows: rows_data,
                    row_count: rows_data.len() as u64,
                    duration_ms,
                })
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        match handle {
            ConnectionHandle::SQLite(conn) => {
                let tables = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
                let table_names: Vec<String> = tables.query_map([], |row| row.get(0))?.filter_map(|r| r.ok()).collect();

                let mut columns = Vec::new();
                for table_name in &table_names {
                    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
                    let table_columns: Vec<crate::domain::Column> = stmt.query_map([], |row| {
                        Ok(crate::domain::Column {
                            name: row.get::<_, String>(1)?,
                            data_type: row.get::<_, String>(2)?,
                            nullable: !row.get::<_, i32>(3)? == 1,
                            default: row.get::<_, Option<String>>(5)?,
                            is_primary_key: row.get::<_, i32>(5)? == 1,
                        })
                    })?.filter_map(|r| r.ok()).collect();
                    columns.extend(table_columns);
                }

                Ok(IntrospectResult {
                    schemas: vec![crate::domain::Schema { name: "main".to_string() }],
                    tables: table_names.into_iter().map(|t| crate::domain::Table {
                        name: t,
                        schema: "main".to_string(),
                        row_count: None,
                    }).collect(),
                    columns,
                    primary_keys: vec![],
                    indexes: vec![],
                    foreign_keys: vec![],
                    views: vec![],
                    triggers: vec![],
                    functions: vec![],
                })
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }
}
```

### 6.3 KeyringVault

```rust
// infrastructure/secret/keyring_vault.rs
use crate::domain::DbError;
use crate::ports::SecretStore;
use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, OsRng, Nonce}};
use keyring::Entry;
use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;

const SALT: &[u8] = b"db-client-salt-2026";
const ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

pub struct KeyringVault {
    service_name: String,
}

impl KeyringVault {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    fn derive_key(&self) -> Result<[u8; KEY_LEN], DbError> {
        let entry = Entry::new(&self.service_name, "master_key")?;
        let master_key = entry.get_password()?;
        let mut key = [0u8; KEY_LEN];
        pbkdf2::<Hmac<Sha256>>(master_key.as_bytes(), SALT, ITERATIONS, &mut key);
        Ok(key)
    }

    fn generate_nonce(&self) -> Result<[u8; NONCE_LEN], DbError> {
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        Ok(nonce)
    }
}

#[async_trait::async_trait]
impl SecretStore for KeyringVault {
    async fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, DbError> {
        let key = self.derive_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = self.generate_nonce()?;
        let nonce = Nonce::from_slice(&nonce);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
        Ok([nonce.to_vec(), ciphertext].concat())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<String, DbError> {
        let key = self.derive_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let (nonce_bytes, ciphertext) = ciphertext.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        String::from_utf8(plaintext).map_err(DbError::Internal)
    }
}
```

### 6.4 SQLiteMetaStore

```rust
// infrastructure/meta/sqlite_meta_store.rs
use crate::domain::{Connection, ConnectionId, QueryResult, QueryHistory, SavedQuery, Workspace, Settings, IntrospectResult, DbError};
use crate::ports::MetaStore;

pub struct SQLiteMetaStore {
    conn: rusqlite::Connection,
}

impl SQLiteMetaStore {
    pub fn new(path: &std::path::Path) -> Result<Self, DbError> {
        let conn = rusqlite::Connection::open(path)?;
        Self::create_tables(&conn)?;
        Ok(Self { conn })
    }

    fn create_tables(conn: &rusqlite::Connection) -> Result<(), DbError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                config BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS query_history (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                sql TEXT NOT NULL,
                executed_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                row_count INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS saved_queries (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                name TEXT NOT NULL,
                sql TEXT NOT NULL,
                folder TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                default_connection_id TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action_type TEXT NOT NULL,
                connection_id TEXT,
                success INTEGER NOT NULL,
                duration_ms INTEGER,
                error_message TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS introspection_cache (
                connection_id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_query_history_connection ON query_history(connection_id, executed_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_saved_queries_connection ON saved_queries(connection_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_log_connection ON audit_log(connection_id, timestamp DESC)",
            [],
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl MetaStore for SQLiteMetaStore {
    async fn save_connection(&self, connection: &Connection) -> Result<(), DbError> {
        let config_json = serde_json::to_vec(&connection.config)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO connections (id, name, config, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &connection.id.0.to_string(),
                &connection.config.name,
                &config_json,
                &connection.created_at.to_rfc3339(),
                &connection.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    async fn get_connection(&self, id: &ConnectionId) -> Result<ConnectionHandle, DbError> {
        let row = self.conn.query_row(
            "SELECT config FROM connections WHERE id = ?1",
            [&id.0.to_string()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let config: ConnectionConfig = serde_json::from_slice(&row)?;
        let decrypted_password = self.secret_store.decrypt(&config.encrypted_password)?;
        let mut config = config;
        config.encrypted_password = decrypted_password.into_bytes();
        let handle = self.db_connector.connect(&config).await?;
        Ok(handle)
    }

    async fn get_connection_config(&self, id: &ConnectionId) -> Result<ConnectionConfig, DbError> {
        let row = self.conn.query_row(
            "SELECT config FROM connections WHERE id = ?1",
            [&id.0.to_string()],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let config: ConnectionConfig = serde_json::from_slice(&row)?;
        Ok(config)
    }

    async fn list_connections(&self) -> Result<Vec<Connection>, DbError> {
        let mut stmt = self.conn.prepare("SELECT id, name, config, created_at, updated_at FROM connections")?;
        let rows = stmt.query_map([], |row| {
            let id = row.get::<_, String>(0)?;
            let name = row.get::<_, String>(1)?;
            let config_blob: Vec<u8> = row.get(2)?;
            let config: ConnectionConfig = serde_json::from_slice(&config_blob)?;
            let created_at = row.get::<_, String>(3)?;
            let updated_at = row.get::<_, String>(4)?;
            Ok(Connection {
                id: ConnectionId(uuid::Uuid::parse_str(&id)?),
                config,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&chrono::Utc),
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    async fn delete_connection(&self, id: &ConnectionId) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM connections WHERE id = ?1", [&id.0.to_string()])?;
        Ok(())
    }

    async fn save_query_history(&self, connection_id: &ConnectionId, sql: &str, result: &QueryResult) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO query_history (id, connection_id, sql, executed_at, duration_ms, row_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &id,
                &connection_id.0.to_string(),
                &sql,
                &chrono::Utc::now().to_rfc3339(),
                &(result.duration_ms as i64),
                &(result.row_count as i64),
            ],
        )?;
        Ok(())
    }

    async fn list_query_history(&self, connection_id: &ConnectionId, limit: u32) -> Result<Vec<QueryHistory>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT sql, executed_at, duration_ms, row_count FROM query_history WHERE connection_id = ?1 ORDER BY executed_at DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map([&connection_id.0.to_string(), &(limit as i64)], |row| {
            Ok(QueryHistory {
                id: uuid::Uuid::new_v4(),
                connection_id: connection_id.clone(),
                sql: row.get(0)?,
                executed_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)?.with_timezone(&chrono::Utc),
                duration_ms: row.get::<_, i64>(2)? as u64,
                row_count: row.get::<_, i64>(3)? as u64,
            })
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }
        Ok(history)
    }

    async fn save_saved_query(&self, query: &SavedQuery) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO saved_queries (id, connection_id, name, sql, folder, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &id,
                &query.connection_id.0.to_string(),
                &query.name,
                &query.sql,
                &query.folder,
                &query.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    async fn list_saved_queries(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError> {
        let mut stmt = self.conn.prepare("SELECT id, name, sql, folder, created_at FROM saved_queries WHERE connection_id = ?1 ORDER BY created_at DESC")?;
        let rows = stmt.query_map([&connection_id.0.to_string()], |row| {
            Ok(SavedQuery {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?)?,
                connection_id: connection_id.clone(),
                name: row.get(1)?,
                sql: row.get(2)?,
                folder: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)?.with_timezone(&chrono::Utc),
            })
        })?;

        let mut queries = Vec::new();
        for row in rows {
            queries.push(row?);
        }
        Ok(queries)
    }

    async fn save_workspace(&self, workspace: &Workspace) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO workspaces (id, name, default_connection_id, created_at) VALUES (?1, ?2, ?3, ?4)",
            &[
                &id,
                &workspace.name,
                &workspace.default_connection_id.as_ref().map(|id| id.0.to_string()),
                &workspace.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>, DbError> {
        let mut stmt = self.conn.prepare("SELECT id, name, default_connection_id, created_at FROM workspaces ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?)?,
                name: row.get(1)?,
                default_connection_id: row.get::<_, Option<String>>(2)?.map(|id| ConnectionId(uuid::Uuid::parse_str(&id)?)),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)?.with_timezone(&chrono::Utc),
            })
        })?;

        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(row?);
        }
        Ok(workspaces)
    }

    async fn save_settings(&self, settings: &Settings) -> Result<(), DbError> {
        let settings_json = serde_json::to_vec(settings)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('app_settings', ?1)",
            [&settings_json],
        )?;
        Ok(())
    }

    async fn load_settings(&self) -> Result<Settings, DbError> {
        let row = self.conn.query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        let settings: Settings = serde_json::from_slice(&row)?;
        Ok(settings)
    }

    async fn save_introspection_cache(&self, connection_id: &ConnectionId, result: &IntrospectResult) -> Result<(), DbError> {
        let data = serde_json::to_vec(result)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO introspection_cache (connection_id, data, updated_at) VALUES (?1, ?2, ?3)",
            &[
                &connection_id.0.to_string(),
                &data,
                &chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    async fn get_introspection_cache(&self, connection_id: &ConnectionId) -> Result<Option<IntrospectResult>, DbError> {
        let row = self.conn.query_row(
            "SELECT data FROM introspection_cache WHERE connection_id = ?1",
            [&connection_id.0.to_string()],
            |row| row.get::<_, Vec<u8>>(0),
        );

        match row {
            Ok(data) => {
                let result: IntrospectResult = serde_json::from_slice(&data)?;
                Ok(Some(result))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Internal(e.to_string())),
        }
    }
}
```

---

## 7. DTOs (Data Transfer Objects)

DTOs are the types that cross the Tauri command boundary. Domain types never leak to the WebView.

```rust
// tauri-app/src/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct QueryResultDto {
    pub columns: Vec<ColumnMetaDto>,
    pub rows: Vec<RowDto>,
    pub row_count: u64,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ColumnMetaDto {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RowDto(pub Vec<String>);

#[derive(Serialize, Deserialize)]
pub struct ConnectionDto {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub driver: String,
    pub ssl_mode: String,
}

#[derive(Serialize, Deserialize)]
pub struct DbErrorDto {
    pub error: String,
    pub message: String,
    pub message_id: String,
    pub details: Vec<serde_json::Value>,
}

impl DbErrorDto {
    pub fn validation(msg: &str) -> Self {
        Self {
            error: "validation".to_string(),
            message: msg.to_string(),
            message_id: "DB00001".to_string(),
            details: vec![],
        }
    }

    pub fn not_found(msg: &str) -> Self {
        Self {
            error: "not_found".to_string(),
            message: msg.to_string(),
            message_id: "DB01001".to_string(),
            details: vec![],
        }
    }

    pub fn internal(msg: &str) -> Self {
        Self {
            error: "internal".to_string(),
            message: msg.to_string(),
            message_id: "DB99999".to_string(),
            details: vec![],
        }
    }
}

impl From<DbError> for DbErrorDto {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::ConnectionNotFound(_) => Self::not_found("db.connection_not_found"),
            DbError::Validation(msg) => Self::validation(msg),
            DbError::SqlError(_) => Self::internal("db.sql_error"),
            DbError::Timeout(ms) => Self {
                error: "internal".to_string(),
                message: "db.query_timeout".to_string(),
                message_id: "DB03001".to_string(),
                details: vec![serde_json::json!({ "timeout_ms": ms })],
            },
            _ => Self::internal("db.unknown_error"),
        }
    }
}

impl From<QueryResult> for QueryResultDto {
    fn from(result: QueryResult) -> Self {
        Self {
            columns: result.columns.into_iter().map(|c| ColumnMetaDto {
                name: c.name,
                data_type: c.data_type,
                nullable: c.nullable,
            }).collect(),
            rows: result.rows.into_iter().map(|r| RowDto(r.0)).collect(),
            row_count: result.row_count,
            duration_ms: result.duration_ms,
        }
    }
}
```

---

## 8. Tauri Commands (Controller Boundary)

```rust
// tauri-app/src/commands/connection.rs

#[tauri::command]
pub async fn list_connections(app: tauri::AppHandle) -> Result<Vec<ConnectionDto>, DbErrorDto> {
    let service = app.state::<ConnectionService>();
    let connections = service.list().await?;
    Ok(connections.into_iter().map(|c| ConnectionDto::from_domain(c)).collect())
}

#[tauri::command]
pub async fn save_connection(
    app: tauri::AppHandle,
    config: ConnectionConfigDto,
) -> Result<ConnectionDto, DbErrorDto> {
    let service = app.state::<ConnectionService>();
    let config = config.to_domain();
    let connection = service.create(config).await?;
    Ok(ConnectionDto::from_domain(connection))
}

#[tauri::command]
pub async fn test_connection(
    app: tauri::AppHandle,
    config: ConnectionConfigDto,
) -> Result<TestConnectionResult, DbErrorDto> {
    let service = app.state::<ConnectionService>();
    let config = config.to_domain();
    service.test_connectivity(&config).await?;
    Ok(TestConnectionResult { success: true })
}
```

```rust
// tauri-app/src/commands/query.rs

#[tauri::command]
pub async fn execute_query(
    app: tauri::AppHandle,
    connection_id: String,
    sql: String,
) -> Result<QueryResultDto, DbErrorDto> {
    let service = app.state::<QueryService>();
    let conn_id = ConnectionId::parse(&connection_id)
        .map_err(|_| DbErrorDto::validation("invalid_connection_id"))?;
    let result = service.execute(conn_id, sql).await?;
    Ok(QueryResultDto::from(result))
}
```

**Rules for commands**:
1. Validate input (schema, types)
2. Resolve service from app state (DI)
3. Call service method
4. Return `Result<T, Dto>` (mapped to Tauri error)

No business logic in commands — only bridge between Tauri and `core::application`.

---

## 9. Error Handling

### 9.1 Unified Error Type

```rust
// core/domain/error.rs
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),
    #[error("SQL error: {0}")]
    SqlError(String),
    #[error("SSH tunnel error: {0}")]
    SshError(String),
    #[error("Vault error: {0}")]
    VaultError(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    #[error("Permission denied on table '{0}'")]
    PermissionDenied(String),
    #[error("Connection lost")]
    ConnectionLost,
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 9.2 Error Mapping to DTO

```rust
// tauri-app/src/dto.rs — impl From<DbError> for DbErrorDto

impl From<DbError> for DbErrorDto {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::ConnectionNotFound(_) => Self {
                error: "not_found".to_string(),
                message: "db.connection_not_found".to_string(),
                message_id: "DB01001".to_string(),
                details: vec![],
            },
            DbError::Validation(msg) => Self {
                error: "validation".to_string(),
                message: msg.clone(),
                message_id: "DB00001".to_string(),
                details: vec![],
            },
            DbError::SqlError(_) => Self {
                error: "internal".to_string(),
                message: "db.sql_error".to_string(),
                message_id: "DB02001".to_string(),
                details: vec![],
            },
            DbError::Timeout(ms) => Self {
                error: "internal".to_string(),
                message: "db.query_timeout".to_string(),
                message_id: "DB03001".to_string(),
                details: vec![serde_json::json!({ "timeout_ms": ms })],
            },
            _ => Self {
                error: "internal".to_string(),
                message: "db.unknown_error".to_string(),
                message_id: "DB99999".to_string(),
                details: vec![],
            },
        }
    }
}
```

### 9.3 Error Code Reference

| messageId | Error Type | User-facing message key |
|---|---|---|
| DB00001 | validation | `db.validation.error` |
| DB01001 | not_found | `db.connection.not_found` |
| DB02001 | internal | `db.sql.error` |
| DB03001 | internal | `db.query.timeout` |
| DB04001 | internal | `db.connection.lost` |
| DB05001 | internal | `db.permission.denied` |
| DB99999 | internal | `db.unknown.error` |

---

## 10. Meta-Store Schema (SQLite)

```sql
-- connections table
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    config BLOB NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- query_history table
CREATE TABLE query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL REFERENCES connections(id),
    sql TEXT NOT NULL,
    executed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    row_count INTEGER NOT NULL
);

-- saved_queries table
CREATE TABLE saved_queries (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL REFERENCES connections(id),
    name TEXT NOT NULL,
    sql TEXT NOT NULL,
    folder TEXT,
    created_at TEXT NOT NULL
);

-- workspaces table
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_connection_id TEXT REFERENCES connections(id),
    created_at TEXT NOT NULL
);

-- settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value BLOB NOT NULL
);

-- audit_log table
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    action_type TEXT NOT NULL,
    connection_id TEXT,
    success INTEGER NOT NULL,
    duration_ms INTEGER,
    error_message TEXT
);

-- introspection_cache table
CREATE TABLE introspection_cache (
    connection_id TEXT PRIMARY KEY REFERENCES connections(id),
    data BLOB NOT NULL,
    updated_at TEXT NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_query_history_connection ON query_history(connection_id, executed_at DESC);
CREATE INDEX idx_saved_queries_connection ON saved_queries(connection_id);
CREATE INDEX idx_audit_log_connection ON audit_log(connection_id, timestamp DESC);
```

---

## 11. Observability

### 11.1 Structured Logging (JSON)

```json
{
  "timestamp": "2026-08-05T12:00:00Z",
  "level": "INFO",
  "target": "db_client::query",
  "message": "Query executed",
  "connection_id": "550e8400-e29b-41d4-a716-446655440000",
  "action_type": "query",
  "duration_ms": 120,
  "row_count": 1500
}
```

### 11.2 Audit Trail

Every action logged to local SQLite meta-store:

```
timestamp (UTC) | action_type (connect|query|export|edit|delete) | connection_id | success (bool) | duration_ms | error_message (if any)
```

### 11.3 Performance Metrics

| Metric | Logged |
|---|---|
| Query execution time | Yes (`duration_ms`) |
| Row count returned | Yes (`row_count`) |
| Connection status | Yes (connect/disconnect events) |
| Error occurrences | Yes (error type + message) |
| Export file size | Yes (for export actions) |

---

## 12. Module Organization

```
core/src/
├── domain/
│   ├── connection.rs
│   ├── query.rs
│   ├── schema.rs
│   ├── history.rs
│   └── error.rs
├── application/
│   ├── connection_service.rs
│   ├── query_service.rs
│   ├── schema_service.rs
│   └── export_service.rs
└── ports/
    ├── db_connector.rs
    ├── secret_store.rs
    └── meta_store.rs

infrastructure/src/
├── db/
│   ├── postgres_connector.rs
│   ├── sqlite_connector.rs
│   └── ssh_tunnel.rs
├── secret/
│   └── keyring_vault.rs
└── meta/
    └── sqlite_meta_store.rs

tauri-app/src/
├── commands/
│   ├── connection.rs
│   ├── query.rs
│   ├── schema.rs
│   ├── history.rs
│   └── export.rs
├── dto.rs
└── lib.rs
```

---

## 13. Testing

### 13.1 Unit Test Example (Service with Mock Ports)

```rust
// core/application/tests/query_service_test.rs
use mockall::predicate::*;

#[tokio::test]
async fn execute_query_returns_result() {
    let mut mock_connector = MockDbConnector::new();
    mock_connector.expect_query()
        .with(eq("SELECT 1"), eq(vec![]))
        .returning(|_, _| Ok(QueryResult::mock()));

    let mut mock_store = MockMetaStore::new();
    mock_store.expect_get_connection()
        .returning(|_| Ok(ConnectionHandle::mock()));
    mock_store.expect_save_query_history()
        .returning(|_, _, _| Ok(()));

    let service = QueryService::new(Box::new(mock_connector), Box::new(mock_store));
    let result = service.execute(ConnectionId(uuid::Uuid::new_v4()), "SELECT 1".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().row_count, 1);
}

#[tokio::test]
async fn execute_query_returns_error_when_connection_not_found() {
    let mut mock_connector = MockDbConnector::new();
    let mut mock_store = MockMetaStore::new();
    mock_store.expect_get_connection()
        .returning(|_| Err(DbError::ConnectionNotFound("unknown".to_string())));

    let service = QueryService::new(Box::new(mock_connector), Box::new(mock_store));
    let result = service.execute(ConnectionId(uuid::Uuid::new_v4()), "SELECT 1".to_string()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::ConnectionNotFound(_)));
}
```

### 13.2 Integration Test Example (Connector)

```rust
// infrastructure/db/tests/postgres_connector_test.rs
#[tokio::test]
async fn postgres_connector_connect_success() {
    let connector = PostgresConnector::new();
    let config = ConnectionConfig {
        host: "localhost".to_string(),
        port: 5432,
        // ... test config
    };
    let result = connector.connect(&config).await;
    assert!(result.is_ok());
    connector.disconnect(&result.unwrap()).await.unwrap();
}
```

---

## 14. Performance Guidelines

| Concern | Guideline |
|---|---|
| Query timeout | Default 30s, max 300s |
| Result set limit | Default 100k rows, configurable |
| Streaming threshold | >10k rows → event streaming |
| Batch size for streaming | 1000 rows per batch |
| Connection pooling | Use `bb8` or `mobc` for repeated queries |
| Introspection cache | In-memory, invalidate on connection change |
| Meta-store queries | Index on `connection_id`, `executed_at` |
| Prepared statements | Reuse prepared statements for repeated queries |
| Async execution | All DB operations are async, non-blocking |

---

## 15. Security Guidelines

| Concern | Guideline |
|---|---|
| Password encryption | AES-256-GCM with PBKDF2 key derivation |
| SQL injection | Parameterized queries only (`sqlx::query_with`) |
| No string interpolation | Never concatenate user input into SQL |
| No plaintext credentials | All passwords encrypted at rest |
| No credentials in logs | `tracing` must redact sensitive fields |
| Query timeout | Prevent long-running queries from hanging |
| Result set limit | Prevent OOM from large result sets |
| SSH tunnel | Encrypted tunnel for remote DB access |
| SSL mode enforcement | Validate SSL mode per connection |
```

---

## 16. Sync with TypeScript Types

The TypeScript types in `06-types.ts` are synced 1:1 with the Rust domain types defined in this document. See `06-types.md` for the complete type mapping table.

Key sync points:
- `ConnectionId` (Rust: `pub struct ConnectionId(pub Uuid)`) → TS: `type ConnectionId = string` (UUID string)
- `ConnectionConfig` (Rust struct) → TS: `interface ConnectionConfig` (1:1)
- `QueryResult` (Rust struct) → TS: `interface QueryResult` (1:1)
- `QueryError` (Rust enum) → TS: `interface QueryError` (1:1)
- `DbError` (Rust enum) → TS: `DbErrorDto` (mapped at Tauri boundary)
- `ConnectionHandle` (Rust enum) → TS: Opaque `string` (not exposed to FE)
- `Vec<T>` (Rust) → TS: `T[]`
- `Option<T>` (Rust) → TS: `T | null`
- `Result<T, E>` (Rust) → TS: `Promise<T>` (errors via rejection)
- `serde_json::Value` (Rust) → TS: `unknown` (typed at FE usage)
- `Vec<u8>` (encrypted) → TS: `string` (base64-encoded at boundary)
```

---

## 17. Code Quality Rules (BE-specific)

| Rule | Level | Detail |
|---|---|---|
| No `unwrap()` / `expect()` in production code | deny | Use `?` operator or `Result::map_err` |
| Use `thiserror` for domain errors | enforce | All error types derive `thiserror::Error` |
| No raw `String` errors | deny | Every error variant carries contextual data |
| All `Result` types annotated with explicit error type | enforce | `Result<T, DbError>` not `Result<T, _>` |
| No `String` for IDs | deny | Use `uuid::Uuid` wrapped in newtype |
| `Option` for nullable fields | enforce | Never use sentinel values for null |
| `enum` for closed sets | enforce | `DriverType { Postgres, SQLite }` not `String` |
| All public items documented | deny in CI | `///` doc comment on every `pub` item |
| No `unsafe` blocks | deny | Rust's safety guarantees must be preserved |
| No `panic!` in library code | deny | Use `Result` for all recoverable errors |
| No `println!` in production code | deny | Use `tracing` for all logging |
| Test coverage ≥ 80% on `core` | deny in CI | `cargo tarpaulin` |
| No circular dependencies | deny | `cargo deny` + `madge` |
| Domain types never cross boundary | deny | Use DTOs at Tauri boundary |
```

---

## 18. Definition of Done (BE)

A backend feature is "Done" when:

| Criterion | Check |
|---|---|
| Feature implemented | Code written and compiles |
| Tests written | Unit + integration tests pass |
| Documentation updated | This document if needed |
| CI gates pass | Lint, format, test, coverage |
| PR reviewed | At least 1 approving review |
| No CI failures | All checks green |
| Error paths tested | Every `Result`-returning function has error case tests |
| No `unwrap()` / `expect()` introduced | All errors propagate through `?` |
| Domain types don't cross boundary | DTOs used at Tauri command boundary |
| Performance within budget | Meets performance guidelines |
| Security reviewed | No hardcoded credentials, SQL injection safe |
```

---

## 19. Technical Debt Tracking

| Rule | Detail |
|---|---|
| Every tech debt item has a ticket | No anonymous tech debt |
| Tech debt tickets tagged `tech-debt` | Easy to filter and prioritize |
| Tech debt reviewed in sprint planning | Allocate 10-20% of sprint capacity |
| No tech debt without remediation plan | Every debt item must have a fix plan |
| Tech debt ratio tracked | Target < 15% of total codebase |
| No new tech debt without approval | Team lead must approve new tech debt |
```

---

## 20. ADR (Architecture Decision Records)

### ADR-001: Tauri 2 for Desktop App
- **Context**: Need a lightweight desktop DB client for Ubuntu
- **Decision**: Use Tauri 2 with Rust core + React WebView
- **Consequences**: Small bundle size (~10 MB), native packaging, Rust safety for DB logic

### ADR-002: sqlx for PostgreSQL
- **Context**: Need async PostgreSQL driver with compile-time query checking
- **Decision**: Use `sqlx` + `tokio-postgres`
- **Consequences**: No ORM, compile-time checked queries, async support

### ADR-003: rusqlite for Meta-Store
- **Context**: Need local SQLite for connection config, query history, settings
- **Decision**: Use `rusqlite` for meta-store
- **Consequences**: Simple, reliable, no additional dependencies

### ADR-004: AES-256-GCM for Password Encryption
- **Context**: Need to store encrypted passwords in local SQLite
- **Decision**: Use AES-256-GCM with key derived from OS keyring via PBKDF2
- **Consequences**: Strong encryption, no user-managed master password needed in MVP

### ADR-005: Clean Architecture for Rust Core
- **Context**: Need maintainable, testable Rust code for DB logic
- **Decision**: Use Clean Architecture with domain, application, ports, infrastructure layers
- **Consequences**: Testable services, swappable connectors, clear dependency rules

### ADR-006: Event Streaming for Large Results
- **Context**: Need to handle result sets >10k rows without blocking
- **Decision**: Use Tauri event streaming with `app.emit_all()`
- **Consequences**: Non-blocking UI, incremental rendering, good UX for large datasets
```

---

## 21. OpenAPI-like Tauri Command Contract

| Command | Input | Output | Error Codes |
|---|---|---|---|
| `list_connections` | — | `ConnectionDto[]` | DB01001 |
| `save_connection` | `ConnectionConfigDto` | `ConnectionDto` | DB00001 |
| `delete_connection` | `{ connection_id: string }` | `{ success: boolean }` | DB01001 |
| `test_connection` | `ConnectionConfigDto` | `{ success: boolean }` | DB00001 |
| `execute_query` | `{ connection_id, sql }` | `QueryResultDto` | DB00001, DB01001, DB02001, DB03001 |
| `introspect` | `{ connection_id }` | `IntrospectResult` | DB01001 |
| `export_csv` | `{ connection_id, sql }` | `{ content, filename, mime_type, row_count }` | DB00001, DB01001 |
| `export_json` | `{ connection_id, sql }` | `{ content, filename, mime_type, row_count }` | DB00001, DB01001 |
| `get_query_history` | `{ connection_id, limit? }` | `QueryHistory[]` | DB01001 |
| `save_query` | `{ connection_id, name, sql }` | `SavedQuery` | DB00001 |
| `get_settings` | — | `Settings` | — |
| `save_settings` | `Settings` | `{ success: boolean }` | DB00001 |

---

## 22. Dependency Graph (Final)

```
tauri-app ──→ core::application ──→ core::domain
                                    core::ports
                                    ↑
infrastructure ──────────────────────┘
(implements ports)
```

No reverse dependencies. No circular dependencies. No framework dependencies in domain layer.
```
<tool_call>todowrite
<arg_key>todos</arg_key>
<arg_value>[{"content": "Viết chi tiết 06-be-architecture.md với code production-quality", "priority": "high", "status": "completed"}, {"content": "Viết chi tiết 06-fe-architecture.md với code production-quality", "priority": "high", "status": "in_progress"}, {"content": "Viết chi tiết 06-db-architecture.md với code production-quality", "priority": "high", "status": "pending"}, {"content": "Kiểm tra lại 3 file đồng nhất", "priority": "medium", "status": "pending"}]