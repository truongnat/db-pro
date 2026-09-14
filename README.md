# DB Pro

A native desktop Database IDE for PostgreSQL and SQLite, built in Rust with an `egui`/`eframe`
UI. There is no WebView, no Node runtime, and no web build. The shipped binary is
`db-pro-native`.

DB Pro focuses on the core desktop database workflow: connect, explore, open resources, write
SQL, inspect results, edit table data safely, and keep workspace context across tabs.

## Current status

**0.1.0 Release Candidate — not yet release-signed-off.**

- Internal / private release-candidate qualification: **YES**
- Public distribution: **NO** — the project license is undecided (see
  [Governance](#governance--license))

Candidate SHA `fbf9fdab9100f08f12e29434983f32c18f14ac2f`; exact-HEAD quality gates are green on
the pinned toolchain (rustc 1.95.0) with `cargo test --workspace` = 811 passed / 0 failed /
19 ignored. Runtime smoke of the candidate is not complete (V01-01/02/04/05 `EVIDENCE_GAP`,
V01-03 `PARTIAL`), and cross-platform artifacts are pending.

See:
- [`docs/release/0.1.0-release-notes.md`](docs/release/0.1.0-release-notes.md)
- [`docs/release/0.1.0-readiness.md`](docs/release/0.1.0-readiness.md)
- [`docs/release/0.1.0-handoff.md`](docs/release/0.1.0-handoff.md)
- [`plans/07-current-status.md`](plans/07-current-status.md)

## Available now (0.1.0)

Everything in this section exists in the shipping code. Items marked *(Preview)* are labelled
as such in the UI.

- **Native application** — single Rust binary (`eframe`/`egui`), fonts compiled in, SQLite
  bundled, no runtime asset downloads.
- **PostgreSQL and SQLite** — connect, test, connect/disconnect/reconnect, edit, delete;
  credentials in the OS keyring (encrypted-file fallback); secrets stored separately from
  connection metadata; SQLite gets a native file picker.
- **Connection Explorer** — schema/table/view tree, targeted refresh, preview on single click,
  promote to a permanent tab on double click.
- **Query Editor** — syntax highlighting, schema-aware completion, current-statement / selection
  / run-all execution, multi-statement results as separate tabs plus a `Messages` tab,
  diagnostics, SQL formatting, query history, local drafts.
- **Table Data Editor** — virtualized grid, filter/sort/pagination, column resize with persisted
  layout, row selection and copy, staged insert/update/delete with PK targeting, pending-change
  review, three-way conflict resolution, patch-style updates.
- **Schema workbench** — Data / Columns / Indexes / Relations / Triggers / DDL inspection with
  reconstructed DDL and FK navigation.
- **ER Diagram** — schema-level canvas with spatial index, three-tier LOD, BFS neighbourhood
  exploration and a coalescing background layout worker.
- **Agent *(Preview)*** — Ask/Edit/Agent panel that produces reviewable SQL drafts; every
  DB-mutating tool call is confirmation-gated. Optional: set `GROQ_API_KEY` or
  `OPENAI_API_KEY` before launch (see [Agent provider](#agent-provider)).
- **Export / Backup (subset)** — CSV/TSV from the result grid; SQLite `VACUUM INTO` backup;
  PostgreSQL `pg_dump`/`pg_restore` (must be on `PATH`).
- **Safety** — backend-enforced read-only connections, destructive-SQL confirmation, atomic
  batch rollback, conflict resolution, no-PK tables read-only.

## Roadmap (not in 0.1.0)

These are planned, not shipped. Do not treat them as current capabilities. Details:
[`docs/notes/PRODUCT_ROADMAP.md`](docs/notes/PRODUCT_ROADMAP.md) and
[`docs/goals/goal-full-product.md`](docs/goals/goal-full-product.md).

- **v0.2** — Phase A: database Object CRUD workbench (tables, views, indexes, FKs, triggers,
  sequences, types); Phase B: routines (browse, source, execute with arguments).
- **v0.3** — Phase C: data transfer/import (CSV/JSON/Excel) and the Transfers activity;
  Phase D: monitoring (sessions, running queries, locks, sizes); Phase E: PostgreSQL users,
  roles and permissions UI.
- **Later** — Phase F: schema compare, DDL diff, migration apply; Phase G: snippet library,
  scratch SQL, favorites, keybinding editor; Phase H: advanced AI assistants (slow-query
  analysis, index suggestion, migration assistant).
- **Also deferred** — MCP server/client, autonomous (non-Preview) Agent execution, additional
  database drivers, installer formats (`.dmg`, MSI/NSIS, `.deb`/`.rpm`, AppImage), auto-update,
  macOS x86_64/universal builds and code signing/notarization. See
  [`docs/release/0.1.0-packaging.md`](docs/release/0.1.0-packaging.md).

## UI direction

The product UI is **native Rust (`eframe` + `egui`)** and is the only UI under active
development:

- `crates/ui` (`db-pro-ui`) — shell, views, `AppState`, reducer, task bridge, theme
- `crates/native-app` (`db-pro-native`) — the shipped desktop binary
- `crates/runtime` (`db-pro-runtime`) — bootstrap, services, worker/event bridge

The earlier React 19 / TypeScript / Vite frontend, which ran inside a Tauri 2 system WebView,
was **archived on 2026-09-11** under [`_archive/frontend/`](_archive/README.md). It is reference
material for parity comparison only — it is not built, tested, or packaged.

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

The UI never calls a database driver directly. Every user intent becomes a typed `UiCommand`
handled off the UI thread by the runtime worker, which replies with `UiEvent`s that the reducer
applies. See `docs/10-egui-native-migration-plan.md`.

| Crate | Path | Responsibility |
|---|---|---|
| `db-pro-core` | `crates/core` | Domain types, application services, port traits |
| `db-pro-infrastructure` | `crates/infrastructure` | PostgreSQL, SQLite, metadata, secrets, SSH plumbing |
| `db-pro-runtime` | `crates/runtime` | Bootstrap, service wiring, worker/event bridge, cancellation |
| `db-pro-ui` | `crates/ui` | egui shell, views, `AppState`, reducer, theme |
| `db-pro-native` | `crates/native-app` | The shipped native desktop binary |
| `db-pro-tauri` | `crates/tauri-app` | **Legacy** transitional Tauri host; scheduled for removal |

**Tech stack:** Rust, `eframe`/`egui`, native GL rendering, `sqlx`/PostgreSQL,
`rusqlite`/SQLite (bundled), `keyring`, AES-GCM/Argon2, Criterion benchmarks; CI on
macOS/Windows/Linux.

### Agent provider

Set `GROQ_API_KEY` or `OPENAI_API_KEY` outside the repository before launching `db-pro-native`.
Groq defaults to `openai/gpt-oss-120b`; `DB_PRO_GROQ_MODEL`, `DB_PRO_GROQ_ENDPOINT`,
`DB_PRO_CODEX_MODEL`, and `DB_PRO_CODEX_ENDPOINT` are optional overrides. Credentials are read
once by the runtime worker and are not rendered in the UI or logs. With no key set, the Agent
panel has no provider and generates nothing.

## Development

### Prerequisites

- Rust toolchain from `rust-toolchain.toml` (pinned to **1.95.0**)
- Native windowing/GL development headers
  - macOS: Xcode command line tools
  - Linux: `libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev`; at runtime a D-Bus
    Secret Service provider (gnome-keyring/KWallet) is required for credential storage
  - Windows: MSVC build tools

No Node.js, pnpm, or WebView runtime is required.

### Run the app

```bash
git clone https://github.com/truongnat/db-pro.git
cd db-pro
cargo run -p db-pro-native
```

Runtime state is written to `./.db-pro-data` by default (override with
`DB_PRO_DATA_DIR=/some/path`). It holds your connection metadata (`meta.db`) and credential
store; it is gitignored and must never be copied into a release archive.

### Rust quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

PostgreSQL integration tests are `#[ignore]`d by default and require `DATABASE_URL`; they run
with `cargo test -p db-pro-infrastructure --test pg_integration -- --ignored`.

### Build a release binary

```bash
cargo build --release --locked -p db-pro-native
# -> target/release/db-pro-native
```

Release archives are produced by `.github/workflows/release.yml` (or locally with
`scripts/release/package-macos.sh`, `package-linux.sh`, `package-windows.ps1`,
`write-checksums.sh`). Installer formats are intentionally not implemented — see
[`docs/release/0.1.0-packaging.md`](docs/release/0.1.0-packaging.md).

### Legacy Tauri host (not part of the product)

`crates/tauri-app` still exists as a transitional host so the archived React frontend can be run
for parity comparison. It is not built by CI and is not shipped.

## Known limitations for 0.1.0

- Runtime smoke of this candidate has not been run to completion; V01-01/02/04/05 runtime
  evidence is missing (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`).
- Workspace/tab/settings persistence is not implemented in the native build, so tabs do not
  restore after restart.
- Query cancellation works for SQLite; PostgreSQL cancellation is capability-gated
  `Unsupported`.
- Row insert ships in the native Table Data Editor (staged insert) but is narrower than a full
  insert workflow: complex column types (JSON/array/UUID) lack complete input widgets (LIM-003).
- Grid update/delete requires a primary key; no-PK tables are read-only.
- Advanced schema mutation/DDL execution is limited to a confirmation-gated single-statement
  editor; richer migration workflows remain deferred.
- Users/roles workbench, Monitoring, Import and MCP are not included.
- The Agent panel **ships as Preview**; production/autonomous Agent execution is excluded.
- SSH tunnel plumbing is not end-to-end qualified across all platforms.
- `pg_dump`/`pg_restore` and `ssh` are not bundled and must be on `PATH`.
- Release artifacts are **unsigned** (macOS Gatekeeper and Windows SmartScreen will warn) and
  macOS x86_64 is not built.
- Only PostgreSQL and SQLite are supported.

## Governance / license

The project license is **not defined**: there is no `LICENSE` file and no `license`/`license-file`
key in any `Cargo.toml`. This is an open governance item that blocks public distribution — the
0.1.0 artifacts may be built and evaluated internally, but must not be presented as licensed for
public use, and the release archives deliberately contain no `LICENSE` file. See `R-LICENSE` in
[`docs/release/risk-register.md`](docs/release/risk-register.md).

## Release readiness

Do not tag `v0.1.0` until:

1. the candidate SHA has fully green Rust gates and a green release build;
2. cross-platform archives + `SHA256SUMS.txt` exist from a completed release run;
3. the packaged install smoke has been performed and recorded;
4. the runtime evidence gaps (V01-01…V01-05) are closed or explicitly accepted;
5. the license decision is made.

Steps, rollback and install-smoke procedure: [`docs/release/0.1.0-handoff.md`](docs/release/0.1.0-handoff.md).

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
scripts/release/            Release packaging scripts
.github/workflows/          CI and release pipelines
fixtures/                   Database test fixtures
```
