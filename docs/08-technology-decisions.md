# DB Client — Technology Decisions & Technical Strategy

Status: proposed baseline for the planning phase

This document records the technology choices and implementation constraints that must be agreed before production code is added. It is intentionally explicit: a task plan is not complete until its dependencies, runtime model, security policy, and test strategy are implementable.

## 1. Product baseline

DB Client is a Linux-first desktop application for PostgreSQL and SQLite. The first release optimizes for a safe, reliable database workflow rather than broad database-driver coverage.

The first vertical slice is:

```text
Create connection → test PostgreSQL connectivity → execute read-only SQL → render result page
```

SQLite, schema browsing, editable grids, exports, SSH tunneling, and advanced administration follow after this slice is stable.

## 2. Chosen stack

| Concern | Decision | Reason / constraint |
|---|---|---|
| Desktop shell | Tauri 2 | Small Linux footprint and native Rust integration |
| Backend language | Rust stable, edition 2021 initially | Safety, predictable resource handling, strong database ecosystem |
| Frontend | React + TypeScript + Vite | Component ecosystem and fast desktop development loop |
| UI system | MUI 7 | Accessible primitives and consistent desktop UI theming |
| SQL editor | Monaco Editor | Mature SQL editing, selection, keyboard shortcuts, future completion |
| Client routing | TanStack Router | Typed route contracts and predictable module boundaries |
| Server state | TanStack Query 5 | Request lifecycle, cache invalidation, mutation state |
| Local UI state | Zustand with persist | Small state surface for settings and UI preferences |
| Validation | Zod at the UI boundary; Rust validation in domain | User feedback plus backend safety; neither layer is trusted alone |
| PostgreSQL driver | `sqlx` with PostgreSQL + Tokio + rustls | One async driver and pool implementation; compile-time SQL is optional for dynamic SQL |
| SQLite driver | `rusqlite` with bundled SQLite | Stable synchronous SQLite API and predictable deployment |
| Async runtime | Tokio | Required by Tauri/Rust services and PostgreSQL I/O |
| Metadata store | SQLite via a dedicated repository | Local connections metadata, history, cache, settings, and audit records |
| Secret storage | OS keyring via `keyring`; encrypted file fallback only by explicit policy | Avoid plaintext passwords and keep fallback migrationable |
| Serialization | Serde + serde_json | Rust/TypeScript DTO boundary and persisted metadata |
| Errors | `thiserror` internally; stable serializable DTO at Tauri boundary | Preserve context without exposing implementation details |
| Logging | `tracing` with redaction layer | Structured diagnostics without secrets or SQL credentials |
| Tests | Rust unit/integration tests, Vitest, Playwright | Test domain, adapters, UI utilities, and real user flows |
| Packaging | Tauri bundler: `.deb` and AppImage | Ubuntu-first distribution; Flatpak remains later |
| CI | GitHub Actions | Format, lint, typecheck, test, security checks, and packaging |

## 3. Deliberate exclusions

- `tokio-postgres`, `bb8`, and `bb8-sqlx` are not part of the baseline. Adding multiple PostgreSQL clients or pool abstractions would duplicate connection lifecycle logic.
- No ORM is planned for the target database. The product is a database client, so SQL and database metadata must remain visible and controllable.
- No remote API, account system, telemetry, or cloud sync is required for the MVP.
- No write query is executed automatically by the UI. Destructive operations require explicit confirmation and a visible target/SQL preview.

## 4. Runtime and concurrency model

The application has four boundaries:

```text
React UI → Tauri commands/events → application services → ports → database adapters
```

- Domain types and ports must not depend on React, Tauri, `sqlx`, or `rusqlite`.
- Application services own use cases, validation orchestration, authorization-by-policy, and audit events.
- PostgreSQL uses an async pool owned by the application state.
- SQLite operations run through a dedicated worker/actor or blocking task boundary. A raw `rusqlite::Connection` must never be shared directly across async command handlers.
- Active connections are identified by opaque `ConnectionId` values. Credentials are resolved only inside the backend and are never returned to the frontend after save.
- Long-running queries return a job/request ID and emit typed progress/result events. Events are versioned and support cancellation.

## 5. Query safety policy

### Parameter values

Use an explicit serializable parameter enum rather than an arbitrary `Decode` trait object:

```text
Null | Bool | Int64 | Float64 | String | Bytes | Uuid | DateTime | Json
```

Each adapter maps this enum to its native bind API. Unsupported values fail with a typed validation error.

### Statement execution

- SQL parsing must be performed with a PostgreSQL-aware tokenizer/parser where possible; splitting on `;` is not sufficient because of strings, dollar quoting, comments, and procedural blocks.
- Multi-statement execution is disabled by default for the first slice and enabled only with an explicit transaction policy.
- `SELECT`, explain, and metadata queries are read-only by default.
- `INSERT`, `UPDATE`, `DELETE`, DDL, and transaction commands are classified and shown clearly in the UI.
- `EXPLAIN ANALYZE` is opt-in because it executes the query. Plain `EXPLAIN` is the safe default.
- Query timeout, maximum rows, maximum payload size, cancellation, and pagination are enforced in the backend.

### Transactions

Transactions are backend-owned handles with explicit `begin`, `commit`, `rollback`, and `discard` operations. A transaction must not silently span unrelated editor tabs or application restarts.

## 6. Security strategy

- Passwords and private key material are stored through the OS keyring where available.
- The fallback file is encrypted, versioned, permission-restricted, and opt-in; its key must not be stored beside the ciphertext.
- Connection strings, passwords, bound values, private key paths when sensitive, and raw database errors are redacted from logs and audit records.
- SSH private keys are referenced by path and never copied into frontend state.
- Tauri capabilities are deny-by-default and limited to required windows, commands, dialogs, filesystem paths, and events.
- Export paths require user-selected filesystem permissions and never overwrite silently.
- The application must show the target connection and operation class before destructive execution.

## 7. Data contracts

The Tauri boundary uses versioned DTOs and one error envelope:

```json
{
  "code": "QUERY_TIMEOUT",
  "message": "The query exceeded the configured timeout.",
  "message_id": "query.timeout",
  "details": {},
  "request_id": "..."
}
```

Query results use column metadata plus typed cell values where possible. Converting every value to `String` is acceptable only for the initial prototype; the production contract must preserve null, numeric, binary, temporal, and JSON values.

## 8. Testing and quality gates

Every adapter must have both unit tests and real-database integration tests. Mocks alone cannot validate SQL introspection, type mapping, cancellation, transaction behavior, or keyring integration.

Required gates before M1/M2:

- `cargo fmt --check`, Clippy with warnings treated as errors for project code.
- TypeScript strict mode, ESLint, Prettier check, and typecheck.
- Unit tests for domain validation, DTO mapping, error normalization, and stores.
- PostgreSQL fixture tests for connectivity, parameter binding, timeouts, cancellation, transactions, and introspection.
- SQLite fixture tests for file/in-memory modes, WAL, foreign keys, and metadata.
- Playwright smoke flow for connection → query → result → error handling.
- Dependency audit and secret scanning in CI.

## 9. Plan corrections required

Before implementation, update the task files to follow this baseline:

1. Replace undefined `C-076` dependency with the actual PostgreSQL fixture task (`D-037` or a new CI fixture task).
2. Keep frontend packaging tasks in `03-frontend-tasks.md`; CI tasks should depend on `F-149`/`F-150` only after those IDs are defined and completed.
3. Remove duplicate PostgreSQL driver and pool tasks from `06-database-tasks.md`.
4. Replace `sqlx::Decode` parameter wording with the explicit parameter enum strategy.
5. Replace `app.emit_all()` with a Tauri 2 typed event channel design.
6. Replace raw `rusqlite::Connection` fields in shared async state with a worker/actor boundary.
7. Replace semicolon splitting with parser-backed statement classification, initially disabling multi-statement execution.
8. Change explain behavior to plain `EXPLAIN` by default and explicit `EXPLAIN ANALYZE` confirmation.
9. Add the missing `docs/01`–`docs/07` reference documents or mark them as planned documents in the index.

## 10. Decision status

This is the technology baseline for the planning phase. Any later change should be recorded as an ADR with context, alternatives, decision, consequences, and migration impact.

## 11. Alternatives comparison

