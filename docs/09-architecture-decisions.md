# DB Pro — Architecture Decisions

Status: ratified before implementation; decisions 1 and 7 superseded by the native UI cutover

> **Amendment (2026-09-11) — native UI direction.** Decisions 1 and 7 below described the
> React/Tauri-webview presentation layer. That layer was retired: the React frontend is
> archived under `_archive/frontend/` and the UI is now native `eframe`/`egui`. Decision 1
> is replaced by the native UI architecture in `docs/07-fe-architecture.md`; decision 7's
> `Channel<T>` streaming is replaced by the typed `UiCommand`/`UiEvent` task bridge. All
> other decisions (2–6, 8, 9) remain in force unchanged. `crates/tauri-app` is kept only as
> a legacy transitional host and is scheduled for removal at cutover.

This document closes the nine architecture decisions required before Phase 0 implementation. It supersedes ambiguous wording in the task plans and is the implementation source of truth.

## Decision summary

| # | Decision | Status |
|---:|---|---|
| 1 | ~~Frontend UI uses source-owned shadcn/ui-style components, Radix primitives, Tailwind, and CSS variables; MUI is removed from the baseline~~ — **superseded**: the UI is native `eframe`/`egui` with `DbProTheme` tokens (`docs/07-fe-architecture.md`) | Superseded |
| 2 | PostgreSQL uses `sqlx::PgPool` directly; no `tokio-postgres`, `bb8`, ORM, or second pool abstraction | Ratified |
| 3 | Dynamic query parameters use a serializable `QueryParam` enum mapped by each adapter | Ratified |
| 4 | SQLite uses a dedicated actor/worker owning its `rusqlite::Connection`; async callers communicate through commands | Ratified |
| 5 | Multi-statement execution is disabled in MVP; later support requires parser-backed classification and explicit transaction mode | Ratified |
| 6 | OS keyring is primary; encrypted fallback uses Argon2id + AES-256-GCM with versioned records and strict permissions | Ratified |
| 7 | ~~Tauri 2 `Channel<T>` is used for result streaming; events are reserved for lifecycle notifications~~ — **superseded**: the typed `UiCommand`/`UiEvent` task bridge carries bounded, request-scoped batches | Superseded |
| 8 | Query/result contract uses typed cells, bounded pages, request IDs, and stable error envelopes | Ratified |
| 9 | The first implementation is a vertical slice with PostgreSQL read-only execution before SQLite, CRUD grid, exports, or SSH | Ratified |

## 1. UI system

**Superseded on 2026-09-11.** The React + TypeScript + Vite stack, shadcn/Radix
components, Tailwind/CSS variables, Monaco, TanStack Query, and Zustand were retired with
the archived frontend.

The current UI system is native `eframe` + `egui`:

```text
eframe + egui 0.29
DbProTheme semantic tokens → egui::Visuals
shared widgets in crates/ui/src/components.rs
lucide-icons
native SQL editor + native virtualized grid
```

All UI code lives in `crates/ui` and is owned by the product. Widgets read semantic tokens
from `DbProTheme` instead of hard-coding colors. See `docs/07-fe-architecture.md`.

## 2. PostgreSQL driver

Use `sqlx` directly with:

```text
sqlx::postgres::PgPool
sqlx::postgres::PgConnectOptions
runtime-tokio-rustls
postgres, chrono, uuid, json
```

The pool is created once per saved active connection and owned by a backend connection registry. `tokio-postgres`, `bb8`, `bb8-sqlx`, Diesel, and SeaORM are excluded. A second abstraction can be introduced only through a new ADR if a concrete PostgreSQL capability cannot be implemented with SQLx.

Compile-time SQL checking is used for application-owned static introspection queries where practical. User-authored SQL remains dynamic and is validated, classified, parameterized, bounded, and mapped at runtime.

## 3. Typed parameter model

The domain contract is:

```rust
enum QueryParam {
    Null,
    Bool(bool),
    Int64(i64),
    Float64(f64),
    Text(String),
    Bytes(Vec<u8>),
    Uuid(String),
    DateTime(String),
    Json(serde_json::Value),
}
```

