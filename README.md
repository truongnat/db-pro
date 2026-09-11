# DB Pro

A native desktop Database IDE for PostgreSQL and SQLite, built in Rust with a native
`egui`/`eframe` UI. There is no WebView, no Node runtime, and no web build.

DB Pro focuses on the core desktop database workflow: connect, explore, open resources,
write SQL, inspect results, edit table data safely, and keep workspace context across tabs.

## Current status

**0.1.0 Release Candidate — not yet release-signed-off.**

The intended 0.1.0 feature scope is largely implemented on the native UI, but the project
still requires current-HEAD automated verification, cross-platform native artifacts, and
manual desktop smoke before tagging.

See:
- [`plans/07-current-status.md`](plans/07-current-status.md)
- [`docs/release/0.1.0-readiness.md`](docs/release/0.1.0-readiness.md)
- [`docs/release/0.1.0-release-checklist.md`](docs/release/0.1.0-release-checklist.md)

## UI direction

The product UI is **native Rust (`eframe` + `egui`)** and is the only UI under active
development:

- `crates/ui` (`db-pro-ui`) — shell, views, `AppState`, reducer, task bridge, `DbProTheme`
- `crates/native-app` (`db-pro-native`) — the shipped desktop binary
- `crates/runtime` (`db-pro-runtime`) — bootstrap, services, worker/event bridge

The earlier React 19 / TypeScript / Vite frontend, which ran inside a Tauri 2 system
WebView, was **archived on 2026-09-11** under [`_archive/frontend/`](_archive/README.md).
It is reference material for parity comparison only — it is not built, tested, or
packaged, and it is not a fallback UI.

The rationale, target architecture, phases, and visual acceptance gates are in
[`docs/10-egui-native-migration-plan.md`](docs/10-egui-native-migration-plan.md).

## Features

### Connection management

- PostgreSQL and SQLite connections
- Create, test, connect, disconnect, reconnect, edit, and delete flows
- Secure credential storage with OS keyring integration and encrypted fallback support
- Startup reconnect source flow for previously active connections
- SQLite native file picker wiring
- SSH tunnel plumbing exists, but is not yet qualified end-to-end across all release targets

### SQL editor

- Native SQL editor with syntax highlighting
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

- Native virtualized data grid
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
- Optional Responses text provider via `GROQ_API_KEY` or `OPENAI_API_KEY`; SQL remains a reviewable draft
- MCP server is not shipped in 0.1.0

To enable the optional native AI provider, set `GROQ_API_KEY` or `OPENAI_API_KEY`
outside the repository before launching `db-pro-native`. Groq defaults to
`openai/gpt-oss-120b`; `DB_PRO_GROQ_MODEL`, `DB_PRO_GROQ_ENDPOINT`,
`DB_PRO_CODEX_MODEL`, and `DB_PRO_CODEX_ENDPOINT` are optional overrides.
Credentials are read once by the
runtime worker and are not rendered in the UI or logs.

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│  Native UI (egui / eframe, no WebView)               │
│  shell · panels · tabs · dialogs · grid · editor     │
├──────────────────────────────────────────────────────┤
│  UiCommand / UiEvent task bridge + reducer           │
│  AppState · workspace · query · grid · agent state   │
├──────────────────────────────────────────────────────┤
│  Runtime worker (db-pro-runtime)                     │
│  bootstrap · registries · service wiring · cancel    │
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

The UI never calls a database driver directly. Every user intent becomes a typed
`UiCommand` handled off the UI thread by the runtime worker, which replies with
`UiEvent`s that the reducer applies. See `docs/10-egui-native-migration-plan.md`.

### Crate layout

| Crate | Path | Responsibility |
|---|---|---|
| `db-pro-core` | `crates/core` | Domain types, application services, port traits |
| `db-pro-infrastructure` | `crates/infrastructure` | PostgreSQL, SQLite, metadata, secrets, SSH plumbing |
| `db-pro-runtime` | `crates/runtime` | Bootstrap, service wiring, worker/event bridge, cancellation |
| `db-pro-ui` | `crates/ui` | egui shell, views, `AppState`, reducer, theme |
| `db-pro-native` | `crates/native-app` | The shipped native desktop binary |
| `db-pro-tauri` | `crates/tauri-app` | **Legacy** transitional Tauri host; scheduled for removal |

