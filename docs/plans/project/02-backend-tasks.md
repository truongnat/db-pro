# DB Pro — Backend (Rust) Tasks

---

## Phase 0: Scaffolding

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-001 | Initialize Cargo workspace | `cargo init --lib`, set up workspace with `core/`, `infrastructure/`, `tauri-app/` | — | 2h |
| B-002 | Create Cargo.toml files | `core/Cargo.toml`, `infrastructure/Cargo.toml`, `tauri-app/Cargo.toml` with correct dependencies | B-001 | 1h |
| B-003 | Create folder structure | `core/domain/`, `core/application/`, `core/ports/`, `infrastructure/db/`, `infrastructure/secret/`, `infrastructure/meta/`, `tauri-app/src/commands/` | B-002 | 1h |
| B-004 | Set up rustfmt + clippy config | `rust-toolchain.toml`, `.rustfmt.toml`, `clippy.toml` | B-003 | 30m |
| B-005 | Verify `cargo build` passes | Empty project compiles | B-004 | 30m |

## Phase 1: Core Domain + Ports

### 1.1 Domain Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-006 | Create `ConnectionId` newtype | `pub struct ConnectionId(pub Uuid)`, with `new()`, `parse()`, `Display`, `Serialize`, `Deserialize` | B-005 | 2h |
| B-007 | Create `ConnectionConfig` struct | All fields from `03-types.md`: name, host, port, database, username, encrypted_password, driver, ssl_mode, ssh_tunnel, query_timeout_ms, max_rows | B-006 | 2h |
| B-008 | Create `DriverType` enum | `Postgres`, `SQLite` | B-006 | 1h |
| B-009 | Create `SslMode` enum | `Disable`, `Require`, `VerifyCa`, `VerifyFull` | B-006 | 1h |
| B-010 | Create `SshTunnelConfig` struct | host, port, user, private_key_path | B-006 | 1h |
| B-011 | Create `Connection` entity | id, config, created_at, updated_at | B-006 | 1h |
| B-012 | Create `QueryResult` struct | columns, rows, row_count, duration_ms | B-005 | 1h |
| B-013 | Create `QueryError` enum | `Validation`, `NotFound`, `Conflict`, `Unauthorized`, `Internal` — each with contextual data | B-005 | 2h |
| B-014 | Create `Schema` struct | name | B-005 | 30m |
| B-015 | Create `Table` struct | name, schema | B-005 | 30m |
| B-016 | Create `Column` struct | name, data_type, nullable, default_value | B-005 | 30m |
| B-017 | Create `Index` struct | name, columns, unique | B-005 | 30m |
| B-018 | Create `ForeignKey` struct | name, from_table, from_column, to_table, to_column | B-005 | 30m |
| B-019 | Create `View` struct | name, definition | B-005 | 30m |
| B-020 | Create `ExplainPlan` type | `serde_json::Value` or custom struct | B-012 | 1h |
| B-021 | Create connection handle registry | PostgreSQL uses `sqlx::PgPool`; SQLite uses an opaque actor handle — neither is exposed to FE | B-005 | 1h |
| B-022 | Create `QueryHistory` struct | sql, executed_at, duration_ms, row_count | B-005 | 1h |
| B-023 | Create `SavedQuery` struct | id, name, sql, folder, created_at | B-005 | 1h |

### 1.2 Ports (Traits)

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-024 | Create `DbConnector` trait | `connect`, `disconnect`, `query`, `introspect`, `explain` — all async, `Send + Sync` | B-006, B-021 | 2h |
| B-025 | Create `SecretStore` trait | `encrypt`, `decrypt`, `store`, `retrieve` | B-005 | 1h |
| B-026 | Create `MetaStore` trait | `list_connections`, `save_connection`, `delete_connection`, `get_connection`, `save_query_history`, `get_query_history`, `get_introspection_cache`, `set_introspection_cache` | B-005 | 2h |

### 1.3 Domain Validation

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-027 | Implement `ConnectionConfig::validate()` | Validate all fields: name non-empty, host non-empty, port 1-65535, etc. | B-007 | 1h |
| B-028 | Write domain unit tests | Test `ConnectionId::parse()` invalid UUID, `ConnectionConfig::validate()` missing fields | B-027 | 1h |

## Phase 2: Infrastructure Adapters

### 2.1 PostgresConnector

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-029 | Create `PostgresConnector` struct | Fields: `pool: Option<sqlx::PgPool>` | B-024 | 2h |
| B-030 | Implement `connect()` | Build connection string from `ConnectionConfig`, create `sqlx::PgPool`, return `ConnectionHandle::Postgres(pool)` | B-029 | 2h |
| B-031 | Implement `disconnect()` | Drop pool, set handle to None | B-030 | 1h |
| B-032 | Implement `query()` | Execute SQL with parameters via `sqlx::query()`, map rows to `QueryResult` | B-030 | 3h |
| B-033 | Implement `introspect()` | Run all 11 introspection queries from `06-db-architecture.md`, return `Schema` | B-030 | 4h |
| B-034 | Implement `explain()` | Run plain `EXPLAIN (FORMAT JSON)` by default; require explicit confirmation for `EXPLAIN ANALYZE` | B-030 | 2h |

