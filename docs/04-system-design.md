# 06 — DB Client — System Design

---

## 1. Context

OPASS Fab is Windows-first (`PROA800.exe` via COM/Excel). A native Ubuntu Python script exists (`.memory-bank/DDL_Ubuntu/`) but covers one operation, not a full DB client.

This blueprint proposes a **Tauri 2 desktop app** (Rust core + React WebView) for Ubuntu that provides the same capabilities as DBeaver/Beekeeper — built on OPASS Fab's existing Clean Architecture conventions.

**Why Tauri**: Rust core compiles DB logic safely; WebView is presentation only. Lightweight (~10 MB), native packaging, reuses OPASS FE stack (React/MUI/TanStack).

---

## 2. Technology Stack

| Layer | Technology | Version | Rationale |
|---|---|---|---|
| App shell | Tauri 2 | 2.x | Native Rust, thin bridge |
| DB — Postgres | `sqlx` + `tokio-postgres` | 0.7 + 0.7 | Async, compile-time query check |
| DB — SQLite | `rusqlite` | 0.30 | Local meta-store |
| SSH tunnel | `openssh` | 0.9 | Spawn binary, no C dep |
| Secret vault | `keyring` + `aes-gcm` | 0.16 + 0.10 | libsecret on Ubuntu |
| Local meta-store | `rusqlite` | 0.30 | Connections, history, settings |
| UI | React 19 + Vite + TypeScript | 19 + 5 + 5.x | Consistent with OPASS FE |
| Grid | `@tanstack/react-virtual` | 3.x | MIT license |
| Editor | Monaco | 0.44+ | Autocomplete, syntax highlight |
| State (server) | TanStack Query 5 | 5.x | Caching, optimistic update |
| State (client) | Zustand | 4.x | Persist, lightweight |
| DI | `DIContainer` (TS) | custom | Copy OPASS pattern |
| i18n | i18next + react-i18next | 23.x + 13.x | Consistent with OPASS |
| Routing | TanStack Router | 1.x | File-based, type-safe |
| Packaging | `tauri-bundler` | 2.x | `.deb` + AppImage |
| Lint (Rust) | `cargo clippy` + `rustfmt` | stable | CI gate |
| Lint (TS) | `eslint 9` + `prettier` | 9 + 3 | CI gate |
| Test (Rust) | `cargo test` + `tarpaulin` | stable | Coverage gate |
| Test (TS) | `vitest` + `playwright` | 1.x + 1.x | Unit + E2E |

---

## 3. High-Level Architecture

### 3.1 Container Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER WORKSTATION                          │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  WebView (React SPA)                                        │ │
│  │  ├── app/          composition root, providers              │ │
│  │  ├── commons/      DI container, stores, utils, components │ │
│  │  ├── modules/      feature modules (CO, QU, SC, DG, EX)    │ │
│  │  └── routes/       file-based routes (TanStack Router)     │ │
│  │                                                               │ │
│  │  tauri.invoke('command', args)  ↕  Tauri JS ↔ Rust bridge  │ │
│  │                                                               │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Tauri App (Rust binary) — thin controller                  │ │
│  │  ├── commands/connection.rs                                  │ │
│  │  ├── commands/query.rs                                       │ │
│  │  ├── commands/schema.rs                                      │ │
│  │  ├── commands/history.rs                                     │ │
│  │  └── commands/export.rs                                      │ │
│  │                                                               │ │
│  │  State: app.state::<QueryService>()                          │ │
│  │  State: app.state::<ConnectionService>()                     │ │
│  │  State: app.state::<SchemaService>()                         │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Core (Rust) — pure business logic, no framework dep        │ │
│  │  ├── domain/                                                   │ │
│  │  │   ├── connection.rs   ConnectionId, ConnectionConfig      │ │
│  │  │   ├── query.rs        QueryId, QueryResult, QueryError    │ │
│  │  │   ├── schema.rs       Schema, Table, Column, Index, FK    │ │
│  │  │   └── history.rs      QueryHistory, SavedQuery            │ │
│  │  ├── application/                                                    │ │
│  │  │   ├── connection_service.rs                                   │ │
│  │  │   ├── query_service.rs                                        │ │
│  │  │   ├── schema_service.rs                                       │ │
│  │  │   └── export_service.rs                                       │ │
│  │  └── ports/                                                        │ │
│  │      ├── db_connector.rs     trait DbConnector                    │ │
│  │      ├── secret_store.rs     trait SecretStore                    │ │
│  │      └── meta_store.rs       trait MetaStore                      │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Infrastructure (Rust) — adapters                            │ │
│  │  ├── db/                                                     │ │
│  │  │   ├── postgres_connector.rs    impl DbConnector           │ │
│  │  │   ├── sqlite_connector.rs      impl DbConnector           │ │
│  │  │   └── ssh_tunnel.rs            impl tunnel trait          │ │
│  │  ├── secret/                                                   │ │
│  │  │   └── keyring_vault.rs         impl SecretStore           │ │
│  │  └── meta/                                                       │ │
│  │      └── sqlite_meta_store.rs     impl MetaStore             │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Data Stores                                                  │ │
│  │  ├── PostgreSQL (remote DB) — primary data source            │ │
│  │  ├── SQLite (local meta-store) — config, history, settings  │ │
│  │  └── OS Keyring (GNOME Keyring / KDE Wallet) — credentials  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Layer Dependency Graph

