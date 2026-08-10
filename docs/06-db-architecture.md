# 06 — DB Client — Database Layer Architecture

---

## 1. Stack

| Concern | Technology | Version | Rationale |
|---|---|---|---|
| Postgres driver | `sqlx` + `tokio-postgres` | 0.7 + 0.7 | Async, compile-time checked queries, no ORM |
| SQLite driver | `rusqlite` | 0.30 | Local meta-store + SQLite file connections |
| SSH tunnel | `openssh` | 0.9 | Spawn binary, no C compile dependency on Ubuntu |
| Secret vault | `keyring` + `aes-gcm` | 0.16 + 0.10 | libsecret on GNOME/KDE; AES-256-GCM at rest |
| Meta-store | `rusqlite` | 0.30 | Connections config, query history, preferences |
| Crypto | `aes-gcm` + `pbkdf2` | 0.10 + 0.12 | AES-256-GCM encryption, PBKDF2 key derivation |
| UUID | `uuid` | 1.x | Type-safe IDs |
| Date/time | `chrono` | 0.4 | UTC timestamps |
| Serialization | `serde` + `serde_json` | 1.x | JSON for Tauri boundary |
| CSV | `csv` | 1.x | Streaming CSV export |
| Excel write | `rust_xlsxwriter` | 0.20+ | Excel export |
| Logging | `tracing` | 0.1 | Structured logs |
| Testing | `#[cfg(test)]` + `mockall` | latest | Unit + integration tests |

---

## 2. Connector Abstraction

### 2.1 Trait Definition

```rust
// core/ports/db_connector.rs
use crate::domain::{ConnectionConfig, ConnectionHandle, DbError, QueryResult, Schema, ExplainPlan};

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
    async fn introspect(&self, handle: &ConnectionHandle) -> Result<Schema, DbError>;
    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<ExplainPlan, DbError>;
}
```

### 2.2 ConnectionHandle

```rust
// core/domain/connection.rs — add ConnectionHandle

pub enum ConnectionHandle {
    Postgres(sqlx::PgPool),
    SQLite(rusqlite::Connection),
}
```

### 2.3 Implementations

| Connector | Crate | Use case |
|---|---|---|
| `PostgresConnector` | `sqlx` + `tokio-postgres` | PostgreSQL (primary) |
| `SQLiteConnector` | `rusqlite` | Local meta-store + SQLite file connections |

Each connector implements `DbConnector` trait. New connectors (MySQL, SQL Server) are added by implementing the trait — no changes to core or application layers.

---

## 3. Schema Introspection

### 3.1 PostgreSQL Introspection Queries

| Step | Query | Output |
|---|---|---|
| 1. List schemas | `SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'information_schema')` | `Vec<Schema>` |
| 2. List tables per schema | `SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE'` | `Vec<Table>` |
| 3. List columns per table | `SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position` | `Vec<Column>` |
| 4. List primary keys | `SELECT tc.constraint_name, kcu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name WHERE tc.table_schema = $1 AND tc.table_name = $2 AND tc.constraint_type = 'PRIMARY KEY'` | `Vec<PrimaryKey>` |
| 5. List indexes | `SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = $1 AND tablename = $2` | `Vec<Index>` |
| 6. List foreign keys | `SELECT tc.constraint_name, kcu.column_name, ccu.table_name AS foreign_table_name, ccu.column_name AS foreign_column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name WHERE tc.table_schema = $1 AND tc.table_name = $2 AND tc.constraint_type = 'FOREIGN KEY'` | `Vec<ForeignKey>` |
| 7. List views | `SELECT table_name, view_definition FROM information_schema.views WHERE table_schema = $1` | `Vec<View>` |
| 8. List triggers | `SELECT trigger_name, event_manipulation, action_statement FROM information_schema.triggers WHERE event_object_schema = $1 AND event_object_table = $2` | `Vec<Trigger>` |
| 9. List functions | `SELECT routine_name, routine_type, data_type FROM information_schema.routines WHERE specific_schema = $1` | `Vec<Function>` |
| 10. DDL for one table | `pg_get_tabledef()` or reconstruct from `information_schema` | `String` (CREATE TABLE DDL) |
| 11. Row count | `SELECT reltuples FROM pg_class WHERE relname = $1` | `u64` |

