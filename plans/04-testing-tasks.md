# DB Client — Testing Tasks

---

## 1. Testing Strategy

| Layer | Tool | Coverage Target | Location |
|---|---|---|---|
| Rust unit tests | `cargo test` + `mockall` | ≥ 80% on `core` | `src/` inline `#[cfg(test)]` |
| Rust integration tests | `cargo test --test` | All trait impls tested | `tests/` directory |
| TS unit tests | `vitest` | ≥ 70% on `modules/` | `*.test.ts` / `*.test.tsx` alongside source |
| TS component tests | `vitest` + `@testing-library/react` | All components rendered | `*.test.tsx` alongside source |
| E2E tests | `playwright` | Critical user flows | `e2e/` directory |
| Coverage gate | `cargo tarpaulin` / `vitest --coverage` | 80% Rust / 70% TS | CI |

## 2. Rust Testing Tasks

### 2.1 Domain Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-001 | Test `ConnectionId::new()` | Generates valid UUID | B-028 | 30m |
| T-002 | Test `ConnectionId::parse()` valid | Parses valid UUID string | B-028 | 30m |
| T-003 | Test `ConnectionId::parse()` invalid | Returns error for non-UUID string | B-028 | 30m |
| T-004 | Test `ConnectionConfig::validate()` happy path | Valid config passes | B-028 | 30m |
| T-005 | Test `ConnectionConfig::validate()` missing name | Returns Validation error | B-028 | 30m |
| T-006 | Test `ConnectionConfig::validate()` missing host | Returns Validation error | B-028 | 30m |
| T-007 | Test `ConnectionConfig::validate()` invalid port | Returns Validation error for port 0 and 65536 | B-028 | 30m |
| T-008 | Test `QueryError` variants | All variants carry correct contextual data | B-013 | 30m |
| T-009 | Test `DbError` `From` conversions | `sqlx::Error` → `DbError`, `rusqlite::Error` → `DbError` | B-056 | 1h |

### 2.2 Port Trait Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-010 | Test `DbConnector` trait object | `Box<dyn DbConnector + Send + Sync>` compiles and dispatches | B-024 | 30m |
| T-011 | Test `SecretStore` trait object | `Box<dyn SecretStore + Send + Sync>` compiles | B-025 | 30m |
| T-012 | Test `MetaStore` trait object | `Box<dyn MetaStore + Send + Sync>` compiles | B-026 | 30m |

### 2.3 Infrastructure Adapter Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-013 | Test `PostgresConnector::connect()` with mock pool | Verify connection string built correctly | B-034 | 1h |
| T-014 | Test `PostgresConnector::query()` with mock pool | Verify SQL and params passed correctly | B-034 | 1h |
| T-015 | Test `PostgresConnector::introspect()` with mock pool | Verify all 11 introspection queries executed | B-034 | 2h |
| T-016 | Test `PostgresConnector::explain()` with mock pool | Verify EXPLAIN ANALYZE query | B-034 | 1h |
| T-017 | Test `SQLiteConnector::connect()` with in-memory DB | Verify connection opens | B-040 | 1h |
| T-018 | Test `SQLiteConnector::query()` with in-memory DB | Verify SQL execution and row mapping | B-040 | 1h |
| T-019 | Test `SQLiteConnector::introspect()` with in-memory DB | Verify schema introspection for SQLite | B-040 | 1h |
| T-020 | Test `KeyringVault::encrypt()/decrypt()` round-trip | Encrypt then decrypt returns original | B-045 | 1h |
| T-021 | Test `KeyringVault::store()/retrieve()` round-trip | Store then retrieve returns original | B-045 | 1h |
| T-022 | Test `SQLiteMetaStore` CRUD with in-memory DB | Test all meta-store operations | B-054 | 2h |
| T-023 | Test `SQLiteMetaStore` introspection cache | Test cache set/get/invalidation | B-054 | 1h |

### 2.4 Application Service Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-024 | Test `ConnectionService::create()` | Mock MetaStore, verify save called with encrypted password | B-070 | 1h |
| T-025 | Test `ConnectionService::list()` | Mock MetaStore, verify list returns connections | B-070 | 1h |
| T-026 | Test `ConnectionService::test_connectivity()` success | Mock DbConnector, verify connect/disconnect called | B-070 | 1h |
| T-027 | Test `ConnectionService::test_connectivity()` failure | Mock DbConnector returning error | B-070 | 1h |
| T-028 | Test `ConnectionService::connect()` | Mock MetaStore + DbConnector, verify handle stored | B-070 | 1h |
| T-029 | Test `ConnectionService::disconnect()` | Mock MetaStore + DbConnector, verify handle removed | B-070 | 1h |
| T-030 | Test `QueryService::execute()` | Mock DbConnector + MetaStore, verify query + history saved | B-078 | 2h |
| T-031 | Test `QueryService::execute()` error path | Mock DbConnector returning error, verify error propagated | B-078 | 1h |
| T-032 | Test `QueryService::execute_multi()` | Mock DbConnector, verify each statement executed | B-078 | 1h |
| T-033 | Test `QueryService::explain()` | Mock DbConnector, verify EXPLAIN ANALYZE call | B-078 | 1h |
| T-034 | Test `QueryService::get_history()` | Mock MetaStore, verify history returned | B-078 | 1h |
| T-035 | Test `QueryService::streaming()` | Mock DbConnector returning >10k rows, verify streaming | B-078 | 2h |
| T-036 | Test `SchemaService::introspect()` | Mock DbConnector + MetaStore, verify cache logic | B-083 | 1h |
| T-037 | Test `SchemaService::get_table_ddl()` | Mock DbConnector, verify DDL generation | B-083 | 1h |
| T-038 | Test `ExportService::export_csv()` | Mock DbConnector, verify CSV output format | B-088 | 1h |
| T-039 | Test `ExportService::export_json()` | Mock DbConnector, verify JSON output format | B-088 | 1h |
| T-040 | Test `ExportService::export_excel()` | Mock DbConnector, verify xlsx output | B-088 | 1h |