### 2.2 SQLiteConnector

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-035 | Create `SQLiteActor` connector | Dedicated Tokio task owns `rusqlite::Connection`; callers use typed commands | B-024 | 1h |
| B-036 | Implement `connect()` | Open SQLite file or memory DB, return `ConnectionHandle::SQLite(conn)` | B-035 | 1h |
| B-037 | Implement `disconnect()` | Close connection | B-036 | 30m |
| B-038 | Implement `query()` | Execute SQL via `rusqlite`, map rows to `QueryResult` | B-036 | 2h |
| B-039 | Implement `introspect()` | Query SQLite `sqlite_master` and `pragma_table_info`, return `Schema` | B-036 | 2h |
| B-040 | Implement `explain()` | Run `EXPLAIN QUERY PLAN`, return `ExplainPlan` | B-036 | 1h |

### 2.3 KeyringVault

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-041 | Create `KeyringVault` struct | Uses `keyring` crate for OS keyring, `aes-gcm` for encryption | B-025 | 2h |
| B-042 | Implement `encrypt()` | Argon2id key derivation from user master password, AES-256-GCM encryption | B-041 | 2h |
| B-043 | Implement `decrypt()` | Reverse of encrypt | B-042 | 2h |
| B-044 | Implement `store()` | Save encrypted password to OS keyring | B-042 | 1h |
| B-045 | Implement `retrieve()` | Read encrypted password from OS keyring, decrypt | B-043 | 1h |

### 2.4 SQLiteMetaStore

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-046 | Create meta-store SQL schema | `connections`, `query_history`, `introspection_cache`, `settings` tables | B-026 | 1h |
| B-047 | Create `SQLiteMetaStore` struct | Uses `rusqlite` to connect to meta-store DB file | B-046 | 1h |
| B-048 | Implement `list_connections()` | Query connections table, decrypt passwords | B-047 | 2h |
| B-049 | Implement `save_connection()` | Insert/update connection, encrypt password | B-047 | 2h |
| B-050 | Implement `delete_connection()` | Delete from connections table | B-048 | 1h |
| B-051 | Implement `get_connection()` | Query by ID, decrypt password | B-048 | 1h |
| B-052 | Implement `save_query_history()` | Insert query history entry | B-049 | 1h |
| B-053 | Implement `get_query_history()` | Query history by connection_id, limit | B-052 | 1h |
| B-054 | Implement `get_introspection_cache()` / `set_introspection_cache()` | Cache introspection results | B-053 | 1h |

### 2.5 Error Types

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-055 | Create `DbError` enum with `thiserror` | All error variants with contextual data: `ConnectionFailed`, `QueryFailed`, `IntrospectionFailed`, `EncryptionFailed`, `ValidationError`, etc. | B-005 | 2h |
| B-056 | Implement `From` conversions | `From<sqlx::Error>`, `From<rusqlite::Error>`, `From<keyring::Error>`, `From<aes_gcm::Error>` for `DbError` | B-055 | 1h |

### 2.6 Infrastructure Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-057 | Write unit tests for `PostgresConnector` | Use `mockall` to mock `DbConnector` trait, test connect/disconnect/query | B-034 | 2h |
| B-058 | Write unit tests for `SQLiteConnector` | Same pattern | B-040 | 2h |
| B-059 | Write unit tests for `KeyringVault` | Mock keyring, test encrypt/decrypt round-trip | B-045 | 1h |
| B-060 | Write unit tests for `SQLiteMetaStore` | Use in-memory SQLite, test CRUD operations | B-054 | 2h |

## Phase 3: Application Services

### 3.1 ConnectionService

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-061 | Create `ConnectionService` struct | Receives `DbConnector` + `MetaStore` via constructor injection | B-024, B-026 | 1h |
| B-062 | Implement `create()` | Validate config, encrypt password, save to meta-store | B-061 | 2h |
| B-063 | Implement `list()` | Query meta-store for all connections (without passwords) | B-061 | 1h |
| B-064 | Implement `get_by_id()` | Query meta-store by ID | B-061 | 1h |
| B-065 | Implement `update()` | Update connection config in meta-store | B-061 | 1h |
| B-066 | Implement `delete()` | Delete from meta-store | B-061 | 1h |
| B-067 | Implement `test_connectivity()` | Create temp connection, run `SELECT 1`, disconnect | B-061 | 2h |
| B-068 | Implement `connect()` | Get config from meta-store, call `DbConnector::connect()`, store handle | B-061 | 2h |
| B-069 | Implement `disconnect()` | Get handle, call `DbConnector::disconnect()`, remove from active handles | B-061 | 1h |
| B-070 | Write unit tests for `ConnectionService` | Mock `DbConnector` and `MetaStore`, test all methods including error paths | B-062-B-069 | 3h |