### 3.2 Introspection Caching

Introspection results are cached in-memory, invalidated on connection change — same as OPASS `Cache-aside` pattern.

```rust
// core/application/schema_service.rs — caching logic

pub async fn introspect(&self, connection_id: ConnectionId) -> Result<IntrospectResult, QueryError> {
    // Check cache first
    if let Some(cached) = self.meta_store.get_introspection_cache(&connection_id).await? {
        if !self.is_cache_stale(&cached) {
            return Ok(cached);
        }
    }

    // Cache miss or stale — introspect from DB
    let handle = self.meta_store.get_connection(&connection_id).await?;
    let result = self.db_connector.introspect(&handle).await?;

    // Cache the result
    self.meta_store.save_introspection_cache(&connection_id, &result).await?;

    Ok(result)
}
```

---

## 4. Query Execution

### 4.1 Executor

- Receives SQL string + params → runs through `DbConnector::query()` → returns `QueryResult { columns, rows, row_count, duration_ms }`
- No string interpolation — always parameterized (`sqlx::query_with`)
- Rust type system catches type mismatches at compile time

### 4.2 Multi-statement

- Split by `;` delimiter
- Execute each statement independently
- Collect results + errors per statement
- Return partial results if some statements succeed and others fail

### 4.3 Transaction

- `BEGIN` / `COMMIT` / `ROLLBACK`
- Each transaction is a `UnitOfWork` scope (same as OPASS pattern)
- Transaction boundary is at the application service layer

### 4.4 EXPLAIN

- Run `EXPLAIN ANALYZE <sql>`
- Parse plan JSON
- Return tree + cost metrics to FE for visualization

### 4.5 Query Timeout

- Default timeout: 30 seconds
- Maximum timeout: 300 seconds
- Configurable per connection
- Timeout enforced at `DbConnector::query()` level

### 4.6 Result Set Limit

- Default limit: 100,000 rows
- Configurable per connection
- Streaming for >10k rows (event-based)
- Direct fetch for ≤10k rows

---

## 5. Streaming Large Results

### 5.1 Streaming Implementation

For result sets > 10k rows:

```rust
// infrastructure/db/postgres_connector.rs — streaming implementation

impl PostgresConnector {
    pub async fn query_stream(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[sqlx::Decode<'_>],
        batch_size: usize,
    ) -> Result<QueryStream, DbError> {
        match handle {
            ConnectionHandle::Postgres(pool) => {
                let stream = sqlx::query(sql)
                    .fetch(pool)
                    .buffered(batch_size);
                Ok(QueryStream::Postgres(stream))
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }
}
```

### 5.2 Streaming Flow

```
Rust core → emit event per batch via app.emit_all("query-result-batch", batch)
         → app.emit_all("query-done", QueryDone { total_rows, duration_ms })
         → app.emit_all("query-error", error)
```

FE subscribes via `tauri::Event` listener → feeds into TanStack Query `useInfiniteQuery` or custom streaming hook → virtualized grid renders incremental rows.

### 5.3 Batch Size

- Default batch size: 1000 rows
- Configurable
- Each batch emitted as separate Tauri event
- FE appends each batch to existing result set

---

## 6. Local Meta-Store (rusqlite)

### 6.1 Data Stored

| Data | Retention | Purpose |
|---|---|---|
| Connection configs (encrypted password) | Persistent | Reconnect without re-entering credentials |
| Query history | Last 500, searchable | Re-run previous queries |
| Saved queries / workspaces | Persistent | User-organized query folders |
| Grid layout preferences | Per table | Column order, width, visibility |
| User preferences | Persistent | Theme, language, default connection |
| Introspection cache | Per connection | Cached schema metadata |
| Feature flags | Persistent | Feature toggle state |
| Audit log | Rotated weekly | Action history |

