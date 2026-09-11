# System Overview

> Status: **Implemented**

## Architecture

DB Pro is a native desktop application built on a Rust workspace. The UI is native
`eframe`/`egui` (`crates/ui` + `crates/native-app`) and talks to the shared
`db-pro-runtime` service graph through a typed command/event task bridge.

The earlier React/TypeScript frontend and the Tauri WebView that hosted it were retired:
the frontend is archived under `_archive/frontend/`, its React-era ER benchmark harness
under `_archive/bench/`, and `crates/tauri-app` remains only as a legacy transitional host.
The native UI is the only UI under active development.

```text
┌──────────────────────────────────────────────────────┐
│  Native UI (egui / eframe, no WebView)               │
│  crates/ui owns all product UI surfaces              │
├──────────────────────────────────────────────────────┤
│  UiCommand / UiEvent Task Bridge                     │
│  (typed command → event, reducer-applied)            │
├──────────────────────────────────────────────────────┤
│  Application Layer (services)                        │
│  QueryService, ConnectionService, SchemaService,     │
│  TableDataService, ExportService, BackupService,     │
│  UserService, DataDiffService                        │
├──────────────────────────────────────────────────────┤
│  Domain Layer (core types, no I/O)                   │
│  error, query, schema, connection, execution,        │
│  capabilities, safety, secret, diagnostics, history  │
├──────────────────────────────────────────────────────┤
│  Ports (traits)                                      │
│  DbConnector, SecretStore, *Repository               │
├──────────────────────────────────────────────────────┤
│  Infrastructure                                      │
│  postgres/, sqlite/, meta/, secret/, ssh/, backup/   │
└──────────────────────────────────────────────────────┘
```

## Crate Layout

| Crate | Path | Responsibility |
|-------|------|----------------|
| `db-pro-core` | `crates/core` | Domain types, application services, port traits |
| `db-pro-infrastructure` | `crates/infrastructure` | PostgreSQL, SQLite, metadata store, secrets, SSH |
| `db-pro-runtime` | `crates/runtime` | Shared service graph and async runtime worker |
| `db-pro-ui` | `crates/ui` | Native egui shell, views, `AppState`, reducer, `DbProTheme` |
| `db-pro-native` | `crates/native-app` | Shipped native binary and task-bridge adapter |
| `db-pro-tauri` | `crates/tauri-app` | **Legacy** transitional Tauri host, not shipped; scheduled for removal at cutover |

## Database Drivers

| Driver | Status | Source |
|--------|--------|--------|
| PostgreSQL | Implemented | `infrastructure/postgres/` (sqlx) |
| SQLite | Implemented | `infrastructure/sqlite/` (rusqlite) |

## Key Domain Models

| Model | Path | Status |
|-------|------|--------|
| Error taxonomy | `domain/error.rs` | Implemented (P2-02) |
| Capabilities | `domain/capabilities.rs` | Implemented (P2-03) |
| Execution lifecycle | `domain/execution.rs` | Implemented (P2-04) |
| Safety policy | `domain/safety.rs` | Implemented (P2-08) |
| Credential boundary | `domain/secret.rs` | Implemented (P2-09) |
| Diagnostics | `domain/diagnostics.rs` | Implemented (P2-12) |

## Data Flow

```text
egui view
  → UiCommand sent over the task bridge (off the UI thread)
  → runtime worker parses/translates input, validates
  → call the shared `DbProRuntime` service graph
    → get ConnectionHandle from registry
    → call port trait (DbConnector)
      → infrastructure implementation (postgres/sqlite)
    → map result to domain types
  → map to UiEvent
  → reducer applies the event to `AppState`
  → UI repaints with refreshed state
```

## Error Flow

```text
sqlx/rusqlite error
  → infrastructure/error.rs (from_sqlx/from_rusqlite)
  → domain::DbError (typed taxonomy)
  → UiEvent::Failed { request_id, error } (transport: code, message_id, retryable)
  → reducer / UI surface (structured error, no raw SQL string parsing)
```

## Source Paths

- Domain: `crates/core/src/domain/`
- Application: `crates/core/src/application/`
- Ports: `crates/core/src/ports/`
- Infrastructure: `crates/infrastructure/src/`
- Runtime worker: `crates/runtime/src/`
- Native UI: `crates/ui/src/`
- Native binary entry: `crates/native-app/src/main.rs`
- Legacy Tauri commands (transitional only): `crates/tauri-app/src/commands/`