### 3.2 QueryService

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-071 | Create `QueryService` struct | Receives `DbConnector` + `MetaStore` via constructor injection | B-024, B-026 | 1h |
| B-072 | Implement `execute()` | Get connection handle, call `DbConnector::query()`, save to history, return `QueryResult` | B-071 | 3h |
| B-073 | Reject multi-statement execution in MVP | Require one selected statement; future parser-backed execution requires a new ADR | B-072 | 1h |
| B-074 | Implement `explain()` | Get handle, call `DbConnector::explain()`, return `ExplainPlan` | B-071 | 1h |
| B-075 | Implement `get_history()` | Query meta-store for history entries | B-071 | 1h |
| B-076 | Implement `save_to_history()` | Save query + result metadata to meta-store | B-071 | 1h |
| B-077 | Implement bounded query streaming | Use Tauri 2 `Channel<QueryStreamEvent>` with request ID, batch limits, and cancellation | B-072 | 3h |
| B-078 | Write unit tests for `QueryService` | Mock all dependencies, test execute, multi-execute, error paths | B-072-B-077 | 3h |

### 3.3 SchemaService

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-079 | Create `SchemaService` struct | Receives `DbConnector` + `MetaStore` via constructor injection | B-024, B-026 | 1h |
| B-080 | Implement `introspect()` | Call `DbConnector::introspect()`, cache result in meta-store | B-079 | 2h |
| B-081 | Implement `get_table_ddl()` | Run introspection, reconstruct CREATE TABLE DDL | B-080 | 2h |
| B-082 | Implement `get_table_info()` | Return columns, indexes, foreign keys, triggers for a table | B-080 | 2h |
| B-083 | Write unit tests for `SchemaService` | Mock dependencies, test introspect, DDL generation | B-080-B-082 | 2h |

### 3.4 ExportService

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-084 | Create `ExportService` struct | Receives `DbConnector` via constructor injection | B-024 | 1h |
| B-085 | Implement `export_csv()` | Execute query, stream rows to CSV using `csv` crate, return file content | B-084 | 3h |
| B-086 | Implement `export_json()` | Execute query, stream rows to JSON, return file content | B-084 | 2h |
| B-087 | Implement `export_excel()` | Execute query, write to xlsx using `rust_xlsxwriter`, return file content | B-084 | 3h |
| B-088 | Write unit tests for `ExportService` | Mock `DbConnector`, test CSV/JSON export with sample data | B-085-B-087 | 2h |

## Phase 4: Tauri Commands

### 4.1 Command Handlers

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-089 | Create `tauri-app/src/commands/connection.rs` | `list_connections`, `create_connection`, `update_connection`, `delete_connection`, `test_connection`, `connect`, `disconnect` | B-062-B-069 | 3h |
| B-090 | Create `tauri-app/src/commands/query.rs` | `execute_query`, `execute_multi_query`, `explain_query`, `get_query_history`, `cancel_query` | B-072-B-077 | 3h |
| B-091 | Create `tauri-app/src/commands/schema.rs` | `introspect_schema`, `get_table_ddl`, `get_table_info` | B-080-B-082 | 2h |
| B-092 | Create `tauri-app/src/commands/export.rs` | `export_csv`, `export_json`, `export_excel` | B-085-B-087 | 2h |
| B-093 | Create `tauri-app/src/commands/history.rs` | `save_query_history`, `get_query_history` | B-075-B-076 | 1h |

### 4.2 DTO Mapping

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-094 | Create `DbErrorDto` struct | Maps `DbError` to JSON-serializable error envelope: `{ error, message, messageId, details }` | B-055 | 1h |
| B-095 | Create `ConnectionDto` struct | Maps `Connection` to Tauri-boundary DTO | B-011 | 1h |
| B-096 | Create `QueryResultDto` struct | Maps `QueryResult` to Tauri-boundary DTO | B-012 | 1h |
| B-097 | Create `SchemaDto`, `TableDto`, `ColumnDto`, etc. | Maps domain types to Tauri-boundary DTOs | B-014-B-019 | 2h |
| B-098 | Implement `From<DbError> for DbErrorDto` | Error mapping at Tauri boundary | B-094 | 1h |

### 4.3 App State & DI

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-099 | Set up Tauri app state | `app.manage(QueryService::new(...))`, `app.manage(ConnectionService::new(...))`, etc. | B-089-B-093 | 2h |
| B-100 | Wire up dependency injection in `tauri-app/src/main.rs` | Create infrastructure adapters, inject into services, register as app state | B-099 | 2h |
| B-101 | Create `tauri-app/src/lib.rs` | Re-export all commands, services, and state setup | B-100 | 1h |

### 4.4 Tauri Command Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-102 | Write integration tests for Tauri commands | Test command → service → port chain with mocked dependencies | B-094-B-101 | 3h |
| B-103 | Write error path tests | Test each command with invalid input, missing connection, query errors | B-102 | 2h |

## Phase 4b: Meta-Store SQL Schema

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| B-104 | Create meta-store SQL migration file | `migrations/001_init.sql` with all table schemas | B-046 | 1h |
| B-105 | Create meta-store migration runner | Auto-runs migrations on first access | B-104 | 1h |
