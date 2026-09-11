# DB Client — Technology Decisions & Technical Strategy

Status: ratified baseline, amended for the native UI cutover; detailed architecture decisions are recorded in `docs/09-architecture-decisions.md`

> **Amendment (2026-09-11) — native UI direction.** The React/TypeScript/Vite presentation
> layer and the Tauri WebView that hosted it were retired. The product UI is now native
> `eframe`/`egui` (`crates/ui` + `crates/native-app`), and the React frontend is archived
> under `_archive/frontend/`. Rows and comparisons below that describe the React stack are
> kept as the historical record of that decision, but they no longer describe the current
> or planned UI. The authoritative current direction is `docs/07-fe-architecture.md` and
> `docs/10-egui-native-migration-plan.md`.

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
| Desktop shell | `eframe` + `egui` 0.29, native window | No WebView and no Node runtime; UI state lives in Rust |
| Backend language | Rust stable, edition 2021 initially | Safety, predictable resource handling, strong database ecosystem |
| UI system | `DbProTheme` semantic tokens mapped to `egui::Visuals`, plus shared widgets in `crates/ui` | One source of truth for color, density, and focus behavior |
| SQL editor | Native editor in `crates/ui` | Selection, undo/redo, and future completion stay in-process |
| Client navigation | `WorkspaceTabKind` + command registry | A desktop app does not need URL routing |
| Server state | Typed `UiCommand` / `UiEvent` task bridge + request registry | Explicit invalidation by event; no hidden query cache |
| Local UI state | `AppState` sub-states with versioned persistence | Reducer-owned display state; workers never mutate it |
| Validation | Rust validation in the domain, plus UI input validation | The domain is the only trust boundary |
| Legacy desktop shell | Tauri 2 (`crates/tauri-app`) | Transitional host for the archived frontend; scheduled for removal |
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
egui UI → UiCommand → runtime worker → application services → ports → database adapters
                 ◀────────── UiEvent (bounded, request-scoped) ──────────
