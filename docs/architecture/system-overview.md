# System Overview

> Status: **Implemented**

## Architecture

DB Pro is a Tauri desktop application with a Rust backend and React/TypeScript frontend.

```text
┌──────────────────────────────────────────────────────┐
│  Frontend (React / TypeScript / TanStack Router)     │
│  PATCH 1 owns all product UI surfaces                │
├──────────────────────────────────────────────────────┤
│  Tauri Command Boundary (dto.rs → CommandError)      │
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
| `db-pro-tauri` | `crates/tauri-app` | Tauri commands, DTOs, cancel/execution registry |

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
Tauri command (dto.rs)
  → parse input, validate
  → call application service
    → get ConnectionHandle from registry
    → call port trait (DbConnector)
      → infrastructure implementation (postgres/sqlite)
    → map result to domain types
  → map to DTO
  → return to frontend
```

## Error Flow

```text
sqlx/rusqlite error
  → infrastructure/error.rs (from_sqlx/from_rusqlite)
  → domain::DbError (typed taxonomy)
  → dto::CommandError (transport: code, message_id, retryable)
  → frontend (structured error, no raw SQL string parsing)
```

## Source Paths

- Domain: `crates/core/src/domain/`
- Application: `crates/core/src/application/`
- Ports: `crates/core/src/ports/`
- Infrastructure: `crates/infrastructure/src/`
- Tauri commands: `crates/tauri-app/src/commands/`
- DTOs: `crates/tauri-app/src/dto.rs`
