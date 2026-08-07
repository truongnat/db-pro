# Backend Contract Map

> Status: **Implemented** (P2-01 audit baseline)
> Last updated: 2026-08-08

This document maps every backend operation from Tauri command through application service to infrastructure implementation.

## Layer Architecture

```text
Tauri command (crates/tauri-app/src/commands/)
    ↓
Application service (crates/core/src/application/)
    ↓
Port / trait (crates/core/src/ports/)
    ↓
Infrastructure implementation (crates/infrastructure/src/)
```

---

## Connection Management

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `list_connections` | `ConnectionService::list` | `ConnectionRepository::list` | `SQLiteMetaStore` | All |
| `get_connection` | `ConnectionService::get` | `ConnectionRepository::get` | `SQLiteMetaStore` | All |
| `create_connection` | `ConnectionService::create` | `ConnectionRepository::save` + `SecretStore::store_secret` | `SQLiteMetaStore` + `KeyringVault` | All |
| `update_connection` | `ConnectionService::update` | `ConnectionRepository::save` + `SecretStore` | `SQLiteMetaStore` + `KeyringVault` | All |
| `delete_connection` | `ConnectionService::delete` | `ConnectionRepository::delete` + `SecretStore::delete_secret` | `SQLiteMetaStore` + `KeyringVault` | All |
| `test_connection` | `ConnectionService::test_connectivity` | `DbConnector::test_connection` | `CompositeConnector` → `PostgresConnector` / `SQLiteConnector` | All |
| `connect` | `ConnectionService::connect` | `DbConnector::connect` + `ConnectionRegistry` | `CompositeConnector` | All |
| `disconnect` | `ConnectionService::disconnect` | `DbConnector::disconnect` + `ConnectionRegistry` | `CompositeConnector` | All |
| `test_ssh_tunnel` | *(direct to infrastructure)* | N/A | `CompositeConnector::test_ssh_tunnel` → `SshTunnel` | All |

**Input DTO**: `ConnectionConfigDto` → `ConnectionConfig` (domain)
**Output DTO**: `ConnectionDto` ← `Connection` (domain)

---

## Query Execution

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `execute_query` | `QueryService::execute` | `DbConnector::query` | `CompositeConnector` | All |
| `cancel_query` | `CancelRegistry::cancel` | N/A (in-memory) | `CancelRegistry` | All |
| `execute_query_multi` | `QueryService::execute_multi` | `DbConnector::query` + `DbConnector::execute` | `CompositeConnector` | All |
| `explain_query` | `QueryService::explain` | `DbConnector::explain` | `CompositeConnector` | All |

**Input**: `connection_id: String`, `sql: String`
**Output DTO**: `QueryResultDto` ← `QueryResult`, `MultiQueryResultDto` ← `MultiQueryResult`

---

## Query History & Saved Queries

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `get_query_history` | `QueryService::get_history` | `QueryHistoryRepository::list` | `SQLiteMetaStore` | All |
| `save_query` | `QueryService::save_query` | `SavedQueryRepository::save` | `SQLiteMetaStore` | All |
| `list_saved_queries` | `QueryService::list_saved_queries` | `SavedQueryRepository::list` | `SQLiteMetaStore` | All |
| `delete_saved_query` | `QueryService::delete_saved_query` | `SavedQueryRepository::delete` | `SQLiteMetaStore` | All |
| `create_folder` | `QueryService::create_folder` | `SavedQueryRepository::create_folder` | `SQLiteMetaStore` | All |
| `list_folders` | `QueryService::list_folders` | `SavedQueryRepository::list_folders` | `SQLiteMetaStore` | All |
| `delete_folder` | `QueryService::delete_folder` | `SavedQueryRepository::delete_folder` | `SQLiteMetaStore` | All |
| `save_run_config` | `QueryService::save_run_config` | `RunConfigRepository::save` | `SQLiteMetaStore` | All |
| `list_run_configs` | `QueryService::list_run_configs` | `RunConfigRepository::list` | `SQLiteMetaStore` | All |
| `delete_run_config` | `QueryService::delete_run_config` | `RunConfigRepository::delete` | `SQLiteMetaStore` | All |

---

## Schema / Metadata

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `introspect` | `SchemaService::introspect` | `DbConnector::introspect` | `CompositeConnector` | All |
| `get_table_info` | `SchemaService::get_table_info` | `DbConnector::introspect` (cached) | `CompositeConnector` | All |
| `get_table_ddl` | `SchemaService::get_table_ddl` | `DdlBuilder` + introspection | `SchemaService` | All |
| `execute_ddl` | `SchemaService::execute_ddl` | `DbConnector::execute` | `CompositeConnector` | All |
| `create_index` | `SchemaService::execute_ddl` | `DbConnector::execute` | `CompositeConnector` | All |
| `drop_index` | `SchemaService::execute_ddl` | `DbConnector::execute` | `CompositeConnector` | All |
| `create_trigger` | `SchemaService::execute_ddl` | `DbConnector::execute` | `CompositeConnector` | All |
| `drop_trigger` | `SchemaService::execute_ddl` | `DbConnector::execute` | `CompositeConnector` | All |
| `invalidate_cache` | `SchemaService::invalidate_cache` | `IntrospectionCache::invalidate` | `SQLiteMetaStore` | All |