### 2.5 Tauri Command Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-041 | Test `list_connections` command | Mock services, verify DTO mapping | B-102 | 1h |
| T-042 | Test `execute_query` command | Mock services, verify input validation + DTO mapping | B-102 | 1h |
| T-043 | Test `execute_query` error mapping | Verify `DbError` → `DbErrorDto` mapping | B-102 | 1h |
| T-044 | Test all commands with invalid input | Verify validation errors returned as DTOs | B-102 | 1h |

## 3. TypeScript Testing Tasks

### 3.1 Service Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-045 | Test `connection.service.ts` | Mock `tauri.invoke`, test all CRUD + connect/disconnect/test methods | F-055 | 1h |
| T-046 | Test `query.service.ts` | Mock `tauri.invoke`, test execute, history, error paths | F-073 | 1h |
| T-047 | Test `schema.service.ts` | Mock `tauri.invoke`, test introspect, DDL | F-085 | 1h |
| T-048 | Test `export.service.ts` | Mock `tauri.invoke`, test CSV/JSON export | F-113 | 1h |

### 3.2 Hook Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-049 | Test `useConnectionList()` | Mock TanStack Query, verify data returned | F-045 | 1h |
| T-050 | Test `useExecuteQuery()` | Mock TanStack Query mutation, verify execute + cache update | F-061 | 1h |
| T-051 | Test `useIntrospect()` | Mock TanStack Query, verify schema data | F-079 | 1h |
| T-052 | Test `useTableData()` | Mock TanStack Query, verify pagination + sorting + filtering | F-090 | 1h |
| T-053 | Test `useExport()` | Mock TanStack Query mutation, verify export | F-109 | 1h |

### 3.3 Component Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-054 | Test `connection-list.tsx` | Render list, test click to select, loading state, empty state | F-056 | 1h |
| T-055 | Test `connection-editor.tsx` | Render form, test validation, submit, error display | F-057 | 1h |
| T-056 | Test `connection-status.tsx` | Test connected/disconnected/connecting/error states | F-056 | 30m |
| T-057 | Test `query-editor.tsx` | Render Monaco, test value changes, keyboard shortcuts | F-074 | 1h |
| T-058 | Test `result-grid.tsx` | Render grid, test sort, filter, pagination, inline edit | F-075 | 2h |
| T-059 | Test `schema-tree.tsx` | Render tree, test expand/collapse, selection | F-086 | 1h |
| T-060 | Test `table-detail.tsx` | Render columns/indexes/FKs, test DDL view | F-086 | 1h |
| T-061 | Test `data-grid.tsx` | Render grid, test all CRUD operations, error states | F-104 | 2h |
| T-062 | Test `cell-editor.tsx` | Test edit flow, validation, error highlight | F-105 | 1h |
| T-063 | Test `export-dialog.tsx` | Render dialog, test format selection, submit | F-113 | 30m |
| T-064 | Test `ErrorBoundary` | Test error boundary catches errors, shows fallback UI | F-026 | 30m |

### 3.4 Utility Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-065 | Test `api.ts` | Mock `tauri.invoke`, test success and error paths | F-019 | 30m |
| T-066 | Test `server-error-normalize.ts` | Test error normalization for all error types | F-020 | 30m |
| T-067 | Test `server-error-translate.ts` | Test i18n key resolution | F-021 | 30m |
| T-068 | Test `validation.ts` | Test Zod schema validation for all input types | F-022 | 30m |
| T-069 | Test `clipboard.ts` | Test copy with headers | F-023 | 30m |

### 3.5 E2E Tests

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-070 | E2E: Connection CRUD | Create, list, edit, delete, test connection | F-152 | 2h |
| T-071 | E2E: Query execution | Write SQL, execute, view results | F-152 | 2h |
| T-072 | E2E: Grid CRUD | Edit cell, add row, delete row, verify DB changes | F-152 | 2h |
| T-073 | E2E: Schema browsing | Expand tree, view table details, view DDL | F-152 | 1h |
| T-074 | E2E: Export | Execute query, export to CSV/JSON, verify file | F-152 | 1h |
| T-075 | E2E: Error handling | Execute invalid SQL, verify error displayed inline | F-152 | 1h |

## 4. Test Data

| # | Task | Detail | Depends | Est. |
|---|---|---|---|---|
| T-076 | Create test PostgreSQL database | Docker container with sample schema and data | T-001 | 1h |
| T-077 | Create test SQLite database | In-memory DB with sample schema and data | T-017 | 30m |
| T-078 | Create test fixtures | Connection configs, SQL queries, expected results | T-076 | 1h |

## 5. Coverage Targets

| Module | Target | Gate |
|---|---|---|
| `core/domain/` | 90% | `cargo tarpaulin` |
| `core/application/` | 85% | `cargo tarpaulin` |
| `core/ports/` | 80% | `cargo tarpaulin` |
| `infrastructure/` | 80% | `cargo tarpaulin` |
| `frontend/src/modules/` | 70% | `vitest --coverage` |
| `frontend/src/commons/` | 60% | `vitest --coverage` |
| E2E critical flows | 100% | `playwright test` |
