# DB Pro

A native desktop database IDE for PostgreSQL and SQLite. Built with Tauri 2, Rust, and React.

DB Pro provides a DBeaver-like workflow — connection management, SQL editing with IntelliSense, query execution with cancellation, schema browsing, data grid, and export — in a lightweight, secure desktop application.

## Current release

**v0.1.0** — PostgreSQL and SQLite support. See [release notes](docs/release/0.1.0-release-notes.md).

## Features

### Connection management
- PostgreSQL and SQLite connections
- Secure credential storage with OS keyring integration
- SSH tunnel support
- Connection testing and health monitoring

### SQL editor
- Monaco editor with SQL syntax highlighting
- Auto-completion from live schema (tables, columns, keywords)
- Multi-tab query workspace with keyboard navigation
- SQL formatting and diagnostics

### Query execution
- Run full query or selection (Ctrl/Cmd+Enter, Shift+Enter, F5)
- Execution cancellation with proper cleanup
- Explain plan visualization
- Query history with re-run
- Result grid with virtualized rendering

### Schema browsing
- Database/object explorer tree
- Table details: columns, indexes, constraints, foreign keys
- DDL inspection and generation
- Multi-database and schema switching

### Data grid
- Editable data grid with inline cell editing
- Staged changes with commit/rollback
- Filtering and sorting
- CSV, JSON, and Excel export

### Agent workspace (Preview)
- Agent-native architecture for future AI-assisted workflows
- Action Platform with typed execution lifecycle, confirmation gates, and cancellation support
- MCP integration is not shipped in 0.1.0

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│  Frontend (React / TypeScript / TanStack Router)     │
│  Monaco · Zustand · TanStack Query · shadcn/ui       │
├──────────────────────────────────────────────────────┤
│  Tauri Command Boundary (dto.rs → CommandError)      │
├──────────────────────────────────────────────────────┤
│  Application Layer (services)                        │
│  QueryService · ConnectionService · SchemaService    │
│  TableDataService · ExportService · BackupService    │
├──────────────────────────────────────────────────────┤
│  Domain Layer (core types, no I/O)                   │
│  error · query · schema · connection · execution     │
│  capabilities · safety · secret · diagnostics        │
├──────────────────────────────────────────────────────┤
│  Ports (traits)                                      │
│  DbConnector · SecretStore · *Repository             │
├──────────────────────────────────────────────────────┤
│  Infrastructure                                      │
│  postgres/ · sqlite/ · meta/ · secret/ · ssh/        │
└──────────────────────────────────────────────────────┘
```

### Crate layout

| Crate | Path | Responsibility |
|-------|------|----------------|
| `db-pro-core` | `crates/core` | Domain types, application services, port traits |
| `db-pro-infrastructure` | `crates/infrastructure` | PostgreSQL, SQLite, metadata store, secrets, SSH |
| `db-pro-tauri` | `crates/tauri-app` | Tauri commands, DTOs, cancel/execution registry |

### Tech stack

**Backend:** Rust (edition 2021), Tauri 2, sqlx (PostgreSQL), rusqlite (SQLite), keyring, AES-GCM encryption, Argon2  
**Frontend:** React 19, TypeScript 5, Vite 6, TanStack Router, TanStack Query, Zustand, Monaco Editor, shadcn/ui, Radix UI, Tailwind CSS 4, i18next, Recharts  
**Testing:** Vitest (1000 frontend tests), cargo test (17 Rust tests)  
**CI:** GitHub Actions with release matrix (macOS / Windows / Linux)

## Development

### Prerequisites

- Rust 1.77.2+ (see `rust-toolchain.toml`)
- Node.js 22+
- npm 10+
- System dependencies for Tauri (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

### Setup

```bash
# Clone
git clone https://github.com/truongnat/db-pro.git
cd db-pro

# Frontend
cd frontend
npm ci
npm run dev          # starts Vite dev server on :5174

# In another terminal — Tauri dev
cargo tauri dev
```

### Frontend commands

```bash
npm run dev            # Vite dev server
npm run build          # Production build (route gen + typecheck + vite build)
npm run typecheck      # TypeScript check (tsc --noEmit)
npm run lint           # ESLint
npm run format:check   # Prettier check
npm run test           # Vitest (all tests)
npm run test:watch     # Vitest watch mode
```

### Rust commands

```bash
cargo fmt --all --check                        # Format check
cargo check --workspace                        # Type check
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo test --workspace                         # Test
```

### Tauri build

```bash
# Development
cargo tauri dev

# Release build (runs frontend build automatically)
cargo tauri build
```

Release artifacts: DMG (macOS), MSI/NSIS (Windows), DEB/AppImage/RPM (Linux).

## Known limitations (0.1.0)

- Data CRUD (INSERT/UPDATE/DELETE via grid) — not wired to backend
- Schema DDL editing — read-only inspection, no live ALTER
- MCP integration — not shipped
- Agent workspace — Preview status, not production-ready
- No code signing on release artifacts
- SSH tunnel — implemented but not end-to-end tested in CI

## Repository structure

```text
crates/                     Rust workspace
  core/                     Domain, application services, ports
  infrastructure/           Database drivers, secrets, metadata
  tauri-app/                Tauri entry point, commands, DTOs
frontend/                   React/TypeScript application
  src/
    commons/                Shared stores, actions, components, services
    modules/                Feature modules (query, schema, data-grid, connection, export, backup)
    components/ui/          shadcn/ui primitives
    routes/                 TanStack Router file-based routes
docs/                       Architecture and release documentation
plans/                      Implementation plan and task tracking
.github/workflows/          CI and release pipelines
fixtures/                   PostgreSQL and SQLite test fixtures
```

## License

Not defined yet.
