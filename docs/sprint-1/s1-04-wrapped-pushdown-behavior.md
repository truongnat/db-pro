# Sprint 1 - S1-04 Wrapped Pagination Fallback Policy

## Goal

Prevent silent fallback when quick filter/sort pushdown cannot be guaranteed.

## Policy

- If quick filter/sort is requested:
  - Query must be pushdown-capable via wrapped SQL (`SELECT`/`WITH`).
  - If wrapped execution fails, return explicit error.
  - If statement type is not pushdown-capable, return explicit error.
- If quick filter/sort is not requested:
  - Normal stream fallback is allowed.

## Behavior Matrix

| Statement | Filter/Sort Requested | Wrapped SQL Supported | Behavior |
|---|---:|---:|---|
| `SELECT` / `WITH` | No | Yes | Wrapped page preferred; stream fallback allowed |
| `SELECT` / `WITH` | Yes | Yes | Wrapped page required; wrapped failure => explicit error |
| `SHOW` / `PRAGMA` / other row query | Yes | No | Explicit error: pushdown requires `SELECT/WITH` |
| Any non-row statement | N/A | N/A | Execute path unchanged |

## User-Facing Error (new)

```text
Filter/sort pushdown requires SELECT/WITH statements for wrapped pagination; got '<statement>'.
```

## Implementation

- File: `src-tauri/src/commands/query/execution.rs`
- Added:
  - `ensure_pushdown_supported(first_word, pushdown)`
- Enforced in:
  - `fetch_sqlite_page(...)`
  - `fetch_postgres_page(...)`
  - `fetch_mysql_page(...)`

## Validation (local)

- PostgreSQL matrix run (local): `artifacts/pg-matrix/pg-matrix-20260305-220954-summary.md` => PASS 6/6
- `npm run -s build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

All passed on 2026-03-05.
