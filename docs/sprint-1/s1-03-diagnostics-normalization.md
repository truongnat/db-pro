# Sprint 1 - S1-03 Diagnostics Normalization

## Goal

Standardize query diagnostics format so logs/status are deterministic across SQLite, PostgreSQL, and MySQL.

## Output Shape

All stage diagnostics now follow one shape:

```text
diag <key>=<value> <key>=<value> ...
```

Current keys:
- row query: `connect_ms`, `session_ms` (PostgreSQL), `fetch_ms`, `fetch_mode`
- non-row query: `connect_ms`, `session_ms` (PostgreSQL), `execute_ms`

## Example Messages

Row query:

```text
Fetched rows 1-500 (page 1) (more rows available) [diag connect_ms=12 session_ms=3 fetch_ms=44 fetch_mode=wrapped]
```

Non-row query:

```text
Done. 1 row(s) affected [diag connect_ms=8 execute_ms=4]
```

## Implementation

- File: `src-tauri/src/commands/query/execution.rs`
- Introduced shared diagnostics builder:
  - `format_diagnostics(...)`
  - `DiagnosticsBuilder::with_text(...)`
  - `DiagnosticsBuilder::render()`
- Applied consistently in:
  - `execute_sqlite_query`
  - `execute_postgres_query`
  - `execute_mysql_query`

## Validation (local)

- `npm run -s build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check`

All passed on 2026-03-05.
