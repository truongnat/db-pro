# DB Pro

`DB Pro` is a desktop SQL client built with `Tauri 2 + Rust + React`.
The current baseline focuses on MVP foundations with a modern macOS-inspired UI.

## Implemented MVP baseline

- Connection manager UI and state (SQLite, PostgreSQL, MySQL profiles)
- Auto-bootstrap `Sample SQLite` connection + seeded demo dataset on first launch
- Connection persistence (`connections.json` in app data directory)
- Password storage in system keychain (not plaintext in JSON)
- Test connection command (`SELECT 1`)
- SQL editor panel with run action
- Result grid for row queries
- Schema navigator / object tree (schemas, tables, views, columns)
- Auto-refresh schema navigator after DDL queries (`CREATE/ALTER/DROP/TRUNCATE/RENAME`)
- Non-row query execution with affected-row summary
- Status bar + lightweight workflow for daily querying

## Tech stack

- Frontend: React + TypeScript + Vite
- Desktop shell: Tauri 2
- Backend: Rust + SQLx Any driver

## Run locally

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Next milestones

- Virtualized result grid for very large result sets
- Import/export wizard (CSV/JSON/XLSX)
- ER diagram view
- Visual query builder + multi-tab SQL workspace
# db-pro
