# DB Pro — Phase D Goal: Monitoring & Administration

- Doc ID: `GOAL-PHASE-D`
- Phase: D — Monitoring / Administration
- Release targets: v0.3 (D01–D02), v0.4 (D03–D04)
- Priority: P1 (D01–D02), P2 (D03–D04)
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: master goal §2.5 **C1** (query-cancel semantics must be settled against code
  before the cancel UX is designed) and §9 capability enforcement.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-h-ai.md` (AI administration tools depend on this phase),
  `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §1/§8/§16.

---

## 1. Problem

DB Pro cannot tell you anything about what the server is doing right now.

Observed facts (source-verified, 2026-09-14):

1. **There is no administration backend at all.** There is no `AdminService`, no `AdminPort`,
   and no occurrence of `pg_stat_activity`, `pg_locks`, or `pg_stat_user_tables` anywhere
   under `crates/`. The capability matrix (§8) lists every monitoring row as `MISSING`:
   active sessions, running queries, locks, transactions, terminate session, database size,
   table size, statistics, vacuum/analyze, server information.
2. **The rail entry is a placeholder.** `Activity::Monitor` renders
   `draw_activity_placeholder` with a `COMING SOON` badge and the text "Connection health and
   query activity will appear here when monitoring is available."
   (`crates/ui/src/navigation_view.rs:616-621, 637-648`). There is no `UiCommand` behind it.
3. **The capability flag overstates reality in a confusing way.**
   `FeatureCapabilities.server_sessions` is `true` for PostgreSQL
   (`crates/core/src/domain/capabilities.rs:167`), but the only implementation behind that
   flag is **user/role management** (`UserService` + `PostgresUserManager`), not session
   listing. A reader of the flag reasonably concludes sessions are supported. Master goal §2.5
   C3 records this; this phase splits the concept into *role management* (Phase E) and
   *session monitoring* (this phase).
4. **The only administrative primitives that exist are PG-only extras exposed outside the
   application layer**: `get_object_dependencies`, `list_partitions`, `list_tablespaces`
   (`crates/infrastructure/src/postgres/cross_connection.rs:5-139`, surfaced through
   `crates/runtime/src/api.rs:916-952` on `PostgresApi`) and `rename_schema_object`
   (`:954`). None of them is session/monitoring related, and all of them bypass the service
   layer (`docs/architecture/backend-debt-register.md` P1-01).
5. **Estimated sizes are the only size signal.** Table introspection returns `c.reltuples`
   approximations (`postgres/introspect.rs:129-140`); SQLite returns `row_count: None`
   unconditionally (`sqlite/introspect.rs:82-91`). There is no database-size query, no
   relation-size query, and no index-size query on either provider.
6. **Cancellation is asymmetric and the docs have it backwards.** Code truth:
   PostgreSQL `cancel` is `Unsupported` (`crates/infrastructure/src/postgres/connector.rs:199-203`;
   `capabilities.rs:129-132` sets `cancel = false`) while SQLite implements a real interrupt
   with actor acknowledgement (`crates/infrastructure/src/sqlite/actor.rs:106-124`;
   `capabilities.rs:185-187` sets `cancel = true`). The release docs claim the opposite
   (`docs/release/provider-capability-matrix.md:75-76, 137`; LIM-014). The UI gates its Stop
   button on the code flag, so behavior follows code. **Any admin cancel design must be built
   on the code truth.**
7. **No maintenance actions exist.** `VACUUM` appears nowhere in the product as a user
   operation; the only `VACUUM` is `VACUUM INTO` inside the SQLite backup engine
   (`sqlite_backup.rs:84-94`). PostgreSQL backup uses `pg_dump`, not maintenance commands.

Consequences: when a query blocks a table, when a connection pile-up occurs, or when a table
balloons in size, DB Pro offers nothing — the user leaves the application. This is the clearest
boundary between "SQL client" and "database IDE/admin tool", and it also blocks the Phase H
administration assistant.

## 2. Scope

### 2.1 In scope

| Group | Content | Milestone |
|---|---|---|
| Overview | Server version, uptime, current database, connection count vs limit, database size, cache-related counters *only where trustworthy* | D02 (data from D01) |
| Sessions | pid, user, database, application name, client address, state, backend start, transaction start, last query start, wait event type/name, query (truncated + full view) | D01/D02 |
| Active Queries | Filter to non-idle states, duration, wait state, query text, "Explain Query" and "Open SQL" hand-offs, Cancel Query | D01/D02 |
| Locks | `pg_locks` joined to activity, lock mode, relation, granted vs waiting, blocking/blocked relationships with durations | D03 |
| Transactions | Derived from activity (`xact_start`, state `idle in transaction`), with a warning affordance for long-idle transactions | D03 |
| Size & Stats | Database size, per-table total/table/index size, row estimates vs exact counts (clearly distinguished), sequential vs index scan counters | D04 |
| Maintenance | `VACUUM`, `VACUUM ANALYZE`, `ANALYZE`, `VACUUM FULL` (PostgreSQL, with lock warning); `VACUUM`, `ANALYZE`, `PRAGMA optimize` (SQLite) | D04 |
| Actions | Cancel Query, Terminate Session, Explain Query, Open SQL, Copy (pid/query), Refresh, Pause/Resume polling | D02/D03 |
| SQLite meaningful subset | File size, page size/count, freelist count, `PRAGMA`-derived schema/journal/mmap info, maintenance actions that exist | D01/D04 |

