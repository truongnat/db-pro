# Crash Recovery, Error Boundaries, Logging & Sensitive-Data Redaction Audit

> Baseline SHA: `8d6aa54`
> Issue: #123
> Source: `crates/`, `frontend/src/`, `tauri-app/src/lib.rs`

## Frontend resilience

### Error boundaries

| Component | Error boundary? | Notes |
|---|---|---|
| App root (`App.tsx`) | **No** | No React error boundary wrapping the root |
| Route pages | **No** | No per-route error boundaries found |
| Query editor | **No** | Monaco errors propagate to console |
| Data grid | **No** | Render errors propagate to console |
| ER diagram | **No** | Cytoscape/ReactFlow errors propagate to console |

**Finding F1 [P1]**: No React error boundaries anywhere in the frontend. An unhandled render error in any component will crash the entire application (white screen of death). This is the most critical reliability gap.

**Recommendation**: Add at least a root-level error boundary that catches render errors and shows a recovery UI (reload button, crash report).

### Async error handling

| Area | Error handling | Notes |
|---|---|---|
| TanStack Query mutations | Error state in query cache | Components handle `isError` state |
| Connection errors | `server-error-normalize.ts`, `server-error-translate.ts` | Error translation layer exists |
| Workspace persistence | `onRehydrateStorage` with migration | Corrupted state triggers migration |
| Crash recovery store | `crash-recovery.store.ts` | Dirty SQL snapshots persisted to localStorage |

### Startup recovery

| Scenario | Behavior |
|---|---|
| Corrupted workspace localStorage | `migrateWorkspace` runs migrations; if JSON parse fails, zustand uses initial state |
| Orphaned tabs (connection deleted) | `reconcileWorkspaceTabs()` preserves orphan tabs with OrphanedTabView |
| Dirty SQL on restart | `useCrashRecoveryStore` has snapshots in localStorage (`db-pro-crash-recovery`) |
| Splash screen timeout | 6-second timeout in `lib.rs:32-35` forces main window visible if frontend never calls `finish_startup` |

## Rust/Tauri resilience

### Panic/unwrap audit

| Location | Usage | Risk |
|---|---|---|
| `lib.rs:38-43` | `.expect("failed to get/create app data dir")` | Startup panic if filesystem is read-only — acceptable (app can't function) |
| `lib.rs:47` | `.expect("failed to initialize meta store")` | Startup panic if meta.DB is corrupt — acceptable |
| `lib.rs:215` | `.expect("error while running tauri application")` | Standard Tauri bootstrap — acceptable |
| `sqlite/introspect.rs:183` | `.unwrap()` on `map.get_mut(&id)` | **P2**: Could panic on corrupt FK data |
| `dto.rs:1080,1101-1105` | `.unwrap()`/`panic!` in test code only | Acceptable (test code) |
| Integration tests | `.unwrap()`/`panic!` throughout | Acceptable (test code) |

**Finding F2 [P2]**: `sqlite/introspect.rs:183` has an `unwrap()` on a `HashMap::get_mut` that could panic if the FK graph has inconsistent data. This is in the introspection path — a malformed SQLite database with corrupt foreign key metadata could trigger a panic.

### Command error mapping

All Tauri commands return `Result<T, CommandError>`. The `CommandError` type implements `From<DbError>` which maps backend errors to serializable error responses. Frontend receives error messages as strings.

### Disconnect/retry behavior

| Scenario | Behavior |
|---|---|
| PG connection lost during query | SQLx returns error → `QueryService` propagates → frontend shows error |
| SQLite file deleted during use | rusqlite returns error → propagated to frontend |
| SSH tunnel drops | SSH process exits → next query fails → user must reconnect |
| App exit with in-flight queries | Tokio runtime drops tasks → connections closed → no explicit cleanup |

## Logging & redaction audit

### Rust tracing logs

| Log statement | Content | Redacted? |
|---|---|---|
| `keyring_vault.rs:114,133,145,161` | `"OS keyring unavailable: {e}"` | Yes — error message only, no secrets |
| `keyring_vault.rs:127` | `"fallback store failed after keyring success: {e}"` | Yes |
| `keyring_vault.rs:207` | `"secret stored in fallback file (keyring unavailable)"` | Yes — no secret value logged |
| `keyring_vault.rs:26` | `Debug` impl: `service_name: "[REDACTED]"` | Yes |
| `schema_service.rs:66,200,221` | `"failed to cache/introspect: {e}"` | Yes — error only |
| `query_service.rs:99,195` | `"failed to save query history: {e}"` | Yes |
| `connection_service.rs:52,110` | `"failed to clean up/restore secret: {cleanup_err}"` | Yes — error only, no secret value |
| `sqlite/actor.rs:240,245` | `"sqlite actor received/exiting shutdown"` | Yes |

**Finding F3 [P2]**: No tracing log statement leaks passwords, connection strings, or secret values. Redaction is consistent across the codebase.

### Frontend console logs

| Area | Console logging in production? |
|---|---|
| Vite build | `vite build` strips `console.log` only if configured — default keeps them |
| TanStack Query | Devtools disabled in production (no explicit config found) |
| Monaco Editor | Internal logging not controlled by DB Pro |

**Finding F4 [P2]**: Frontend production build does not explicitly strip `console.log/warn/error`. Vite's default behavior preserves console output. Sensitive data could appear in console logs if any component logs query results or connection details.

**Recommendation**: Add `drop_console` or equivalent to the Vite production build config.

### Sensitive data in serialized objects

| Object | Serialized? | Contains secrets? |
|---|---|---|
| `ConnectionConfig` | Yes (to meta.DB) | No — password stored separately via `secret_ref` |
| `Connection` | Yes (to meta.DB) | `secret_ref` only (keyring key, not the password) |
| `SshTunnelConfig` | Yes (to meta.DB) | `private_key_path` (path only, not the key content) |
| `QueryTabData` | Yes (to localStorage) | SQL text may contain sensitive literals |

**Finding F5 [P2]**: SQL text persisted in workspace tabs and crash recovery snapshots may contain sensitive data (e.g., `SELECT * FROM users WHERE password = '...'`). This is inherent to a SQL IDE — users write sensitive queries.

**Recommendation**: ACCEPT RC1. Document that persisted SQL may contain sensitive data. Consider adding a "sensitive mode" that disables persistence in a future release.

## Recovery classification

| Failure | Recovery class | Behavior |
|---|---|---|
| Component render error | **Unrecoverable** | White screen — no error boundary (F1) |
| Query execution error | **Recover inline** | Error shown, user can retry |
| Connection lost | **Reconnect/retry** | User must manually reconnect |
| Corrupted workspace state | **Reopen workspace** | Migration runs, tabs restored |
| Corrupted crash recovery | **Reset** | Store initialized empty |
| App crash with dirty SQL | **Recover on restart** | Snapshots in localStorage |
| Meta.DB corrupt | **Unrecoverable** | Startup panic (acceptable) |
| Keyring unavailable | **Recover inline** | Connection fails with clear error |

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 1 | F1 — No React error boundaries |
| P2 | 4 | F2-F5 |

**Conclusion**: The backend has good error handling with proper `Result` propagation and no secret leakage in logs. The critical gap is the frontend: no React error boundaries means any render crash takes down the entire app. The crash recovery store provides dirty-SQL snapshot persistence. SQL text in localStorage may contain sensitive literals (inherent to IDE). Console output is not stripped in production builds.