The wire format is tagged JSON, for example `{ "type": "int64", "value": 42 }`. Date/time and UUID values are validated at the domain boundary and normalized to canonical strings before adapter binding. The PostgreSQL adapter maps supported variants to `sqlx::Arguments`; unsupported combinations return `UNSUPPORTED_PARAMETER_TYPE`. The UI never interpolates values into SQL text.

## 4. SQLite synchronization

Each SQLite target connection is owned by a `SqliteActor` running on a dedicated Tokio task. The actor owns the non-`Sync` `rusqlite::Connection` and receives typed commands through a bounded `mpsc` channel. Each command returns a oneshot response.

The metadata store uses the same actor pattern. There is no `Arc<Mutex<rusqlite::Connection>>` in application state and no direct connection access from a Tauri command. Long operations are dispatched with `spawn_blocking` only inside the actor implementation when required by the operation.

## 5. Multi-statement policy

MVP accepts exactly one statement per execute request. The backend rejects multiple statements with `MULTI_STATEMENT_DISABLED`; a UI editor may still contain multiple statements, but the user must select one statement to run.

Future support requires:

- parser-backed statement boundaries, never `split(';')`;
- statement classification into read, write, DDL, transaction, and administrative categories;
- an explicit transaction mode and rollback behavior;
- per-statement result envelopes;
- confirmation for write/DDL batches;
- integration tests for strings, comments, dollar quoting, and procedural SQL.

`EXPLAIN (FORMAT JSON)` is the default. `EXPLAIN ANALYZE` is treated as execution and requires explicit confirmation.

## 6. Secret storage

The primary path stores credentials in the platform OS keyring through `keyring`. The metadata database stores only a keyring reference and non-secret connection fields.

Fallback is opt-in and uses a versioned encrypted file record:

```text
DBP1 | algorithm=argon2id/aes-256-gcm | salt | nonce | ciphertext | tag
```

The key is derived from a user-provided master password with Argon2id. The encrypted file is created with owner-only permissions. The master password is never persisted, logged, sent to the frontend, or stored beside the ciphertext. Migration from fallback to OS keyring deletes the fallback ciphertext only after a successful keyring write and verification.

Every secret-bearing type implements redacted `Debug`; tracing filters remove passwords, tokens, private key contents, connection URLs, and bound values.

## 7. Streaming contract

**Superseded on 2026-09-11.** Tauri `Channel<T>` is legacy-only and is not part of the
shipped runtime.

Streaming now uses the typed `UiCommand` / `UiEvent` task bridge. The runtime worker
replies with bounded, request-scoped messages:

```rust
enum UiEvent {
    Loading { request_id: RequestId, operation: Operation },
    ConnectionsLoaded(Vec<ConnectionSummary>),
    QueryBatch(QueryBatch),
    QueryCompleted(QueryCompleted),
    Failed { request_id: RequestId, error: DbErrorDto },
    Notification(Notification),
}
```

Each batch carries a sequence number, a request ID, and row/byte limits, so a stale batch
cannot overwrite a newer result. The UI can cancel by request ID and cancellation is routed
to the adapter. Lifecycle notifications (connection status, schema invalidation) use the
same bridge but are low-volume and never carry row data.

## 8. Query/result contract

Cells preserve type information:

```text
Null | Bool | Int64 | Float64 | Text | Bytes | Uuid | DateTime | Json
```

Results are page-oriented. The default page is 500 rows, with a configurable hard maximum of 100,000 rows and a byte limit. Every operation carries a `RequestId`. Errors use the stable envelope `{ code, message_id, message, details, request_id }`; raw driver errors are logged only after redaction and are not sent directly to the UI.

## 9. First implementation slice

The implementation order is fixed:

```text
Scaffold → domain DTOs → PostgreSQL connect/test → one read-only query
→ typed result mapping → native result grid → integration test → CI gates
```

SQLite actor, metadata store, query history, schema explorer, editable grid, export, SSH tunneling, and advanced administration follow only after this slice passes its integration and error-path tests.

## Consequence

The existing task plans must not implement older wording such as raw `rusqlite::Connection` in shared state, semicolon splitting, `EXPLAIN ANALYZE` by default, `app.emit_all()` row streaming, MUI as the UI system, or a generic `Vec<String>` result contract. They must also not reintroduce a WebView/React UI layer, a Node build step, or an unbounded row stream into display state.
