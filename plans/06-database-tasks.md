# DB Client — Database Layer Tasks

---

## 1. PostgreSQL Connector

### 1.1 Setup

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-001 | Add `sqlx` PostgreSQL support to `Cargo.toml` | `sqlx` with Tokio, rustls, PostgreSQL, chrono, uuid, and json features | — | 15m |
| D-002 | Verify the PostgreSQL pool strategy | Use `sqlx::PgPool`; do not add a second driver or pool abstraction without an ADR | D-001 | 30m |
| D-003 | Verify PostgreSQL client compiles | `cargo check` with the selected driver and features | D-002 | 30m |
| D-004 | Verify connection configuration compiles | Compile connection options, timeout, SSL, and pool configuration | D-003 | 30m |

### 1.2 Connection

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-005 | Implement connection string builder | Build `postgres://user:pass@host:port/db` from `ConnectionConfig` | D-001 | 1h |
| D-006 | Implement connection pooling | Create `sqlx::PgPoolOptions`, set max connections, connect timeout | D-005 | 1h |
| D-007 | Implement SSL mode handling | Map `SslMode` enum to `sqlx::postgres::PgSslMode` | D-005 | 1h |
| D-008 | Implement SSH tunnel integration | Use `openssh` crate to create tunnel, connect through tunnel | D-005 | 3h |
| D-009 | Implement connection timeout | Set `connect_timeout` on pool options, default 10s | D-006 | 30m |
| D-010 | Implement query timeout | Set `statement_timeout` on connection, default 30s | D-006 | 30m |
| D-011 | Implement connection health check | `SELECT 1` on connect, periodic ping | D-006 | 1h |
| D-012 | Implement connection retry logic | Retry 3 times with exponential backoff on connection failure | D-009 | 1h |

### 1.3 Query Execution

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-013 | Implement parameterized query execution | Map the explicit domain `QueryParam` enum to `sqlx::Arguments`; reject unsupported values | D-006 | 2h |
| D-014 | Implement row mapping | Map `sqlx::Row` to `Row` type (`Vec<String>`) | D-013 | 1h |
| D-015 | Implement column metadata extraction | Extract column name, type, nullable from `pg_description` | D-013 | 1h |
| D-016 | Implement result set size limit | Enforce `max_rows` from `ConnectionConfig`, default 100k | D-013 | 30m |
| D-017 | Implement query cancellation | `PgCancelQuery` on `PgPool` for canceling running queries | D-013 | 1h |
| D-018 | Enforce single-statement MVP policy | Reject multiple statements; parser-backed execution is deferred behind a new ADR | D-013 | 1h |
| D-019 | Implement transaction support | `BEGIN`/`COMMIT`/`ROLLBACK` via `PgPool::begin()` | D-013 | 2h |
| D-020 | Implement streaming for large results | Tauri 2 `Channel<QueryStreamEvent>`, bounded batches, cancellation, and payload limits | D-013 | 3h |

### 1.4 Schema Introspection

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-021 | Implement introspection query 1: list schemas | `information_schema.schemata` | D-013 | 1h |
| D-022 | Implement introspection query 2: list tables | `information_schema.tables` per schema | D-021 | 1h |
| D-023 | Implement introspection query 3: list columns | `information_schema.columns` per table | D-022 | 1h |
| D-024 | Implement introspection query 4: list primary keys | `information_schema.table_constraints` + `key_column_usage` | D-022 | 1h |
| D-025 | Implement introspection query 5: list indexes | `pg_indexes` per table | D-022 | 1h |
| D-026 | Implement introspection query 6: list foreign keys | `information_schema.table_constraints` + `key_column_usage` + `constraint_column_usage` | D-022 | 1h |
| D-027 | Implement introspection query 7: list views | `information_schema.views` per schema | D-021 | 1h |
| D-028 | Implement introspection query 8: list triggers | `information_schema.triggers` per table | D-022 | 1h |
| D-029 | Implement introspection query 9: list functions | `information_schema.routines` per schema | D-021 | 1h |
| D-030 | Implement introspection query 10: DDL for table | Reconstruct CREATE TABLE from `information_schema` | D-023 | 2h |
| D-031 | Implement introspection query 11: row count | `pg_class.reltuples` per table | D-022 | 30m |
| D-032 | Implement introspection caching | Cache results in `MetaStore`, invalidate on connection change | D-021 | 2h |
| D-033 | Implement introspection timeout | Default 30s for introspection, configurable | D-021 | 30m |