```
tauri-app (commands)  ──→  core::application  ✅
core::application         ──→  core::domain        ✅
core::application         ──→  core::ports         ✅ (trait objects)
core::domain              ──→  *                     ❌ (zero dependency)
infrastructure            ──→  core::domain        ✅ (implements ports)
infrastructure            ──→  core::ports         ✅ (implements traits)
infrastructure            ──→  core::application   ❌
tauri-app                 ──→  infrastructure      ❌
tauri-app                 ──→  core::domain        ❌
```

---

## 4. Runtime Architecture

### 4.1 App Startup Sequence

```
1. OS launches db-client binary
2. Tauri runtime initializes
3. WebView loads index.html (bundled React app)
4. React app mounts: ReactDOM.createRoot(<App />)
5. App.tsx renders providers tree:
   ThemeProvider → QueryClientProvider → SnackbarProvider → ModalProvider → RouterProvider
6. app.module.ts calls bootstrapServices():
   - DIContainer.register_all()
   - Each service receives its ports via constructor injection
7. Tauri commands registered in app state:
   app.manage(QueryService::new(db_connector, meta_store))
   app.manage(ConnectionService::new(db_connector, meta_store))
   app.manage(SchemaService::new(db_connector, meta_store))
   app.manage(ExportService::new(db_connector))
8. Splash screen shown while services initialize
9. Splash screen dismissed
10. App ready for user interaction
```

### 4.2 Connection Lifecycle

```
1. User clicks "New Connection"
2. CO03001 form renders
3. User fills: name, host, port, database, user, password, ssl_mode, ssh_tunnel
4. User clicks "Test Connection"
5. tauri.invoke('test_connection', { config })
6. Tauri command: ConnectionService.test_connectivity(config)
7. Service: db_connector.connect(config) → if success → disconnect → return Ok(())
8. UI: green checkmark if OK, error toast if failed
9. User clicks "Save"
10. tauri.invoke('save_connection', { config })
11. Tauri command: ConnectionService.create(config)
12. Service: encrypt password via SecretStore → save to MetaStore
13. UI: connection appears in CO01001 list
14. User clicks connection in list
15. tauri.invoke('connect', { connection_id })
16. Tauri command: ConnectionService.connect(connection_id)
17. Service: get config from MetaStore → decrypt password → db_connector.connect(config)
18. Store ConnectionHandle in MetaStore (active connection)
19. UI: connection status shows "Connected"
```

### 4.3 Query Execution Sequence

```
1. User types SQL in Monaco editor (QU01001)
2. User presses Ctrl+Enter
3. QU01001 component calls queryService.execute(sql)
4. queryService.execute(sql):
   a. Gets active connection_id from Zustand store
   b. Calls tauri.invoke('execute_query', { connection_id, sql })
5. Tauri command execute_query:
   a. Validates: connection_id exists, sql non-empty
   b. Resolves QueryService from app.state()
   c. Calls query_service.execute(connection_id, sql)
6. query_service.execute(connection_id, sql):
   a. meta_store.get_connection(connection_id) → ConnectionHandle
   b. start = Instant::now()
   c. db_connector.query(handle, sql, []) → QueryResult
   d. duration_ms = start.elapsed().as_millis()
   e. QueryResult { duration_ms, ..result }
   f. meta_store.save_query_history(connection_id, sql, &result)
   g. Ok(result)
7. Tauri command maps QueryResult → QueryResultDto
8. Result<QueryResultDto, DbErrorDto> returned to FE
9. TanStack Query cache updated
10. QU01001 component re-renders with new data
```

### 4.4 Streaming Sequence (large results > 10k rows)

```
1. query_service.execute() detects result set > 10k rows
2. Rust core starts streaming:
   for batch in result.stream(batch_size=1000) {
       app.emit_all("query-result-batch", batch);
   }
3. app.emit_all("query-done", QueryDone { total_rows, duration_ms });
4. FE subscribes to "query-result-batch" via tauri::Event
5. Each batch appended to TanStack Query cache
6. Virtualized grid renders new rows incrementally
7. On error: app.emit_all("query-error", error)
```