### 6.2 Meta-Store Schema (SQL)

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

### 6.3 W* (Work Buffer) Concept

The meta-store is the "W*" equivalent — local, transient, user-scoped. It stores everything the user needs to work with the database client, separate from the actual database being queried.

---

## 7. Secret Vault

### 7.1 Encryption Flow

```
1. User enters password in connection form
2. Password → PBKDF2 key derivation (100,000 iterations, SHA-256)
3. AES-256-GCM encryption with derived key
4. Ciphertext stored in meta-store (BLOB)
5. On decrypt: ciphertext → AES-256-GCM decryption → plaintext password
6. Plaintext password used for connection, never stored
```

### 7.2 Key Derivation

| Parameter | Value |
|---|---|
| Algorithm | PBKDF2-SHA256 |
| Iterations | 100,000 |
| Salt | Random 16 bytes (stored with ciphertext) |
| Key length | 256 bits (AES-256) |
| Encryption | AES-256-GCM |
| Nonce length | 12 bytes |

### 7.3 Storage

| Concern | Approach |
|---|---|
| Storage | `keyring` crate → GNOME Keyring / KDE Wallet / libsecret |
| Encryption | AES-256-GCM for password at rest |
| Key derivation | Master key derived from OS keyring (no user-managed master password in MVP) |
| Access | `SecretStore` trait (`encrypt`, `decrypt`, `get`) implemented by `KeyringVault` |

---

## 8. Domain Types (DB Layer)

### 8.1 Query Result & Error

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
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 8.2 Explain Plan

```rust
// core/domain/query.rs — add ExplainPlan

#[derive(Debug, Clone, Serialize)]
pub struct ExplainPlan {
    pub plan_json: serde_json::Value,
    pub total_cost: f64,
    pub planning_time_ms: u64,
    pub execution_time_ms: u64,
}
```

### 8.3 Schema Types

```rust
// core/domain/schema.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Schema { pub name: String }

#[derive(Debug, Clone, Serialize)]
pub struct Table { pub name: String, pub schema: String }

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrimaryKey {
    pub constraint_name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Index { pub name: String, pub columns: Vec<String>, pub unique: bool }

#[derive(Debug, Clone, Serialize)]
pub struct ForeignKey {
    pub name: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct View { pub name: String, pub definition: String }

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
```

### 8.4 History Types

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
```

---

## 9. PostgresConnector — Detailed Implementation

### 9.1 Connection

```rust
// infrastructure/db/postgres_connector.rs