### 1.5 Explain Plan

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-034 | Implement safe explain for SELECT | Run plain `EXPLAIN (FORMAT JSON)` by default; require explicit confirmation for `EXPLAIN ANALYZE` | D-013 | 2h |
| D-035 | Implement explain plan parsing | Parse JSON output into `ExplainPlan` struct | D-034 | 1h |
| D-036 | Implement explain plan display format | Convert `ExplainPlan` to tree structure for FE | D-035 | 1h |

### 1.6 PostgreSQL Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-037 | Set up PostgreSQL test container | Docker compose with PostgreSQL 14+, sample schema | C-054 | 1h |
| D-038 | Test connection with real PostgreSQL | Verify all connection options work against real PG | D-037 | 1h |
| D-039 | Test query execution with real PostgreSQL | Verify SELECT, INSERT, UPDATE, DELETE work | D-037 | 1h |
| D-040 | Test introspection with real PostgreSQL | Verify all 11 introspection queries return correct data | D-037 | 2h |
| D-041 | Test streaming with large result sets | Insert 50k+ rows, verify streaming works | D-037 | 1h |
| D-042 | Test transaction support | Verify BEGIN/COMMIT/ROLLBACK with real PG | D-037 | 1h |
| D-043 | Test SSH tunnel with real PostgreSQL | Verify tunnel connection works against PG through SSH | D-037 | 2h |
| D-044 | Test SSL connection modes | Verify disable/require/verify-ca/verify-full all work | D-037 | 1h |
| D-045 | Test connection retry logic | Simulate connection failure, verify retry | D-037 | 1h |
| D-046 | Test query timeout enforcement | Run long-running query, verify timeout kills it | D-037 | 1h |
| D-047 | Test result set size limit | Return >100k rows, verify limit enforced | D-037 | 1h |

## 2. SQLite Connector

### 2.1 Setup

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-048 | Add `rusqlite` to `Cargo.toml` | `rusqlite = { version = "0.30", features = ["bundled", "chrono", "serde_json"] }` | — | 15m |
| D-049 | Verify SQLite client compiles | `cargo check` with new dep | D-048 | 15m |

### 2.2 Connection

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-050 | Implement SQLite connection | Open file-based or in-memory SQLite DB | D-048 | 1h |
| D-051 | Implement SQLite worker boundary | Run `rusqlite::Connection` on a dedicated worker/blocking boundary; do not share it across async handlers | D-050 | 1h |
| D-052 | Implement WAL mode | Set `PRAGMA journal_mode=WAL` on connect | D-050 | 30m |
| D-053 | Implement foreign key enforcement | Set `PRAGMA foreign_keys=ON` on connect | D-050 | 30m |

### 2.3 Query Execution

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-054 | Implement parameterized query execution | `rusqlite::Connection::prepare()` + `query_map()` | D-050 | 1h |
| D-055 | Implement row mapping for SQLite | Map `rusqlite::Row` to `Row` type | D-054 | 1h |
| D-056 | Implement column metadata extraction | `PRAGMA table_info(table_name)` | D-054 | 30m |
| D-057 | Implement result set size limit | Enforce `max_rows`, default 100k | D-054 | 30m |

### 2.4 Schema Introspection

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-058 | Implement SQLite introspection: list tables | `sqlite_master WHERE type='table'` | D-050 | 1h |
| D-059 | Implement SQLite introspection: list columns | `PRAGMA table_info(table_name)` per table | D-058 | 1h |
| D-060 | Implement SQLite introspection: list indexes | `PRAGMA index_list(table_name)` + `PRAGMA index_info()` | D-058 | 1h |
| D-061 | Implement SQLite introspection: list foreign keys | `PRAGMA foreign_key_list(table_name)` | D-058 | 30m |
| D-062 | Implement SQLite introspection: list views | `sqlite_master WHERE type='view'` | D-058 | 30m |
| D-063 | Implement SQLite introspection: row count | `SELECT COUNT(*) FROM table` or `sqlite_stat` | D-058 | 30m |
| D-064 | Implement SQLite introspection: DDL for table | `SELECT sql FROM sqlite_master WHERE name='table'` | D-058 | 30m |

### 2.5 SQLite Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-065 | Test SQLite connection with in-memory DB | Verify basic operations | D-050 | 30m |
| D-066 | Test SQLite connection with file-based DB | Verify file operations | D-050 | 30m |
| D-067 | Test SQLite query execution | Verify SELECT, INSERT, UPDATE, DELETE | D-065 | 30m |
| D-068 | Test SQLite introspection | Verify all introspection queries return correct data | D-065 | 1h |
| D-069 | Test SQLite WAL mode | Verify WAL journal mode is set on connect | D-065 | 30m |
| D-070 | Test SQLite foreign key enforcement | Verify FK constraints are enforced | D-065 | 30m |