---

## 5. Configuration Management

### 5.1 Config Types

| Config type | Location | Format | Persistence |
|---|---|---|---|
| App settings (theme, language, default connection) | SQLite meta-store | JSON in `settings` table | Persistent |
| Connection configs | SQLite meta-store | Encrypted JSON in `connections` table | Persistent |
| Tauri config | `tauri.conf.json` | JSON (build-time) | Immutable |
| Feature flags | SQLite meta-store | JSON in `feature_flags` table | Persistent |
| Environment variables | `.env` or OS env | Key=value | Runtime only |

### 5.2 Config Loading Flow

```
1. App starts
2. Load tauri.conf.json (build-time config, immutable)
3. Open SQLite meta-store (runtime config)
4. Load settings from `settings` table
5. Load connections from `connections` table
6. Load feature flags from `feature_flags` table
7. Merge: runtime config overrides build-time config
8. Apply config to providers:
   - ThemeProvider: theme from settings
   - QueryClientProvider: staleTime=5s, retry=3
   - i18next: locale from settings
9. DI container registers all services with config
10. App ready
```

---

## 6. Logging & Observability

### 6.1 Rust Core Logging

| Concern | Approach |
|---|---|
| Structured logging | `tracing` crate with JSON output |
| Log levels | `ERROR` > `WARN` > `INFO` > `DEBUG` > `TRACE` |
| Log output | Console + local file (`~/.local/share/db-client/logs/`) |
| Log rotation | Weekly, max 10 files, max 10 MB each |
| Sensitive data filtering | Redact passwords, connection strings, query params |
| Context fields | `timestamp`, `level`, `target`, `connection_id`, `action_type` |

**Log format (JSON)**:
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

### 6.2 Frontend Logging

| Concern | Approach |
|---|---|
| Structured logging | Console + local file (Serilog-style) |
| Error tracking | `window.onerror` + React error boundary |
| Performance tracking | `performance.now()` for query duration |
| User action tracking | Log connect, query, export actions to local SQLite |

### 6.3 Audit Trail

Every action logged to local SQLite meta-store:

```
timestamp (UTC) | action_type (connect|query|export|edit|delete) | connection_id | success (bool) | duration_ms | error_message (if any)
```

---

## 7. Error Handling Strategy

### 7.1 Error Type Hierarchy (Rust)

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

### 7.2 Error Flow (End to End)

```
Rust DbError
  │
  ▼
Tauri command maps to DbErrorDto (JSON)
  │ DbErrorDto { error: "not_found", message: "db.connection_not_found", messageId: "DB01001", details: [] }
  │
  ▼
FE receives Dto via tauri.invoke() rejection
  │
  ▼
server-error-normalize.ts maps to { error, message, messageId, details }
  │
  ▼
server-error-translate.ts resolves messageId via i18n
  │ i18n.t("db.connection_not_found") → "Kết nối không tìm thấy"
  │
  ▼
UI shows user-facing message (toast / inline / dialog)
```

### 7.3 Error Envelope (standardized, same as OPASS)

```json
{
  "error": "validation | not_found | conflict | unauthorized | internal",
  "message": "User-facing message (i18n key)",
  "messageId": "DB01001",
  "details": []
}
```

### 7.4 UI Error States

| State | UI Response |
|---|---|
| Query error | Inline error on grid row, toast notification with messageId |
| Connection error | Connection editor shows error below form, retry button |
| Export error | Export dialog shows error message, retry button |
| Edit error (UPDATE) | Cell highlight red, error tooltip on hover |
| Delete error | Row highlight red, error toast |
| Add row error (INSERT) | New row highlight red, error tooltip |
| Timeout error | Toast: "Query timed out after Xms", cancel button |
| Connection lost | Toast: "Connection lost", reconnect button |

---

## 8. Testing Strategy

### 8.1 Rust Tests

| Test type | Location | Tool | Coverage target |
|---|---|---|---|
| Unit tests | `src/` inline `#[cfg(test)]` | `cargo test` | All public functions |
| Integration tests | `tests/` directory | `cargo test --test` | Trait implementations |
| Mock tests | `src/` with `mockall` | `cargo test` | Service layer with mock ports |
| Coverage | — | `cargo tarpaulin` | ≥ 80% on `core` |

**Mock example**:
```rust
// tests/query_service_test.rs
let mock_connector = MockDbConnector::new();
mock_connector.expect_query()
    .returning(|_, _, _| Ok(QueryResult::mock()));

let mock_store = MockMetaStore::new();
mock_store.expect_get_connection()
    .returning(|_| Ok(ConnectionHandle::mock()));

let service = QueryService::new(Box::new(mock_connector), Box::new(mock_store));
let result = service.execute(connection_id, "SELECT 1".to_string()).await;
assert!(result.is_ok());
```