**Output DTO**: `IntrospectResultDto` ← `IntrospectResult`, `TableInfoDto` ← `TableInfo`

---

## Table Data

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `fetch_table_rows` | `TableDataService::fetch_rows` | `DbConnector::query` | `CompositeConnector` | All |
| `insert_table_row` | `TableDataService::insert_row` | `DbConnector::execute` | `CompositeConnector` | All |
| `update_table_row` | `TableDataService::update_row` | `DbConnector::execute` | `CompositeConnector` | All |
| `delete_table_row` | `TableDataService::delete_row` | `DbConnector::execute` | `CompositeConnector` | All |

---

## Export

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `export_csv` | `ExportService::export_csv` | `DbConnector::query` | `CompositeConnector` | All |
| `export_json` | `ExportService::export_json` | `DbConnector::query` | `CompositeConnector` | All |
| `export_excel` | `ExportService::export_excel` | `DbConnector::query` | `CompositeConnector` | All |

---

## User Management (PostgreSQL only)

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `list_users` | `UserService::list_users` | `UserManager::list_users` | `PostgresUserManager` | Postgres |
| `create_role` | `UserService::create_role` | `UserManager::create_role` | `PostgresUserManager` | Postgres |
| `drop_role` | `UserService::drop_role` | `UserManager::drop_role` | `PostgresUserManager` | Postgres |
| `list_privileges` | `UserService::list_privileges` | `UserManager::list_privileges` | `PostgresUserManager` | Postgres |
| `grant_privilege` | `UserService::grant_privilege` | `UserManager::grant_privilege` | `PostgresUserManager` | Postgres |
| `revoke_privilege` | `UserService::revoke_privilege` | `UserManager::revoke_privilege` | `PostgresUserManager` | Postgres |

---

## Backup / Restore

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `backup_database` | `BackupService::backup` | `BackupEngine::backup` | `PgDumpEngine` / `SqliteBackupEngine` | Postgres / SQLite |
| `restore_database` | `BackupService::restore` | `BackupEngine::restore` | `PgDumpEngine` / `SqliteBackupEngine` | Postgres / SQLite |

---

## Cross-Connection

| Command | Service | Port/Trait | Implementation | Drivers |
|---------|---------|------------|----------------|---------|
| `diff_schemas` | `SchemaService::diff_schemas` | `DbConnector::introspect` ×2 | `CompositeConnector` | All |
| `diff_table_data` | `DataDiffService::diff_table_data` | `DbConnector::query` ×2 | `CompositeConnector` | All |
| `get_object_dependencies` | *(direct to infrastructure)* | N/A | `PostgresConnector::get_object_dependencies` | **Postgres only** |
| `list_partitions` | *(direct to infrastructure)* | N/A | `PostgresConnector::list_partitions` | **Postgres only** |
| `list_tablespaces` | *(direct to infrastructure)* | N/A | `PostgresConnector::list_tablespaces` | **Postgres only** |
| `rename_schema_object` | *(direct to infrastructure)* | N/A | `PostgresConnector::rename_schema_object` | **Postgres only** |

---

## Error Transport

All commands return `Result<T, CommandError>` where:

```rust
pub struct CommandError {
    pub error: String,        // machine-readable code e.g. "CONNECTION_FAILED"
    pub message: String,      // human-readable message
    pub message_id: String,   // i18n key e.g. "error.connection.failed"
    pub details: Option<serde_json::Value>,
}
```

`DbError` → `CommandError` conversion is in `dto.rs`.

---

## Ports Summary

| Port | Defined In | Mock Available |
|------|-----------|---------------|
| `DbConnector` | `ports/db_connector.rs` | Yes (`MockDbConnector`) |
| `ConnectionRepository` | `ports/connection_repository.rs` | Yes |
| `SecretStore` | `ports/secret_store.rs` | Yes |
| `QueryHistoryRepository` | `ports/query_history_repository.rs` | Yes |
| `SavedQueryRepository` | `ports/saved_query_repository.rs` | Yes |
| `RunConfigRepository` | `ports/run_config_repository.rs` | Yes |
| `IntrospectionCache` | `ports/introspection_cache.rs` | Yes |
| `UserManager` | `ports/user_manager.rs` | Yes |
| `BackupEngine` | `ports/backup_engine.rs` | No |
| `SqlDialect` | `ports/dialect.rs` | No |
| `SettingsRepository` | `ports/settings_repository.rs` | No |
| `WorkspaceRepository` | `ports/workspace_repository.rs` | No |