## 3. Secret Vault

### 3.1 Setup

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-071 | Add `keyring` to `Cargo.toml` | `keyring = "0.16"` | — | 15m |
| D-072 | Add `aes-gcm` to `Cargo.toml` | `aes-gcm = "0.10"` | — | 15m |
| D-073 | Add `argon2` to `Cargo.toml` | Argon2id for deriving the fallback encryption key from the user master password | — | 15m |
| D-074 | Add `rand` to `Cargo.toml` | `rand = "0.8"` for generating salt/nonce | — | 15m |
| D-075 | Verify crypto deps compile | `cargo check` with new deps | D-071-D-074 | 30m |

### 3.2 Implementation

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-076 | Implement Argon2id key derivation | Derive a 256-bit key from the user master password and salt | D-071-D-074 | 1h |
| D-077 | Implement AES-256-GCM encryption | Encrypt password with derived key + random nonce | D-076 | 1h |
| D-078 | Implement AES-256-GCM decryption | Decrypt ciphertext with derived key + nonce | D-077 | 1h |
| D-079 | Implement OS keyring storage | Store encrypted password in system keyring via `keyring` crate | D-076 | 1h |
| D-080 | Implement OS keyring retrieval | Retrieve encrypted password from system keyring | D-079 | 1h |
| D-081 | Implement keyring fallback | If keyring fails, fall back to file-based encrypted storage | D-079 | 1h |
| D-082 | Implement master key management | Generate/retrieve master key from OS keyring | D-079 | 1h |

### 3.3 Secret Vault Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-083 | Test encrypt/decrypt round-trip | Encrypt then decrypt returns original password | D-077 | 30m |
| D-084 | Test keyring store/retrieve round-trip | Store then retrieve returns original encrypted password | D-079 | 30m |
| D-085 | Test keyring fallback | Simulate keyring failure, verify file-based fallback works | D-081 | 30m |
| D-086 | Test with different OS keyrings | Test on GNOME (libsecret) and KDE (kwallet) if possible | D-083 | 1h |
| D-087 | Test that passwords are never logged | Verify `tracing` redacts sensitive fields | D-077 | 30m |

## 4. Meta-Store

### 4.1 Schema

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-088 | Design meta-store schema | `connections`, `query_history`, `introspection_cache`, `settings`, `audit_log` tables | — | 30m |
| D-089 | Create migration file `migrations/001_init.sql` | All table schemas with indexes | D-088 | 30m |
| D-090 | Create migration file `migrations/002_audit_log.sql` | Audit log table for action tracking | D-089 | 15m |

### 4.2 Implementation

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-091 | Implement meta-store connection | Open SQLite meta-store DB file, run migrations | D-089 | 1h |
| D-092 | Implement connection CRUD in meta-store | Save, list, get, delete connections with encrypted passwords | D-091 | 2h |
| D-093 | Implement query history in meta-store | Save, list, clear query history entries | D-091 | 1h |
| D-094 | Implement introspection cache in meta-store | Cache/get/clear introspection results | D-091 | 1h |
| D-095 | Implement settings in meta-store | Get/set app settings (language, theme, page size) | D-091 | 1h |
| D-096 | Implement audit log in meta-store | Log all actions: connect, query, export, edit, delete | D-091 | 1h |
| D-097 | Implement meta-store migration runner | Auto-run pending migrations on first access | D-089 | 1h |

### 4.3 Meta-Store Testing

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-098 | Test meta-store with in-memory SQLite | All CRUD operations against in-memory DB | D-091 | 1h |
| D-099 | Test connection CRUD | Create, read, update, delete connections | D-098 | 1h |
| D-100 | Test query history CRUD | Save, retrieve, clear history | D-098 | 1h |
| D-101 | Test introspection cache | Set, get, invalidate cache | D-098 | 1h |
| D-102 | Test settings CRUD | Get, set, update settings | D-098 | 30m |
| D-103 | Test audit log | Verify all actions are logged with timestamp | D-098 | 30m |
| D-104 | Test meta-store migration | Verify migrations run correctly on fresh DB | D-097 | 30m |
| D-105 | Test encrypted password storage | Verify passwords are encrypted at rest in meta-store | D-099 | 1h |

## 5. Type Mapping

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| D-106 | Create Rust↔TypeScript type mapping table | Document in `06-types.md` and this file | — | 30m |
| D-107 | Verify type mapping consistency | Cross-check `03-types.ts`, `05-be-architecture.md`, `06-types.md` | D-106 | 30m |