### Tech stack

**UI:** Rust, `eframe` / `egui` 0.29, native GL rendering, `lucide-icons`  
**Backend:** Rust, `sqlx`/PostgreSQL, `rusqlite`/SQLite, `keyring`, AES-GCM, Argon2  
**Testing:** Rust unit/integration tests, Criterion benchmarks  
**CI/Release:** GitHub Actions with macOS / Windows / Linux native build matrix

## Development

### Prerequisites

- Rust toolchain from `rust-toolchain.toml`
- Native windowing/GL development headers for your OS
  - macOS: Xcode command line tools
  - Linux: `libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev`
  - Windows: MSVC build tools

No Node.js, pnpm, or WebView runtime is required.

### Run the app

```bash
git clone https://github.com/truongnat/db-pro.git
cd db-pro
cargo run -p db-pro-native
```

Runtime state is written to `./.db-pro-data` by default. Override it with
`DB_PRO_DATA_DIR=/some/path cargo run -p db-pro-native`.

### Rust quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Build a release binary

```bash
cargo build --release --locked -p db-pro-native
# -> target/release/db-pro-native
```

Installer/packaging formats (DMG, MSI/NSIS, DEB/RPM/AppImage) are not implemented yet
for the native app. See `docs/release/0.1.0-packaging.md`.

### Legacy Tauri host (not part of the product)

`crates/tauri-app` still exists as a transitional host so the archived React frontend can
be run for parity comparison. It is not built by CI and is not shipped. See
`crates/tauri-app/README.md` and `_archive/README.md`.

## Known limitations for 0.1.0

- Complete row insertion workflow is not shipped.
- Grid update/delete requires a primary key; no-PK tables are read-only.
- Advanced schema mutation/DDL execution is limited to a confirmation-gated single-statement editor; richer migration workflows remain deferred.
- Users/roles workbench is deferred.
- Agent workspace is Preview only.
- MCP server is not included.
- SSH tunnel plumbing is not yet end-to-end qualified across all target platforms.
- Release artifacts are unsigned unless signing/notarization is added before distribution.
- Only PostgreSQL and SQLite are supported.
- A development default connection is seeded on launch (`"Xe Lạc Hồng (PostgreSQL)"`,
  `localhost:5432/fullstack_starter`, `postgres`/`postgres`), and the New connection dialog
  opens pre-filled with those values. This is a developer convenience carried over from
  `feature/sidebar-dbeaver-codex-layout`, not a product default — see
  `crates/ui/src/runtime.rs` (`impl Default for UiConnectionDraft`) and
  `crates/native-app/src/main.rs`.
- Project license is not yet defined.

## Release readiness

Do not tag `v0.1.0` until the current release candidate SHA has:

1. fully green Rust tests;
2. green `cargo fmt --check` and Clippy;
3. a successful `cargo build --release --locked -p db-pro-native` on macOS, Windows, and Linux;
4. completed manual runtime smoke on the native app.

## Repository structure

```text
crates/                     Rust workspace
  core/                     Domain, application services, ports
  infrastructure/           Database drivers, secrets, metadata
  runtime/                  Bootstrap, service wiring, worker/event bridge
  ui/                       Native egui UI: shell, views, state, theme
  native-app/               db-pro-native binary (shipped)
  tauri-app/                Legacy transitional Tauri host (not shipped)
_archive/
  frontend/                 Archived React/Vite UI (reference only)
  bench/                    Archived React-era ER renderer benchmarks (reference only)
docs/                       Architecture and release documentation
plans/                      Implementation plans + current status
.github/workflows/          CI and release pipelines
fixtures/                   Database test fixtures
```

## License

Not defined yet.
