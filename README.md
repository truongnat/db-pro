# DB Pro

A native desktop Database IDE for PostgreSQL and SQLite, built with Tauri 2, Rust, React, and TypeScript.

DB Pro focuses on the core desktop database workflow: connect, explore, open resources, write SQL, inspect results, edit table data safely, and keep workspace context across tabs.

## Current status

**0.1.0 Release Candidate — not yet release-signed-off.**

The intended 0.1.0 feature scope is largely implemented, but the project still requires current-HEAD automated verification, cross-platform Tauri artifacts, and manual desktop smoke before tagging.

See:
- [`plans/07-current-status.md`](plans/07-current-status.md)
- [`docs/release/0.1.0-readiness.md`](docs/release/0.1.0-readiness.md)
- [`docs/release/0.1.0-release-checklist.md`](docs/release/0.1.0-release-checklist.md)

## Features

### Connection management

- PostgreSQL and SQLite connections
- Create, test, connect, disconnect, reconnect, edit, and delete flows
- Secure credential storage with OS keyring integration and encrypted fallback support
- Startup reconnect source flow for previously active connections
- SQLite native file picker wiring
- SSH tunnel plumbing exists, but is not yet qualified end-to-end across all release targets

### SQL editor

- Monaco editor with SQL syntax highlighting
- Schema-aware completion foundation
- Multi-tab query workspace
- SQL formatting
- Query history

### Query execution

- Current-statement execution
- Selection execution
- Run-all / multi-statement execution
- Split Run button for direct execution + options
- Query cancellation via Stop / Escape flow
- Explain
- Result grid with row/timing metadata

### Schema browsing

- Database/object Explorer tree
- Schemas, tables, and views
- Targeted metadata refresh
- Data-first table/view navigation
- DB Object workbench: Data, Columns, Indexes, Relations, DDL inspection

### Data grid

- Virtualized data grid
- Filtering, sorting, pagination
- Column resize and persisted layout state
- Row selection and scoped keyboard copy
- Copy cell / row / column-name actions
- Inline PK-based staged row updates
- Staged row deletes
- Patch-style updates that only send changed columns
- Same-row multi-cell patch composition
- Revision-safe partial success/failure handling
- Clear read-only behavior for tables without a primary key

### Workspace

- Query and DB Object tabs
- Preview / permanent / pinned tabs
- Compact inactive pinned tabs
- Collision-aware object titles
- Orphan-tab recovery
- Workspace persistence
- Quick Open (`Cmd/Ctrl+P`)
- Command Palette (`Cmd/Ctrl+Shift+P`)

### Agent workspace (Preview)

- Agent-ready Action Platform architecture
- Typed execution lifecycle, confirmation gates, cancellation identity, and audit hooks
- Preview UI only in 0.1.0
- MCP server is not shipped in 0.1.0

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│  Frontend (React / TypeScript / TanStack Router)     │
│  Monaco · Zustand · TanStack Query · shadcn/ui       │
├──────────────────────────────────────────────────────┤
│  Action Platform / Workspace / Query Runtime         │
├──────────────────────────────────────────────────────┤
│  Tauri Command Boundary (DTO → structured error)     │
├──────────────────────────────────────────────────────┤
│  Application Layer                                   │
│  Query · Connection · Schema · TableData · Export    │
├──────────────────────────────────────────────────────┤
│  Domain Layer                                        │
│  query · schema · connection · execution · safety    │
├──────────────────────────────────────────────────────┤
│  Ports / Infrastructure                              │
│  PostgreSQL · SQLite · secrets · metadata · SSH      │
└──────────────────────────────────────────────────────┘
```

### Crate layout

| Crate | Path | Responsibility |
|---|---|---|
| `db-pro-core` | `crates/core` | Domain types, application services, port traits |
| `db-pro-infrastructure` | `crates/infrastructure` | PostgreSQL, SQLite, metadata, secrets, SSH plumbing |
| `db-pro-tauri` | `crates/tauri-app` | Tauri commands, DTOs, runtime registries |

### Tech stack

**Backend:** Rust, Tauri 2, sqlx/PostgreSQL, rusqlite/SQLite, keyring, AES-GCM, Argon2  
**Frontend:** React 19, TypeScript, Vite, TanStack Router/Query/Virtual, Zustand, Monaco Editor, shadcn/ui, Radix UI, Tailwind CSS 4, i18next  
**Testing:** Vitest + Rust unit/integration tests  
**CI/Release:** GitHub Actions with macOS / Windows / Linux release matrix

## Development

### Prerequisites

- Rust toolchain from `rust-toolchain.toml`
- Node.js 22+
- npm 10+
- Tauri system prerequisites for the host OS

### Setup

```bash
git clone https://github.com/truongnat/db-pro.git
cd db-pro

cd frontend
npm ci
npm run dev
```

In another terminal:

```bash
cargo tauri dev
```

### Frontend quality gates

```bash
cd frontend
npm ci
npm run typecheck
npm run lint
npm run format:check
npm run test
npm run build
```

### Rust quality gates

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Tauri build

```bash
cargo tauri build
```

Configured release formats include macOS DMG/app bundle, Windows MSI/NSIS, and Linux DEB/AppImage/RPM.

## Known limitations for 0.1.0

- Complete row insertion workflow is not shipped.
- Grid update/delete requires a primary key; no-PK tables are read-only.
- Advanced schema mutation/DDL execution UI is deferred; inspection is available.
- Users/roles workbench is deferred.
- Agent workspace is Preview only.
- MCP server is not included.
- SSH tunnel plumbing is not yet end-to-end qualified across all target platforms.
- Release artifacts are unsigned unless signing/notarization is added before distribution.
- Only PostgreSQL and SQLite are supported.
- Project license is not yet defined.

## Release readiness

Do not tag `v0.1.0` until the current release candidate SHA has:

1. fully green frontend tests;
2. green `format:check`;
3. green frontend + Rust quality gates;
4. successful macOS, Windows, and Linux Tauri release builds;
5. completed manual runtime smoke.

## Repository structure

```text
crates/                     Rust workspace
  core/                     Domain, application services, ports
  infrastructure/           Database drivers, secrets, metadata
  tauri-app/                Tauri entry point, commands, DTOs
frontend/                   React/TypeScript application
  src/
    commons/                Shared stores, actions, components, services
    modules/                Query, schema, data-grid, connection, export, etc.
    components/ui/          shadcn/ui primitives
    routes/                 TanStack Router routes
docs/                       Architecture and release documentation
plans/                      Implementation plans + current status
.github/workflows/          CI and release pipelines
fixtures/                   Database test fixtures
```

## License

Not defined yet.