### 2.2 Out of scope

- Historical metrics storage / time-series graphs.
- Alerting, thresholds, notifications.
- Replication status, connection-pool inspection, extension inventory, WAL/checkpoint tuning
  dashboards.
- Automatic remediation (no auto-kill, no auto-vacuum policies, no scheduled maintenance).
- Server-side configuration editing (`pg_settings` write).

## 3. Non-goals

- **No auto-kill / auto-terminate of anything.** Every disruptive action is user-initiated and
  confirmed.
- **No "health score" or composite verdicts.** Show measurements; do not invent a summary
  metric that hides what it is based on.
- **No fabrication of unavailable metrics.** SQLite has no sessions or locks; the UI must say
  so rather than rendering empty tables that look broken.
- **No new cancellation system.** Reuse the existing worker cancellation maps and the
  `DbConnector::cancel` path; do not add a third registry (the legacy Tauri
  `ExecutionRegistry` must not be extended — master goal §4.8).
- **No monitoring of the app's own internals** in this phase (that is diagnostics, Phase L).
- **No polling that blocks the UI thread or the runtime worker** with long-running queries;
  every monitoring query is bounded and timeout-guarded.

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| `AdminService` / `AdminPort` | **Absent** | no such modules in `crates/core` |
| Session / query / lock queries | **Absent** | zero matches for `pg_stat_activity`, `pg_locks`, `pg_stat_user_tables` under `crates/` |
| Monitor activity UI | Placeholder | `ui/src/navigation_view.rs:616-621`, `:637-648` |
| `server_sessions` flag | True for PG, but implemented only as role management | `capabilities.rs:167`; `core/application/user_service.rs:53-136` |
| Per-execution cancel | PG `Unsupported`, SQLite implemented | `postgres/connector.rs:199-203`; `sqlite/actor.rs:106-124`; `capabilities.rs:129-132, 185-187` |
| Cancel UX | Stop button in the query editor, gated on the capability flag; `CancelQuery` in the worker calls `query_api.cancel(connection_id)` then fires the local oneshot (`worker.rs:1524-1553`) | `ui/src/query_view.rs` header; `runtime/src/worker.rs:1524-1553` |
| Sizes | `reltuples` estimate only (PG); `None` (SQLite) | `postgres/introspect.rs:129-140`; `sqlite/introspect.rs:82-91` |
| Maintenance actions | None exposed | only `VACUUM INTO` inside `sqlite_backup.rs:84-94` |
| Server information | None (`version()` not queried) | grep |
| PG extras (dependencies/partitions/tablespaces) | Exist but bypass the service layer | `infrastructure/postgres/cross_connection.rs:5-139`; `runtime/src/api.rs:916-952` |
| Connection health in the UI | `components/workspace.rs::ConnectionHealth` exists (gallery-only) | `crates/ui/src/components/workspace.rs` |
| Timeout/deadline infrastructure | Exists for queries and transactions (both providers) | `postgres/connector.rs:39-48`; `sqlite/actor.rs:370-383` |

## 5. UX

### 5.1 Monitoring activity

```text
MONITORING
  Overview          version · uptime · connections · database size · (cache stats where honest)
  Sessions          all backends with state and wait info
  Active Queries    non-idle backends, sorted by duration desc
  Locks             lock table + blocking graph                (D03)
  Transactions      long-running / idle-in-transaction list     (D03)
  Size & Stats      database / table / index sizes + scan stats (D04)
  Maintenance       VACUUM / ANALYZE / PRAGMA optimize          (D04)
```

### 5.2 Layout rules

- Every list is a virtualized grid with a row cap and a "showing N of M" indicator; the cap is
  raised on demand rather than silently truncated.
- The sidebar header carries the target connection, database, and environment badge; the
  monitoring target is never ambiguous.
- Polling is explicit: a refresh interval selector (Off / 2s / 5s / 10s / 30s), a Pause/Resume
  control, a last-updated timestamp, and a manual Refresh. **No polling by default** — the user
  opts in, and the chosen interval persists per activity.
- Stale responses never overwrite newer ones: each poll is `RequestId`-scoped and the reducer
  rejects out-of-order results (same rule as the rest of the UI).
- Query text is truncated in the grid with a detail pane that shows the full statement
  (read-only) and offers "Open in Query Editor" and "Copy".

### 5.3 Row actions and confirmations