### 11.1 Desktop shell

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| Tauri 2 | Rust core, small footprint, explicit capabilities, native system WebView | Linux WebView differences, Rust/JS boundary, more responsibility for native integrations | **Selected** |
| Electron | Mature ecosystem, bundled Chromium, easiest web compatibility and debugging | Large memory/disk footprint, larger attack surface, Node main-process security burden | Good fallback if UI compatibility dominates |
| Qt/QML | Mature native desktop widgets, strong desktop behavior, excellent long-lived tooling | Different UI stack, higher learning cost, weaker reuse of existing React/web skills | Strong native alternative, not the current product fit |
| Flutter | Consistent rendering and good cross-platform UI | Database desktop ecosystem and native integration require more custom work; less natural SQL-editor/web reuse | Viable for a new mobile-like product, not this client |

Tauri is not automatically “more scalable” than Electron. At application scale, both can handle a large feature set. Tauri is the better fit because the core workload is native/database/security-heavy and the user wants a Rust desktop core. Electron is safer if the product becomes primarily a web application with extensive Chromium-only UI behavior.

### 11.2 Rust database access

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| `sqlx` | Async, built-in pool, PostgreSQL and SQLite support, direct SQL, no ORM lock-in | Dynamic SQL and dynamic result typing require explicit mapping; compile-time query checking is limited for arbitrary user SQL | **Selected for PostgreSQL** |
| `tokio-postgres` | Low-level PostgreSQL control and mature async protocol API | Requires a separate pool and more plumbing; does not help SQLite | Use only if PostgreSQL-specific control becomes a hard requirement |
| Diesel | Strong typed query DSL and migrations | Poor fit for arbitrary SQL typed by users; heavier abstraction for a database client | Not selected |
| SeaORM | Productive ORM and relation modeling | ORM concepts are the wrong center of gravity for a SQL client | Not selected |
| `duckdb` | Excellent analytical queries and local OLAP | It is another database engine, not a replacement for PostgreSQL/SQLite connectivity | Future optional feature |

The strongest alternative is not a different ORM; it is keeping `sqlx` for PostgreSQL and introducing a separate adapter only when a concrete driver capability is missing.

### 11.3 SQLite runtime

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| `rusqlite` + worker boundary | Mature synchronous API, broad SQLite feature access, bundled deployment | Connection is not `Sync`; must isolate it from async shared state | **Selected for metadata and target SQLite** |
| `sqlx::sqlite` | Same conceptual API as PostgreSQL and async pool | Less direct access to SQLite-specific operations and still needs careful concurrency design | Viable simplification if feature parity is sufficient |
| `libsql`/Turso client | Sync-oriented and distributed SQLite options | Adds a separate product direction and does not replace local SQLite requirements | Not selected for MVP |

The worker boundary is part of the choice. Choosing `rusqlite` without that boundary would be an architectural error.

### 11.4 Frontend

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| React + TypeScript | Largest ecosystem, strong Monaco/grid/library support, team familiarity | More dependencies and conventions to govern | **Selected** |
| Svelte | Smaller component surface and good runtime ergonomics | Smaller ecosystem for advanced database grids and desktop integrations | Attractive alternative for a smaller team |
| SolidJS | Fine-grained reactivity and high runtime performance | Smaller ecosystem and hiring pool | Not enough advantage for this product |
| Vue | Mature, productive, good ecosystem | No decisive advantage over existing React plan | Viable alternative, but migration cost is unnecessary |

For this product, grid/editor ecosystem and maintainability matter more than benchmark differences between React, Svelte, and Solid.

### 11.5 State and data fetching

TanStack Query remains the right choice for request state and cache invalidation. Zustand is acceptable for local UI preferences, but it must not become a second server-state cache. Alternatives such as Redux Toolkit are stronger for very large event-driven state graphs, while Jotai is lighter for atomized local state. Neither provides a compelling advantage for the planned modules.

### 11.6 Final recommendation

Keep the current baseline with these clarifications:

1. Tauri 2 + React/TypeScript for the desktop shell and UI.
2. `sqlx` only for PostgreSQL in the first implementation.
3. `rusqlite` behind a dedicated worker for SQLite and the metadata store.
4. No ORM, no second pool library, and no premature support for additional databases.
5. Re-evaluate `sqlx::sqlite` after the first integration tests if sharing one async abstraction materially reduces complexity.

This gives the project the best balance of native performance, security control, SQL-client flexibility, and implementation speed. The main scalability risk is not the selected stack; it is uncontrolled feature scope and an unclear runtime/data contract.