impl PostgresConnector {
    pub async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionHandle, DbError> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            config.username,
            // Password decrypted by caller before passing to connect
            config.encrypted_password,
            config.host,
            config.port,
            config.database,
            match config.ssl_mode {
                SslMode::Disable => "disable",
                SslMode::Require => "require",
                SslMode::VerifyCa => "verify-ca",
                SslMode::VerifyFull => "verify-full",
            }
        );

        let pool = sqlx::PgPool::connect(&connection_string).await?;

        // Test connection with a simple query
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        Ok(ConnectionHandle::Postgres(pool))
    }
}
```

### 9.2 Query Execution

```rust
impl DbConnector for PostgresConnector {
    async fn query(
        &self,
        handle: &ConnectionHandle,
        sql: &str,
        params: &[sqlx::Decode<'_>],
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
                            .map(|col| ColumnMeta {
                                name: col.name().to_string(),
                                data_type: col.type_info().to_string(),
                                nullable: col.nullable(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let rows_data: Vec<Row> = rows
                    .iter()
                    .map(|row| {
                        Row(
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
}
```

### 9.3 Introspection

```rust
impl DbConnector for PostgresConnector {
    async fn introspect(&self, handle: &ConnectionHandle) -> Result<Schema, DbError> {
        match handle {
            ConnectionHandle::Postgres(pool) => {
                // Step 1: List schemas
                let schemas = sqlx::query_scalar::<_, String>(
                    "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'information_schema')"
                )
                .fetch_all(pool)
                .await?;

                // Step 2: List tables
                let tables = sqlx::query_as::<_, (String, String)>(
                    "SELECT table_schema, table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE'"
                )
                .fetch_all(pool)
                .await?;

                // Step 3: List columns (for each table)
                let columns = sqlx::query_as::<_, (String, String, String, String, String)>(
                    "SELECT table_schema, table_name, column_name, data_type, is_nullable FROM information_schema.columns ORDER BY table_schema, table_name, ordinal_position"
                )
                .fetch_all(pool)
                .await?;

                // ... similar for indexes, FKs, views, triggers, functions

                Ok(Schema { schemas, tables, columns, /* ... */ })
            }
            _ => Err(DbError::Validation("wrong connector type".to_string())),
        }
    }
}
```

---

## 10. SQLiteConnector — Detailed Implementation

### 10.1 Connection

```rust
// infrastructure/db/sqlite_connector.rs

impl SQLiteConnector {
    pub async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionHandle, DbError> {
        let conn = rusqlite::Connection::open(&config.host)?;
        // Test connection
        conn.query_row("SELECT 1", [], |_| Ok(()))?;
        Ok(ConnectionHandle::SQLite(conn))
    }
}
```

### 10.2 Query Execution

```rust
impl DbConnector for SQLiteConnector {
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
                    rows_data.push(Row(row?));
                }

                let duration_ms = start.elapsed().as_millis() as u64;

                let columns = column_names
                    .iter()
                    .map(|name| ColumnMeta {
                        name: name.clone(),
                        data_type: "TEXT".to_string(), // SQLite is dynamically typed
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
}
```

---

## 11. SSH Tunnel

### 11.1 Tunnel Configuration

```rust
// infrastructure/db/ssh_tunnel.rs

pub struct OpenSshTunnel {
    config: SshTunnelConfig,
}

impl OpenSshTunnel {
    pub async fn connect(&self) -> Result<TunnelHandle, DbError> {
        let session = openssh::Session::connect(
            format!("{}:{}", self.config.host, self.config.port),
            &self.config.user,
            Some(&self.config.private_key_path),
        )
        .await?;

        // Forward local port to remote DB
        let forward = session
            .request_port_forward("127.0.0.1", 0, &self.config.remote_host, self.config.remote_port)
            .await?;

        Ok(TunnelHandle { session, forward })
    }
}
```

### 11.2 Tunnel Lifecycle

```
1. User enables SSH tunnel in connection editor
2. ConnectionService.connect() checks if SSH tunnel is configured
3. If yes: OpenSshTunnel.connect() → forward local port to remote DB
4. DB connector connects to localhost:forwarded_port instead of remote host
5. On disconnect: tunnel.close() → stop forwarding
```

---

## 12. KeyringVault — Detailed Implementation

### 12.1 Encryption/Decryption

```rust
// infrastructure/secret/keyring_vault.rs

impl KeyringVault {
    fn derive_key(&self) -> Result<[u8; 32], DbError> {
        let entry = Entry::new(&self.service_name, "master_key")?;
        let master_key = entry.get_password()?;

        // PBKDF2 key derivation
        let salt = b"db-client-salt"; // fixed salt for deterministic key
        let mut key = [0u8; 32];
        pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            master_key.as_bytes(),
            salt,
            100_000,
            &mut key,
        );

        Ok(key)
    }

    fn generate_nonce(&self) -> Result<[u8; 12], DbError> {
        let mut nonce = [0u8; 12];
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
        let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;
        Ok([nonce.to_vec(), ciphertext].concat())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<String, DbError> {
        let key = self.derive_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let (nonce_bytes, ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        String::from_utf8(plaintext).map_err(DbError::Internal)
    }
}
```

---

## 13. SQLiteMetaStore — Detailed Implementation

### 13.1 Connection

```rust
// infrastructure/meta/sqlite_meta_store.rs

impl SQLiteMetaStore {
    pub fn new(path: &std::path::Path) -> Result<Self, DbError> {
        let conn = rusqlite::Connection::open(path)?;

        // Create all tables if not exists
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

        // Create other tables...

        Ok(Self { conn })
    }
}
```

### 13.2 Save Connection

```rust
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
        // Decrypt password
        let decrypted_password = self.secret_store.decrypt(&config.encrypted_password)?;
        let mut config = config;
        config.encrypted_password = decrypted_password.into_bytes();

        // Create connection handle
        let handle = self.db_connector.connect(&config).await?;
        Ok(handle)
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
                id: ConnectionId(Uuid::parse_str(&id)?),
                config,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }
}
```

---

## 14. Performance Guidelines (DB Layer)

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
| Error retry | Connection errors trigger auto-reconnect (future) |

---

## 15. Security Guidelines (DB Layer)

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

## 16. Testing (DB Layer)

| Test type | Location | Tool | Coverage |
|---|---|---|---|
| Unit tests | `src/` inline `#[cfg(test)]` | `cargo test` | All public functions |
| Integration tests | `tests/` directory | `cargo test --test` | Trait implementations |
| Mock tests | `src/` with `mockall` | `cargo test` | Service layer with mock ports |
| Connector tests | `infrastructure/db/tests/` | `cargo test --test` | PostgresConnector, SQLiteConnector |
| Vault tests | `infrastructure/secret/tests/` | `cargo test --test` | KeyringVault encrypt/decrypt |
| Meta-store tests | `infrastructure/meta/tests/` | `cargo test --test` | SQLiteMetaStore CRUD |
| Coverage | — | `cargo tarpaulin` | ≥ 80% on `core` |

---

## 17. Module Organization

```
core/src/
├── domain/
│   ├── connection.rs       # ConnectionId, ConnectionConfig, Connection, ConnectionHandle
│   ├── query.rs            # ColumnMeta, Row, QueryResult, QueryError, ExplainPlan
│   ├── schema.rs           # Schema, Table, Column, PrimaryKey, Index, ForeignKey, View, Trigger, Function
│   ├── history.rs          # QueryHistory, SavedQuery, Workspace
│   └── error.rs            # DbError (unified error type)
├── application/
│   ├── connection_service.rs   # Full CRUD for connections
│   ├── query_service.rs        # Execute, multi-statement, transaction
│   ├── schema_service.rs       # Introspection, DDL generation
│   └── export_service.rs       # CSV, JSON, Excel export
└── ports/
    ├── db_connector.rs     # trait DbConnector
    ├── secret_store.rs     # trait SecretStore
    └── meta_store.rs       # trait MetaStore (all CRUD operations)

infrastructure/src/
├── db/
│   ├── postgres_connector.rs   # impl DbConnector for PostgreSQL
│   ├── sqlite_connector.rs     # impl DbConnector for SQLite
│   └── ssh_tunnel.rs           # impl tunnel trait via openssh
├── secret/
│   └── keyring_vault.rs        # impl SecretStore via libsecret + AES-GCM
└── meta/
    └── sqlite_meta_store.rs    # impl MetaStore via rusqlite
```

---

## 18. Type Mapping: Rust ↔ TypeScript (DB Layer)

| Rust Type | TypeScript Type | Notes |
|---|---|---|
| `ConnectionId(pub Uuid)` | `string` (UUID) | UUID as string at boundary |
| `ConnectionConfig` | `ConnectionConfig` interface | 1:1 mapping |
| `DriverType` enum | `'postgres' \| 'sqlite'` | String enum |
| `SslMode` enum | `'disable' \| 'require' \| 'verify-ca' \| 'verify-full'` | String enum |
| `QueryResult` | `QueryResult` interface | 1:1 mapping |
| `ColumnMeta` | `ColumnMeta` interface | 1:1 mapping |
| `Row` | `Row` type (`string[]`) | Array of string values |
| `QueryError` | `QueryError` interface | 1:1 mapping |
| `Schema` | `Schema` interface | 1:1 mapping |
| `Table` | `Table` interface | 1:1 mapping |
| `Column` | `Column` interface | 1:1 mapping |
| `Index` | `Index` interface | 1:1 mapping |
| `ForeignKey` | `ForeignKey` interface | 1:1 mapping |
| `View` | `View` interface | 1:1 mapping |
| `ConnectionHandle` | Opaque `string` | Internal, not exposed to FE |
| `ExplainPlan` | `unknown` | JSON value, typed at FE usage |
| `Vec<T>` | `T[]` | Array |
| `Option<T>` | `T \| null` | Nullable |
| `Result<T, E>` | `Promise<T>` (errors via rejection) | Async |
| `serde_json::Value` | `unknown` | Typed at FE usage |
| `Vec<u8>` (encrypted) | `string` (base64) | Base64-encoded at boundary |
```

---

## 19. Data Flow Diagram (DB Layer)

```
User executes SQL in Monaco editor
  → QU01001 component calls queryService.execute(sql)
    → queryService.execute(sql)
      → tauri.invoke('execute_query', { connection_id, sql })
        → Tauri command (validate → resolve service → call → return Result<Dto, Dto>)
          → QueryService.execute(connection_id, sql)
            → MetaStore.get_connection(connection_id) → ConnectionHandle
            → DbConnector.query(handle, sql, []) → QueryResult
            → MetaStore.save_query_history(connection_id, sql, result)
            → Ok(QueryResult)
      ← Result<QueryResultDto, DbErrorDto>
    ← Result<QueryResultDto, DbErrorDto>
  ← Result<QueryResultDto, DbErrorDto>
    → TanStack Query cache updated
      → QU01001 component re-renders with new data

For large results (>10k rows):
  → QueryService.execute() detects large result set
    → Rust core starts streaming via app.emit_all("query-result-batch", batch)
      → FE subscribes to "query-result-batch" via tauri::Event
        → Each batch appended to TanStack Query cache
          → Virtualized grid renders new rows incrementally
```

---

## 20. Security Flow (DB Layer)

```
1. User saves connection
2. ConnectionService.create(config):
   a. validate_config(config) — check host, port, etc.
   b. test_connectivity(config) — verify connection works
   c. secret_store.encrypt(config.password) — encrypt password
   d. meta_store.save_connection(connection) — save encrypted config
3. User connects to saved connection
4. ConnectionService.connect(connection_id):
   a. meta_store.get_connection(connection_id) → retrieve encrypted config
   b. secret_store.decrypt(config.encrypted_password) → decrypt password
   c. db_connector.connect(config) → establish DB connection
   d. Store ConnectionHandle in MetaStore (active connection)
5. User executes query
6. QueryService.execute(connection_id, sql):
   a. meta_store.get_connection(connection_id) → get handle
   b. db_connector.query(handle, sql, []) → execute query
   c. meta_store.save_query_history(connection_id, sql, result) → save history
   d. Return QueryResult
```

---

## 21. Error Handling (DB Layer)

### 21.1 Error Type Hierarchy

```
DbError (domain error)
├── ConnectionNotFound { connection_id: String }
├── SqlError { source: sqlx::Error }
├── SshError { source: openssh::Error }
├── VaultError { source: keyring::Error }
├── Validation { message: String }
├── Timeout { timeout_ms: u64 }
├── PermissionDenied { table: String }
├── ConnectionLost
└── Internal { message: String }
```

### 21.2 Error Mapping to DTO

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

### 21.3 Error Code Reference

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

## 22. Observability (DB Layer)

### 22.1 Structured Logging

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

### 22.2 Audit Trail

Every action logged to local SQLite meta-store:

```
timestamp (UTC) | action_type (connect|query|export|edit|delete) | connection_id | success (bool) | duration_ms | error_message (if any)
```

### 22.3 Performance Metrics

| Metric | Logged |
|---|---|
| Query execution time | Yes (`duration_ms`) |
| Row count returned | Yes (`row_count`) |
| Connection status | Yes (connect/disconnect events) |
| Error occurrences | Yes (error type + message) |
| Export file size | Yes (for export actions) |
```

---

## 23. Sync with TypeScript Types

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