| Action | Confirmation |
|---|---|
| Cancel Query | Confirm with pid, user, database, running duration, and the first line of the query. States that the transaction may be aborted. |
| Terminate Session | Strong confirmation: typed pid or typed connection name; lists what will be rolled back; Production connection shows the environment banner and requires the typed name. |
| Explain Query (from an active query) | No confirmation (read-only). Opens the plan in the workspace; the plan is generated in a **separate** session, not by re-running the query (see §11). |
| Open SQL | No confirmation (opens the statement in a new query document). |
| VACUUM / ANALYZE | Confirm with target and expected effect; `VACUUM FULL` gets a stronger warning (access-exclusive lock, rewrites the table, requires free disk space equal to the table size). |
| Terminate Blocker (D03) | Same strength as Terminate Session, with the blocked-session list shown. |

### 5.4 Overview content discipline

Only show a metric when it is trustworthy on that provider:

| Metric | PostgreSQL | SQLite |
|---|---|---|
| Version string | `version()` | `sqlite_version()` from the library |
| Uptime | `pg_postmaster_start_time()` | Not applicable — show process/file info instead, or omit with a reason |
| Connections (current/limit) | From `pg_stat_activity` count vs `max_connections` | Not applicable |
| Database size | `pg_database_size(current_database())` | File size on disk |
| Cache hit ratio | Only when computed from `pg_stat_database` counters **and** labeled as cumulative-since-reset (never as a live hit rate that implies a time window) | Not applicable |
| Buffer/cache internals | Omit unless the counter semantics are explained inline | Omit |
| Journal/mmap/page info | n/a | `PRAGMA`-derived, labeled |

The rule: **a metric that cannot be explained on screen is not shown.** If a value is
cumulative, the label says so. If a value requires `pg_monitor` privileges the role lacks, the
UI shows "not visible to this role" rather than a zero.

### 5.5 Empty, error, and permission states

- **Empty**: "No active queries" (not an empty grid with headers).
- **Permission-limited**: a PostgreSQL role without `pg_monitor` sees fewer rows in
  `pg_stat_activity`; the UI states "Some sessions are hidden: your role cannot see other
  users' queries" instead of pretending the list is complete.
- **Error**: a failed monitoring query shows the structured error with Retry; polling pauses
  after repeated failures (backoff) rather than hammering the server.
- **Unsupported (SQLite)**: groups that do not exist are hidden and the reason is listed in an
  "Unavailable on SQLite" block — consistent with the pattern used in Phases A/B.

## 6. Architecture

### 6.1 Flow

```text
UI (Monitoring activity)
  → UiCommand::MonitoringQuery { request_id, connection_id, kind: MonitoringKind, params }
  → RuntimeCommand::MonitoringQuery
  → MonitoringService::{sessions, active_queries, locks, sizes, stats, server_info}
  → AdminPort implementation (postgres | sqlite)
  → bounded, timeout-guarded catalog query
  → UiEvent::MonitoringData { request_id, kind, rows, truncated: bool, permissions_limited: bool }

Actions:
  → UiCommand::CancelQuery(connection_id, pid)      → AdminPort::cancel_backend
  → UiCommand::TerminateSession(connection_id, pid)  → AdminPort::terminate_backend
  → UiCommand::MaintenanceAction(connection_id, action, target) → AdminPort::maintenance
```

### 6.2 Rules

1. **Read paths are pure reads.** Monitoring listings use `SELECT`-only catalog queries with a
   per-connection timeout from `RunConfig`/`ConnectionConfig`; they never start a transaction
   that holds locks and never touch user tables.
2. **Monitoring never reuses the user's interactive session semantics.** "Explain Query" for an
   active query must not re-execute the query. It either (a) explains a *copy* of the statement
   (safe, but can differ from the running plan) or (b) fetches an existing plan from
   `pg_stat_activity`'s query plus `pg_stat_statements` **only if available** — and it must
   state which one it did. Silent re-execution is forbidden because it can duplicate writes.
3. **Bounded queries.** Every listing has an explicit `LIMIT` (for example 500 rows) plus a
   total estimate, and a stated truncation flag. Sorting is done server-side
   (`ORDER BY duration DESC`) so truncation keeps the interesting rows.
4. **Polling is UI-owned.** The UI's request registry drives polling; the runtime does not run
   a background loop. This keeps cancellation, staleness, and lifecycle handling in one place
   and prevents a monitoring poll from competing with user work.
5. **No new cancellation registry.** `AdminPort::cancel_backend`/`terminate_backend` are new
   operations, but the *per-execution* Stop path keeps using the existing worker maps.
6. **Terminate/cancel are audited** as `Administrative`/`Destructive` per master goal §10.1.

### 6.3 The cancel decision (C1 resolution)

D01 must resolve, in writing, the following before implementing:

| Question | Required outcome |
|---|---|
| Does DB Pro implement PostgreSQL per-execution query cancel? | Decision recorded. If yes, the design must track the **backend pid** for each pool connection used by a running query (a `pg_backend_pid()` probe or per-connection tagging), because `pg_cancel_backend` needs the pid, not the app-level handle. If no, the UI must stop offering a Stop button on PostgreSQL and state why. |
| Does the capability flag change? | Yes — it must match the implementation in the same PR (master goal §4.3). A flag-only change without behavior is forbidden. |
| Do the release docs get corrected? | `provider-capability-matrix.md` and `LIM-014` currently assert the inverse; the fix is a follow-up doc chore (outside this task's edit scope) and is recorded in master goal §2.5 C1. |
| What does the Monitoring page offer on PostgreSQL? | `Terminate Session` is always available (privilege permitting). `Cancel Query` is available only if the per-execution cancel decision above lands; otherwise the action is present but explains the limitation and offers Terminate instead. |

This is deliberately a decision-first item: implementing an admin cancel on top of an
`Unsupported` per-execution cancel, without stating the relationship, would produce two
inconsistent answers to "can I stop this query?".

## 7. Domain model

New module `crates/core/src/domain/monitoring.rs`:

```rust
pub struct ServerInfo {
    pub version: String,
    pub uptime_secs: Option<u64>,
    pub current_database: String,
    pub current_user: String,
    pub connections_used: Option<u32>,
    pub connections_limit: Option<u32>,
    pub database_size_bytes: Option<u64>,
    pub provider_notes: Vec<String>,     // e.g. "cache counters are cumulative since last reset"
}

pub enum SessionState { Active, Idle, IdleInTransaction, IdleInTransactionAborted, Disabled, Unknown }

pub struct SessionInfo {
    pub pid: i32,
    pub user: String,
    pub database: String,
    pub application_name: Option<String>,
    pub client_addr: Option<String>,
    pub state: SessionState,
    pub backend_start: Option<OffsetDateTime>,
    pub xact_start: Option<OffsetDateTime>,
    pub query_start: Option<OffsetDateTime>,
    pub wait_event_type: Option<String>,
    pub wait_event: Option<String>,
    pub query: Option<String>,           // truncated for transport; full text via detail fetch
    pub is_current_connection: bool,     // so the UI can protect the user's own session
}

pub struct ActiveQuery {
    pub pid: i32,
    pub user: String,
    pub database: String,
    pub duration_ms: Option<u64>,
    pub state: SessionState,
    pub wait_event_type: Option<String>,
    pub wait_event: Option<String>,
    pub query: String,
    pub query_digest: String,            // stable id for dedupe/grouping
}

pub struct LockInfo {
    pub pid: Option<i32>,
    pub locktype: String,
    pub mode: String,
    pub relation: Option<String>,
    pub granted: bool,
    pub waiting: bool,
    pub blocked_by: Vec<i32>,
    pub wait_duration_ms: Option<u64>,
    pub query: Option<String>,
}

pub struct TransactionInfo {
    pub pid: i32,
    pub state: SessionState,
    pub xact_start: Option<OffsetDateTime>,
    pub idle_duration_ms: Option<u64>,
    pub query: Option<String>,
}

pub struct RelationSize { pub schema: String, pub name: String, pub kind: RelationKind,
                          pub total_bytes: u64, pub table_bytes: Option<u64>, pub index_bytes: Option<u64>,
                          pub row_estimate: Option<i64>, pub exact_rows: Option<i64> }

pub struct RelationStats { pub schema: String, pub name: String, pub seq_scan: u64, pub idx_scan: u64,
                           pub n_live_tup: i64, pub n_dead_tup: i64, pub last_vacuum: Option<OffsetDateTime>,
                           pub last_autovacuum: Option<OffsetDateTime>, pub last_analyze: Option<OffsetDateTime> }

pub enum MaintenanceAction { Vacuum { analyze: bool, full: bool }, Analyze, Reindex, SqliteOptimize }

pub struct MonitoringLimits { pub row_limit: usize, pub truncated: bool, pub permissions_limited: bool }
```

Note on `exact_rows` vs `row_estimate`: both exist so the UI can never present an estimate as a
fact. Computing `exact_rows` requires a real `count(*)`, which the user must request explicitly
(bounded by timeout) because it can be expensive.

## 8. APIs, services, ports

### 8.1 Port

```rust
#[async_trait]
pub trait AdminPort: Send + Sync {
    async fn server_info(&self, handle: ConnectionHandle) -> Result<ServerInfo, DbError>;
    async fn sessions(&self, handle: ConnectionHandle, limit: usize) -> Result<(Vec<SessionInfo>, MonitoringLimits), DbError>;
    async fn active_queries(&self, handle: ConnectionHandle, limit: usize) -> Result<(Vec<ActiveQuery>, MonitoringLimits), DbError>;
    async fn locks(&self, handle: ConnectionHandle, limit: usize) -> Result<(Vec<LockInfo>, MonitoringLimits), DbError>;
    async fn transactions(&self, handle: ConnectionHandle, limit: usize) -> Result<(Vec<TransactionInfo>, MonitoringLimits), DbError>;
    async fn database_size(&self, handle: ConnectionHandle) -> Result<Option<u64>, DbError>;
    async fn relation_sizes(&self, handle: ConnectionHandle, schema: &str, limit: usize) -> Result<Vec<RelationSize>, DbError>;
    async fn relation_stats(&self, handle: ConnectionHandle, schema: &str, limit: usize) -> Result<Vec<RelationStats>, DbError>;
    async fn exact_row_count(&self, handle: ConnectionHandle, schema: &str, table: &str) -> Result<i64, DbError>;
    async fn cancel_backend(&self, handle: ConnectionHandle, pid: i32) -> Result<bool, DbError>;
    async fn terminate_backend(&self, handle: ConnectionHandle, pid: i32) -> Result<bool, DbError>;
    async fn maintenance(&self, handle: ConnectionHandle, action: MaintenanceAction, target: Option<ObjectRef>) -> Result<MaintenanceOutcome, DbError>;
}
```

Implementations:

- `PostgresAdmin` (`crates/infrastructure/src/postgres/admin.rs`, new): `pg_stat_activity`,
  `pg_locks` (joined with activity for blocked_by via `pg_blocking_pids`), `pg_database_size`,
  `pg_total_relation_size`/`pg_table_size`/`pg_indexes_size`, `pg_stat_user_tables`,
  `pg_postmaster_start_time()`, `version()`, `pg_cancel_backend`, `pg_terminate_backend`,
  `VACUUM`/`ANALYZE`/`REINDEX`.
- `SqliteAdmin` (`crates/infrastructure/src/sqlite/admin.rs`, new): returns
  `DbError::Unsupported { capability: "sessions"|"locks"|..., provider: SQLite, reason }` for
  the groups that do not exist, and implements the meaningful subset (`database_size` from the
  file, `database_size`/page stats via `PRAGMA page_count`/`page_size`/`freelist_count`,
  `maintenance` for `VACUUM`/`ANALYZE`/`PRAGMA optimize`). The actor owns the connection, so all
  of this runs through `SqliteCommand` variants — **no direct connection access from the async
  side** (ADR 4 in `docs/09-architecture-decisions.md`).

### 8.2 Service

```rust
impl MonitoringService {
    pub async fn overview(&self, connection_id: ConnectionId) -> Result<MonitoringOverview, DbError>;
    pub async fn sessions(&self, connection_id: ConnectionId, limit: usize) -> Result<MonitoringPage<SessionInfo>, DbError>;
    pub async fn active_queries(&self, connection_id: ConnectionId, limit: usize) -> Result<MonitoringPage<ActiveQuery>, DbError>;
    pub async fn locks(&self, connection_id: ConnectionId, limit: usize) -> Result<BlockingGraph, DbError>;
    pub async fn transactions(&self, connection_id: ConnectionId, limit: usize) -> Result<MonitoringPage<TransactionInfo>, DbError>;
    pub async fn sizes(&self, connection_id: ConnectionId, schema: Option<&str>) -> Result<Vec<RelationSize>, DbError>;
    pub async fn stats(&self, connection_id: ConnectionId, schema: Option<&str>) -> Result<Vec<RelationStats>, DbError>;
    pub async fn cancel_query(&self, connection_id: ConnectionId, pid: i32) -> Result<ActionOutcome, DbError>;
    pub async fn terminate_session(&self, connection_id: ConnectionId, pid: i32) -> Result<ActionOutcome, DbError>;
    pub async fn maintenance(&self, connection_id: ConnectionId, action: MaintenanceAction, target: Option<ObjectRef>)
        -> Result<MaintenanceOutcome, DbError>;
}
```

`ActionOutcome` distinguishes: succeeded, target not found, insufficient privilege, and refused
(own session / own current query). The service must **refuse to terminate the connection's own
backend pid** unless the user explicitly confirms (it would disconnect the app).

`BlockingGraph` is a service-level structure (`nodes: Vec<SessionInfo>`,
`edges: Vec<{blocker: i32, blocked: i32, relation, mode, wait_ms}>`) built from the lock rows —
the port returns lock rows; the service builds the graph so the tree rendering and any future
AI tool share one implementation.

### 8.3 Runtime/UI wiring

```text
UiCommand::MonitoringOverview   { request_id, connection_id }
UiCommand::MonitoringList       { request_id, connection_id, kind, limit, schema? }
UiCommand::MonitoringAction     { request_id, connection_id, action }     // Cancel | Terminate | Maintenance
UiEvent::MonitoringOverviewLoaded { request_id, overview }
UiEvent::MonitoringListLoaded     { request_id, kind, page }
UiEvent::MonitoringActionCompleted{ request_id, action, outcome }
```

`MonitoringAction` carries the A01-style class so the UI can render the right confirmation:
Cancel Query = `Administrative`, Terminate = `Destructive`, Maintenance = `Administrative`
(`VACUUM FULL` = `Destructive + LongRunning`).

### 8.4 Interaction with existing query execution

- The Stop button in the query editor is unchanged in behavior; D01 only records whether
  per-execution PG cancel will be implemented (see §6.3) and, if so, adds it through the same
  `QueryApi::cancel` path (never through a second mechanism).
- Monitoring's Cancel Query calls `AdminPort::cancel_backend(pid)`. Cancel-by-pid and
  cancel-by-request are two legitimate operations on different identities; the UI must label
  them distinctly ("Cancel query (by pid)" in Monitoring vs "Stop" in the editor) so users are
  not surprised by which one they invoked.

## 9. Provider behavior

### 9.1 PostgreSQL

| Group | Queries | Notes |
|---|---|---|
| Server info | `version()`, `pg_postmaster_start_time()`, `current_database()`, `current_user`, counts from `pg_stat_activity`, `max_connections` from `pg_settings` (read-only) | `pg_settings` read is fine (a `SELECT`); writing settings is out of scope |
| Sessions | `pg_stat_activity` | Visibility limited by role: `pg_monitor`/superuser sees all; others see their own rows plus query text of others often as `<insufficient privilege>`. The UI must surface this, not hide it |
| Active queries | `pg_stat_activity WHERE state <> 'idle'` sorted by `now() - query_start DESC` | Duration computed server-side to avoid clock skew |
| Locks | `pg_locks` joined to `pg_stat_activity`; `pg_blocking_pids(pid)` for blocker edges | Wait duration comes from `query_start` of the waiting (non-granted) request; a true wait-start is not exposed in stock PG — label the number as "waiting since query start" if it is an approximation |
| Transactions | Derived from `pg_stat_activity` states | `idle in transaction` gets a warning affordance (holds snapshot/locks) |
| Sizes | `pg_database_size`, `pg_total_relation_size`, `pg_table_size`, `pg_indexes_size` | Per-schema listing joined to `pg_class` + `pg_namespace` |
| Stats | `pg_stat_user_tables` | Scan counters are cumulative since last stats reset — label it |
| Actions | `pg_cancel_backend`, `pg_terminate_backend` | Return `true`/`false`; `false` must be surfaced as "refused (own session or insufficient privilege)", not as success |
| Maintenance | `VACUUM`, `VACUUM ANALYZE`, `VACUUM FULL`, `ANALYZE`, `REINDEX` | Cannot run inside a transaction block; the implementation must not wrap them in one. `VACUUM FULL` takes `ACCESS EXCLUSIVE` |

### 9.2 SQLite

| Group | Behavior |
|---|---|
| Server info | Library version via `rusqlite::version()`, file path, file size, page size, page count, freelist count, journal mode, `auto_vacuum` setting |
| Sessions / queries / locks / transactions | `DbError::Unsupported` with a reason; the UI hides the groups and lists them as unavailable |
| Sizes | File size; per-table size derived from `dbstat` **only if the build exposes it** (the `bundled` feature must be verified in D04 — if unavailable, state that and show page-count-based estimates with a label) |
| Stats | Not available; hide |
| Maintenance | `VACUUM`, `ANALYZE`, `PRAGMA optimize`, and (with explicit warning) `VACUUM INTO` for compaction-into-a-file (which is backup, not maintenance — keep it in Phase C) |
| Actions | No cancel/terminate. Note the asymmetry: SQLite *does* support interrupting a running *statement* (`actor.rs:106-124`), which is a query-editor concern, not a session-admin concern |

### 9.3 Capability flags after this phase

`DatabaseCapabilities.features` must be split so flags stop lying:

| New flag | PostgreSQL | SQLite |
|---|---|---|
| `role_management` (was ambiguously `server_sessions`) | true (Phase E owns the UI) | false |
| `session_monitoring` | true (D01) | false |
| `lock_monitoring` | true (D03) | false |
| `relation_size_metrics` | true (D01/D04) | true (file/page metrics) |
| `maintenance_actions` | true (D04) | true (`VACUUM`/`ANALYZE`/`optimize`) |
| `terminate_session` | true (D01) | false |
| `query_cancel` | per §6.3 decision | true |

Each new flag lands with its implementation and a consistency test (master goal §4.3).

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Group navigation inside Monitoring | `ObjectList`/`SegmentedTabs` or the sidebar list pattern from Queries | reuse |
| Lists (sessions, queries, sizes) | `DataGrid` (`result_grid.rs`) | reuse |
| Blocking graph | `ObjectTree` (`components/tree.rs`) + new edge rendering | extend |
| Status/state badges (state, waiting, granted) | `Badge` (`components/badge.rs`) | reuse |
| Confirmation | `ConfirmationDialog` (A01/§10.4 payload) | reuse |
| Progress for maintenance | `ProgressView` | reuse |
| Row cap / truncation notice | `Alert` + a "load more" affordance | reuse |
| Last-updated + interval selector | `Toolbar` + `Select` | reuse |
| Activity log for maintenance/terminate audit | `ActivityLog` | promote from gallery |

### 10.2 Screens

1. **Overview** — metric cards (`MetricCard` exists, gallery-only → promote) with honesty
   labels; connection count; size; version; provider notes.
2. **Sessions** — grid with columns pid/user/db/app/client/state/start/tx start/last query
   start/wait/query; context menu with all actions; a "hide idle" toggle and a text filter.
3. **Active Queries** — grid sorted by duration; the current connection's own queries marked.
4. **Locks** — two views: a flat lock table and the blocking tree; selecting a node highlights
   the corresponding rows.
5. **Transactions** — long-running and idle-in-transaction list with the warning affordance.
6. **Size & Stats** — database total, per-relation sizes with a treemap-or-bar breakdown
   (simple, no chart library), scan counters with a cumulative-since-reset label, "compute exact
   count" action per relation.
7. **Maintenance** — action list with target picker (whole database / schema / table),
   confirmation, progress, and a result summary; history in the activity log.

### 10.3 Interaction rules

- Polling never runs while the Monitoring activity is not visible (leaving the activity stops
  the interval but keeps the last snapshot with a "stale" marker).
- Every destructive action re-reads the target row immediately before confirmation to avoid
  acting on stale data (pid reuse is possible; the confirmation shows the current state and
  query text).
- The Monitoring page never auto-refreshes a confirmation dialog.
- Connecting the same database twice (two connections) is legitimate; the UI shows which
  connection a pid belongs to and refuses terminate-by-pid unless the pid belongs to the
  selected connection (a pid from another connection's server is a different server).

## 11. Safety model

| Action | Class | Requirements |
|---|---|---|
| All listings | `ReadOnly` | Bounded + timeout; no transaction; no user-table access |
| Exact row count | `ReadOnly` (potentially expensive) | Explicit user action; timeout; cancellation; labeled as a full scan |
| Explain Query (active) | `ReadOnly` | Must not re-execute the query (see §6.2 rule 2); states the source of the plan |
| Cancel Query (by pid) | `Administrative` | Confirmation with pid/user/query; audited |
| Terminate Session | `Destructive + Administrative` | Strong confirmation; Production requires typed name; refuses own session without explicit override; audited |
| VACUUM / ANALYZE | `Administrative` | Confirmation with target/effect; audited |
| VACUUM FULL / REINDEX | `Destructive + LongRunning` | Strong confirmation (exclusive lock, rewrite, disk space); progress + cancel where the server allows; audited |

Additional rules:

- **Privilege honesty**: never present an empty list as "no sessions" when it is actually
  "not visible". `permissions_limited` must be set whenever the role's visibility is reduced
  (detectable on PG by the presence of `<insufficient privilege>` query text or by comparing
  the visible count with `pg_stat_activity` totals the role can see).
- **No auto-refresh of destructive actions**: a terminate/cancel dialog is bound to a snapshot
  and warns if the snapshot is older than a few seconds.
- **No silent failure**: `pg_terminate_backend` returning `false` is an explicit outcome.
- Read-only connections may view monitoring (reads are allowed) but must not perform
  cancel/terminate/maintenance — the service enforces it, not just the UI.

## 12. Testing strategy

### 12.1 Unit

- Graph construction: blocking chains (A blocks B blocks C), self-locks, multiple lock modes on
  one relation, missing activity rows for a pid (lock without a session), cycles.
- Duration computation and the "waiting since query start" approximation labeling.
- Classification of each action per §11; refusal cases (own pid, unknown pid).
- Capability flags: per-driver flag set matches the implemented methods (consistency test).
- Truncation/row-cap metadata propagation.

### 12.2 Service / integration

- PostgreSQL live (ignored suite, Docker fixture like the safety-hardening verification):
  - open two sessions, one holding `ACCESS EXCLUSIVE` and one waiting; assert the blocking graph
    reports the right edges and modes;
  - start a long `pg_sleep` query in a second connection; assert it appears in active queries
    with a plausible duration; cancel it and assert the session ends with the cancellation
    error;
  - terminate a session and assert it disappears;
  - sizes: compare `pg_total_relation_size` values with `psql`;
  - maintenance: run `VACUUM`, `VACUUM ANALYZE`, `ANALYZE` on a fixture and assert they report
    success and update `last_vacuum`/`last_analyze`;
  - permission path: run the session listing as a non-`pg_monitor` role and assert
    `permissions_limited = true`.
- SQLite (always-run): every unsupported group returns `Unsupported` with a reason and no SQL;
  file/page metrics match filesystem values; `VACUUM`/`ANALYZE`/`PRAGMA optimize` execute through
  the actor and succeed.
- Timeout behavior: a monitoring query that exceeds the deadline returns `QueryTimeout` and does
  not wedge the connection (existing timeout infrastructure).

### 12.3 UI

- Polling lifecycle: interval selection, pause/resume, stop on activity change, stale-response
  rejection (out-of-order responses must not overwrite newer data).
- Confirmation variants: cancel vs terminate vs maintenance; Production strength; own-session
  protection.
- Empty vs permission-limited vs unsupported rendering (three distinct states).
- Virtualization and row-cap behavior with a synthetic 5,000-row session list.
- Regression: the query editor Stop button behavior must not change unless §6.3 decides to
  implement PG cancel — in which case its tests change deliberately, not accidentally.

### 12.4 Determinism fixtures

Lock/session tests need a seeded workload. Reuse the Docker-based PG fixture pattern already
used by the safety-hardening and SSH verification suites so these tests are reproducible on CI
and locally.

## 13. Runtime verification

Recorded separately per provider, in each milestone's `VERIFICATION.md`.

**PostgreSQL (D01/D02)**

1. Open Monitoring → Overview on a live connection; capture version, uptime, connection count,
   database size; verify against `psql` values.
2. Sessions: capture the grid with at least three states (`active`, `idle`,
   `idle in transaction`).
3. Start a long query in a second connection; capture it in Active Queries with a live duration;
   cancel it from Monitoring; capture the confirmation payload and the resulting terminated
   session, and capture the error observed in the second session.
4. Terminate an idle session; capture the confirmation and the disappearance.
5. Capture `permissions_limited` behavior using a low-privilege role.
6. Capture polling: interval change, pause, stale marker when leaving the activity.

**PostgreSQL (D03/D04)**

7. Reproduce a lock conflict with two sessions; capture the lock table and the blocking tree
   including modes and wait durations.
8. Capture long-running and idle-in-transaction lists with the warning affordance.
9. Capture sizes for database/table/index and compare with `psql`; run "compute exact count"
   and capture the estimate-vs-exact distinction.
10. Run `VACUUM ANALYZE` on a fixture table; capture confirmation, progress, result, and the
    refreshed `last_analyze` in stats.
11. Run `VACUUM FULL`; capture the stronger warning and the lock implication.

**SQLite (D01/D04)**

12. Capture the Monitoring activity showing only the meaningful subset with the
    "Unavailable on SQLite" block and reasons for sessions/locks/stats.
13. Capture file/page metrics against a real file, and a `PRAGMA optimize` / `VACUUM` run with
    its result.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | D01 | MonitoringService Core | v0.3 | P1 | §6.3 cancel decision recorded | The backend, capability split, and action classification must exist before any UI; also resolves C1/C3 |
| 2 | D02 | Monitoring Activity UI | v0.3 | P1 | D01 | Makes it visible and removes the placeholder; establishes the polling/staleness discipline |
| 3 | D03 | Locks & Blocking Analysis | v0.4 | P2 | D02 | Needs deterministic lock fixtures and the tree rendering; higher complexity than listings |
| 4 | D04 | Size, Statistics & Maintenance | v0.4 | P2 | D02 | Maintenance carries the heaviest safety requirements (`VACUUM FULL` locks) and needs the progress/audit path |

Ordering rules: D01 must not ship without the cancel decision recorded (it is cheap to write
and expensive to retrofit). D03 and D04 are independent of each other. D01 may begin while
Phase C's job/progress model (C01) is in flight, but D04's maintenance progress should reuse
C01's progress plumbing rather than inventing a second one.

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **The Monitor placeholder is gone** — `draw_activity_placeholder` is no longer called for
   Monitoring, and no `COMING SOON` text remains for it.
4. **The capability split is done** (`role_management` vs `session_monitoring` vs
   `lock_monitoring` vs `terminate_session` vs `maintenance_actions`), each flag backed by an
   implementation and a consistency test; the old ambiguous `server_sessions` usage is removed
   or redefined with a recorded decision.
5. **The cancel decision is documented** and the capability flag matches the implementation
   (master goal §2.5 C1 resolved in-code; the release-doc correction is recorded as a chore).
6. **PostgreSQL listings are accurate** against a live server with two-session fixtures, with
   recorded evidence for sessions, active queries, locks, transactions, sizes, and stats.
7. **Cancel/terminate/maintenance are confirmed, audited, and refuse unsafe targets** (own
   session, unknown pid, stale snapshot) — proven by tests and runtime evidence.
8. **Permission-limited states are honest**: a low-privilege role produces an explicit
   "some sessions are hidden" state, not an empty list presented as complete.
9. **SQLite shows only true metrics** and returns `Unsupported` with reasons for the rest; the
   reasons are visible in the UI and covered by tests.
10. **No auto-kill, no auto-vacuum, no auto-anything** exists in the code (review evidence:
    grep for automatic invocation of `cancel_backend`/`terminate_backend`/`maintenance` outside
    explicit command handling returns nothing).
11. **Polling does not affect interactive performance**: measured idle CPU/frame impact with
    polling on at the shortest interval is recorded, and polling stops when the activity is not
    visible.
12. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite including the lock fixture).
13. **Docs updated**: capability matrix §1/§8 rows move off MISSING, master goal §9.5 reflects
    reality, and the `server_sessions` wording conflict (C3) and cancel inversion (C1) are
    recorded as resolved-in-code with the doc-fix chore noted.
14. **Non-goals respected**: no metrics history, no alerting, no auto-remediation, no third
    cancellation registry, no fabricated metrics.

Phase D is complete when an operator can open Monitoring on a PostgreSQL connection, see
sessions and active queries with real durations and wait states, inspect a lock conflict as a
blocking graph, cancel a query and terminate a session with confirmations, view accurate
database/table/index sizes and scan statistics with estimates clearly distinguished, and run
`VACUUM`/`ANALYZE` with progress and audit — while a SQLite user sees exactly the meaningful
metrics and an explicit reason for everything else.