### 8.2 TypeScript Tests

| Test type | Location | Tool | Coverage target |
|---|---|---|---|
| Unit tests | `modules/*/*.test.ts` | `vitest` | Services, hooks, utils |
| Component tests | `modules/*/*.test.tsx` | `vitest` + `@testing-library/react` | Component rendering + interaction |
| E2E tests | `e2e/` directory | `playwright` | Critical user flows |
| Coverage | — | `vitest --coverage` | ≥ 70% on `modules/` |

### 8.3 Test Pyramid

```
        /\       E2E tests (few, critical flows)
       /  \      ~5-10 tests
      /____\     Component tests (moderate)
     /______\    ~20-30 tests per module
    /________\   Unit tests (many, all functions)
   /__________\  ~100+ tests per module
```

---

## 9. Deployment

### 9.1 Targets

| Target | Tool | Output |
|---|---|---|
| `.deb` (Debian/Ubuntu) | `tauri-bundler` + `dpkg` | `db-client_1.0.0_amd64.deb` |
| `.AppImage` | `tauri-bundler` + `appimagetool` | `db-client-1.0.0.AppImage` |
| Flatpak (future) | `flatpak-builder` | `com.opassfab.DbClient` |

### 9.2 Ubuntu Dependencies

All available on Ubuntu 22.04+ repos:
- `webkit2gtk` — Tauri default WebView
- `libsecret` — keyring (secret storage)
- `openssh` — SSH tunnel

### 9.3 Build Pipeline

```
1. Checkout code
2. Install Rust toolchain (stable)
3. Install Node.js (via nvm)
4. cargo build --release (Rust core + Tauri shell)
5. npm run build (React frontend, Vite)
6. tauri build (bundle .deb + AppImage)
7. Run tests (cargo test + vitest + playwright)
8. Sign artifacts (Phase 3+)
9. Publish to release channel
```

### 9.4 CI/CD (GitHub Actions)

```yaml
# .github/workflows/build.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - checkout
      - install rust + node
      - cargo build --release
      - npm run build
      - tauri build
      - cargo test
      - vitest run
      - playwright test
      - cargo tarpaulin (coverage gate)
      - cargo deny (dep check)
      - madge (cycle check)
```

---

## 10. Non-Functional Requirements

| Attribute | Target | Approach |
|---|---|---|
| **Availability** | 99.9% for active sessions | Connection health check + auto-reconnect |
| **Performance** | < 3s cold start | Tauri fast startup, pre-built binary |
| **Performance** | < 500ms for < 1k row query | Async execution, connection pooling |
| **Performance** | < 5s first batch for > 10k rows | Streaming via Tauri events |
| **Scalability** | Handle 100k+ rows | Virtualized grid + streaming |
| **Scalability** | Handle 500+ tables per schema | Cached introspection + lazy loading |
| **Security** | No plaintext passwords | AES-256-GCM + OS keyring |
| **Security** | No SQL injection | Parameterized queries only |
| **Security** | Sandboxed WebView | Tauri 2 default |
| **Observability** | Structured logs | `tracing` JSON + local file |
| **Observability** | Audit trail | SQLite meta-store |
| **Maintainability** | Clean Architecture layering | Domain ↔ Application ↔ Infrastructure |
| **Maintainability** | Screen-ID naming convention | `[FeatureCode][5 digits]` |
| **i18n** | ja + en | i18next |
| **Theme** | Dark + light | MUI theme |

---

## 11. MVP Roadmap

**Phase 1 — Core**: PostgreSQL connection, schema browser, SQL editor, execute + grid (CRUD), export CSV/JSON/Excel, query history, credential vault, `.deb` packaging.

**Phase 2**: SSH tunnel, EXPLAIN ANALYZE visualisation, inline cell edit, saved queries/workspaces, DDL editor (create/edit/delete table), import CSV/Excel.

**Phase 3**: ERD diagram, multi-tab editor, theme per connection, Flatpak.

---

## 12. Open Decisions

| # | Decision | Recommendation |
|---|---|---|
| 1 | Monaco vs CodeMirror 6 | Monaco |
| 2 | MUI X DataGrid Pro vs TanStack Table + virtual | TanStack Table + virtual (MIT) |
| 3 | `ssh2` vs `openssh` crate | `openssh` |
| 4 | Meta-store schema | Design before coding |
| 5 | Feature code prefix | `CO/QU/SC/DG/EX` (2-char) |
| 6 | Password encryption key source | OS keyring only |
| 7 | `.deb` GPG signing | MVP: no sign |
| 8 | CI/CD Ubuntu runner | Needed before Phase 1 |