```

- Domain types and ports must not depend on `egui`/`eframe`, Tauri, `sqlx`, or `rusqlite`.
- Application services own use cases, validation orchestration, authorization-by-policy, and audit events.
- PostgreSQL uses an async pool owned by the application state.
- SQLite operations run through a dedicated worker/actor or blocking task boundary. A raw `rusqlite::Connection` must never be shared directly across async command handlers.
- Active connections are identified by opaque `ConnectionId` values. Credentials are resolved only inside the backend and are never returned to the UI after save.
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
- SSH private keys are referenced by path and never copied into UI state.
- The legacy Tauri host's capabilities are deny-by-default and limited to required windows, commands, dialogs, filesystem paths, and events. (Transitional only; not part of the shipped runtime.)
- Export paths require user-selected filesystem permissions and never overwrite silently.
- The application must show the target connection and operation class before destructive execution.

## 7. Data contracts

The UI boundary uses versioned view models and one error envelope:

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
- Unit tests for domain validation, view-model mapping, error normalization, reducers, and grid/cell codecs.
- PostgreSQL fixture tests for connectivity, parameter binding, timeouts, cancellation, transactions, and introspection.
- SQLite fixture tests for file/in-memory modes, WAL, foreign keys, and metadata.
- Native UI runtime smoke for connection → query → result → error handling, with screenshots at the gate sizes.
- Dependency audit and secret scanning in CI.

## 9. Plan corrections completed

The task files were updated to follow this baseline. The ratified implementation details are maintained in `docs/09-architecture-decisions.md`:

1. UI tasks use native egui views and shared widgets in `crates/ui`.
2. PostgreSQL uses `sqlx::PgPool` only.
3. Parameters use the typed `QueryParam` model.
4. SQLite uses a dedicated actor boundary.
5. MVP rejects multi-statement execution.
6. Secrets use OS keyring with Argon2id/AES-GCM fallback.
7. Streaming uses a bounded, request-scoped typed `UiEvent` task-bridge channel. Tauri 2 `Channel<T>` is legacy-only and is not part of the shipped runtime.
8. Results use typed cells, bounded pages, request IDs, and stable errors.
9. The first delivery is a PostgreSQL read-only vertical slice.

## 10. Decision status

This is the technology baseline for the planning phase. Any later change should be recorded as an ADR with context, alternatives, decision, consequences, and migration impact.

## 11. Alternatives comparison

### 11.1 Desktop shell

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| `eframe` + `egui` (native Rust) | Truly native window and event loop, no WebView, no Node, UI state in Rust, one language across the stack | Immediate-mode UI requires an explicit state model; fewer ready-made complex desktop widgets; native SQL editor and grid must be built | **Selected (current direction)** |
| Tauri 2 | Rust core, small footprint, explicit capabilities, native system WebView | Linux WebView differences, Rust/JS boundary, more responsibility for native integrations | Selected initially; superseded — see the 2026-09-11 amendment and `docs/10-egui-native-migration-plan.md` |
| Electron | Mature ecosystem, bundled Chromium, easiest web compatibility and debugging | Large memory/disk footprint, larger attack surface, Node main-process security burden | Good fallback if UI compatibility dominates |
| Qt/QML | Mature native desktop widgets, strong desktop behavior, excellent long-lived tooling | Different UI stack, higher learning cost, weaker reuse of existing Rust skills | Strong native alternative, not the current product fit |
| Flutter | Consistent rendering and good cross-platform UI | Database desktop ecosystem and native integration require more custom work; less natural SQL-editor/web reuse | Viable for a new mobile-like product, not this client |

The original choice of Tauri 2 was a reasonable trade at the time: it kept a Rust
desktop core while reusing a React UI. The native cutover changes the trade — the
presentation layer is now the largest single source of complexity, and an immediate-mode
Rust UI removes the WebView/Node runtime, the IPC boundary, and the dual-language state
model entirely. The cost is that complex widgets (SQL editor, virtualized grid, ER
diagram) must be built natively, which `docs/10-egui-native-migration-plan.md` treats as
the primary risk.

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

### 11.4 Frontend / presentation layer

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| Native `eframe` + `egui` | No WebView or Node runtime; single language; direct access to Rust state; smallest deployment surface | Complex widgets must be built; immediate mode requires an explicit state model | **Selected (current direction)** |
| React + TypeScript | Largest ecosystem, strong Monaco/grid/library support | Requires a WebView host, a JS build toolchain, and a cross-language state bridge | Selected initially; superseded — frontend archived 2026-09-11 |
| Svelte | Smaller component surface and good runtime ergonomics | Smaller ecosystem for advanced database grids and desktop integrations | Would still require a WebView host |
| SolidJS | Fine-grained reactivity and high runtime performance | Smaller ecosystem and hiring pool | Would still require a WebView host |
| Vue | Mature, productive, good ecosystem | No decisive advantage | Would still require a WebView host |

The decisive factor changed once the product committed to a native window: every
WebView-based option keeps a second runtime, a second state model, and an IPC boundary.
`docs/10-egui-native-migration-plan.md` records the resulting component-level risks and
mitigations.

### 11.5 UI system

| Option | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| `DbProTheme` tokens + shared egui widgets | One source of truth for color, density, focus, and states; no external styling runtime; fits an IDE-like surface | Widgets must be built and maintained in-repo | **Selected (current direction)** |
| shadcn/ui + Radix + Tailwind | Modern visual baseline, source-owned components, accessible primitives | Requires a browser engine, a CSS pipeline, and a token-drift check | Selected initially; retired with the archived frontend |
| egui default visuals | Zero design work | Reads as a generic tool/demo surface; fails the visual acceptance gate | Rejected |
| MUI / Ant Design | Large component catalogs, fast CRUD/admin UI | Material/enterprise look is wrong for a database IDE; both require a browser engine | Not applicable to a native UI |

Semantic tokens live in `DbProTheme` (`crates/ui/src/theme.rs`) and are mapped to
`egui::Visuals`. Widgets read tokens from the theme instead of hard-coding colors, and a
token must not be duplicated across views.

### 11.6 State and data fetching

State is now owned entirely by Rust. `AppState` (`crates/ui/src/app_state.rs`) holds
display state as sub-states, and all backend interaction flows through the typed
`UiCommand` / `UiEvent` task bridge with a request registry for invalidation and
cancellation. The earlier TanStack Query + Zustand split was retired with the frontend:
with a single in-process runtime there is no serialization boundary, so a separate
server-state cache is unnecessary. Workers never mutate display state directly — only the
reducer on the UI thread applies events.

### 11.7 Final recommendation

Current baseline:

1. `eframe` + `egui` native UI in `crates/ui` + `crates/native-app`; no WebView, no Node.
2. `DbProTheme` semantic tokens as the single visual foundation.
3. `sqlx` only for PostgreSQL in the first implementation.
4. `rusqlite` behind a dedicated worker for SQLite and the metadata store.
5. No ORM, no second pool library, and no premature support for additional databases.
6. Re-evaluate `sqlx::sqlite` after the first integration tests if sharing one async abstraction materially reduces complexity.

This gives the project native performance and security control while removing the
WebView/Node runtime and the cross-language state boundary. The main scalability risk is
not the selected stack; it is uncontrolled feature scope and an unclear runtime/data
contract. The second risk is native widget parity for the SQL editor, virtualized grid,
and ER diagram — tracked explicitly in `docs/10-egui-native-migration-plan.md`.
