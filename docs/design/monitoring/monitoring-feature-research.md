# Monitoring — feature research and recommendation

- Research date: 2026-09-24
- Repository baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: product role, native activity structure, provider contract, operational safeguards, reference-product patterns, and roadmap/document drift. No implementation is included.

## Decision

**Keep Monitoring as a first-class top-level activity.** The product goal already gives it a rail position, and diagnosing server sessions, locks, and resource use is distinct from editing database objects or running user queries (`docs/goals/goal-full-product.md:221-258`). The native rail already has a Monitor activity (`crates/ui/src/activity_bar_view.rs:92-96`). Do not demote it to a transient panel.

**Replace the current one-long-page composition with focused monitoring groups.** The current view combines core monitoring with Audit, FDW, replication, event-trigger, PostgreSQL-settings, and maintenance surfaces after the first snapshot (`docs/design/monitoring/monitoring-baseline.md`). Keep only session/health/metric context in Monitor; route those other management domains to their own information architecture. Use the Phase D group plan as the starting point, not its stale implementation baseline.

**Close the administrative-action boundary before calling Monitoring complete.** Termination is guarded in the UI by excluding the current-session row and asking for a PID confirmation, but the service itself only checks PostgreSQL and a positive PID. It does not enforce the configured read-only policy or current-session restriction. Cancel dispatches immediately, despite the Phase D confirmation contract. Make the invariant hold at the service boundary and make the UI show enough target and rollback context to prevent an accidental operation.

## User model

Users arrive with four distinct intents:

1. **See the target's current state** — know which connection/database/environment is selected, last refresh time, and whether a result is partial or stale.
2. **Find and understand a session/query problem** — filter/search active vs idle sessions, inspect wait state and transaction age, distinguish unavailable details from no activity, and open SQL without rerunning it.
3. **Understand contention and capacity** — find all relevant blockers and affected sessions; understand row limits, permissions, and cumulative statistics before treating counts as complete.
4. **Take one deliberate administrative action** — cancel a query, terminate a backend, reset accumulated query statistics, or run provider-specific maintenance after seeing the target and consequence.

SQLite is a separate local-file workflow, not an empty PostgreSQL session manager. Present its page/journal/file facts and only actions that exist. PostgreSQL-specific sessions, locks, and `pg_stat_statements` should be absent with an explicit reason on SQLite, not rendered as blank server dashboards.

## Recommended activity structure

```text
Monitoring
  Overview          connection/database, last updated, server or local facts, concise findings
  Sessions          searchable all/active/idle sessions, permissions and truncation state
  Active Queries    active/non-idle queries, duration/wait, query detail, Open SQL
  Locks & Txns      blockers, blocked sessions, lock waits, idle-in-transaction age
  Size & Stats      bounded relation/database sizes and clearly labeled counters
  Maintenance       provider-specific actions, scope and operational consequences
```

The groups may be pages or stable in-activity navigation; a literal second rail is unnecessary. Keep the target name, database, and environment visible wherever an action can affect the server. Keep deterministic health findings if each finding shows the exact evidence and remains explicitly advisory; do not turn them into a composite health score or an autonomous remediation surface.

Audit, FDW, replication, event-trigger administration, and server configuration are distinct tasks. Their current placement below Monitor is a surface-composition shortcut, not a reason to expand monitoring scope. Related links can navigate to dedicated surfaces without nesting unrelated action panels in the polling page.

## Polling, freshness, and errors

Use manual refresh with polling **off by default**, plus explicit Off / 2s / 5s / 10s / 30s choices and a visible last-updated timestamp. Persist the choice only if its connection/activity scope is unambiguous. Track one in-flight poll per query kind or coalesce requests; attach `RequestId` and reject stale completions. Stop polling when the activity or connection is inactive. On repeated failures, back off and expose an inline error/retry state while retaining the prior snapshot as explicitly stale.

These are already specified in the Phase D UX contract (`docs/goals/goal-phase-d-monitoring.md:137-189,210-228`), but differ from current source: polling defaults on at a fixed five seconds; each worker request is spawned independently; the reducer discards the snapshot request ID; and a Monitor-specific error field has no failure-path assignment (`monitoring-baseline.md`). This is a real server-load and correctness gap, not evidence that current users already experienced a stale overwrite.

## Data and provider contract

| Workflow | PostgreSQL | SQLite | MySQL / SQL Server |
|---|---|---|---|
| Overview / connection facts | Partial source implementation: version, current database, connection count/limit, database size. No uptime, and server-summary query failure can silently remove the entire summary. Provider runtime not verified. | Partial source implementation: journal mode, page/page-size/freelist counts and page-derived approximate file bytes; no server uptime or connection count. Provider runtime not verified. | `MonitoringService` rejects with explicit Unsupported text, but the UI currently requests snapshots for any connected driver and has no capability-specific unavailable surface. |
| Sessions / active queries | Source implementation through `pg_stat_activity`; text capped at 4,000 chars; query is not row-bounded. PostgreSQL may restrict other users' session details without `pg_read_all_stats`/equivalent. Runtime not verified. | No server-session model; empty session list and a general SQLite-local-state message. Query interrupt is a separate operation and this monitoring action returns Unsupported. | Unsupported; do not dispatch recurring polls for unsupported providers. |
| Locks / transactions | Partial source implementation, capped at 200 raw lock rows; blocker subquery returns at most one blocker per blocked PID. No separate transaction group or full blocker graph. | No server lock list; current adapter returns an empty vector rather than a category-specific unsupported result. | Unsupported. |
| Relation size / stats | Partial, top 40 relation rows in the snapshot; UI displays up to 20 without a `showing N of M` indicator. `pg_stat_statements` optional, query slice capped at 100 in the snapshot, UI displays up to 40. | No relation-size/statistics rows; local `PRAGMA` state only. | Unsupported. |
| Maintenance | `VACUUM`, `ANALYZE`, `VACUUM ANALYZE`, database-level from current UI. No `VACUUM FULL` or progress view. Confirmation and service read-only gate exist. | `VACUUM`, `ANALYZE`, `VACUUM ANALYZE`; no `PRAGMA optimize` action. Confirmation and service read-only gate exist. | Unsupported. |
| Session control | Cancel/terminate are parameterized and provider-routed. Cancel is not confirmed; terminate dialog identifies PID only; service has no configured-read-only or own-backend guard. Do not mark safe based only on parameterization. | No backend cancel/terminate; adapter states the session model is unsupported. | Unsupported. |

Do not promise exact data freshness: PostgreSQL cumulative statistics are not instantaneous counters and access to other users' activity is privilege-dependent ([PostgreSQL Monitoring Statistics](https://www.postgresql.org/docs/current/monitoring-stats.html)). SQLite PRAGMAs expose engine/file state rather than a server session model ([SQLite PRAGMA statements](https://www.sqlite.org/pragma.html)). Treat all returned row caps as visible truncation contracts, not silent UI limits.
PostgreSQL documents `pg_cancel_backend` as cancelling the target session's current query and `pg_terminate_backend` as sending a termination signal; with the default zero timeout, a `true` result only means the signal was sent, not that the backend has already exited ([PostgreSQL Server Signaling Functions](https://www.postgresql.org/docs/current/functions-admin.html#FUNCTIONS-ADMIN-SIGNAL)). DB Pro calls the termination function without a timeout and surfaces the boolean as `ok`, so refresh the session list before asserting the backend is gone (`crates/infrastructure/src/postgres/monitoring.rs:136-150`; `crates/ui/src/management_events.rs:181-194`).

## Reference-product observations

- [DBeaver — Session Manager Guide](https://github.com/dbeaver/dbeaver/wiki/Session-Manager-Guide) puts session management under a database connection's **Administer** section. The editor supports search, session actions, active/all, background/inactive views, refresh, and configurable auto-refresh. DBeaver's [PostgreSQL guide](https://dbeaver.com/docs/dbeaver/Database-driver-PostgreSQL/) lists Session Manager and Lock Manager as separate PostgreSQL administration tools. Useful patterns: discoverability through the selected connection, separate session/lock contexts, and explicit all-vs-active/refresh controls.
- [PostgreSQL Monitoring Statistics](https://www.postgresql.org/docs/current/monitoring-stats.html) is the provider contract, not a product workflow. It documents per-process activity and statistics visibility rules; those semantics should inform labels and partial-permission states.
- [SQLite PRAGMA statements](https://www.sqlite.org/pragma.html) is the provider contract for its local state. It supports the distinct local-file dashboard rather than a fake session list.

Reference pages were read on 2026-09-24. These sources describe their own products/providers and do not establish DB Pro runtime behavior.

## Prioritized open work

1. **P1 action-safety gap — enforce session-control invariants in the service.** `MonitoringService::terminate_backend` has no read-only check or current-backend guard; it accepts any positive PID. The UI hides the current row, but that is not the service contract. Cancel is immediate and may abort work in a transaction. Resolve the query-cancel/transaction semantics, apply a service-level policy, and make cancellation/termination confirmations identify the target and consequence before counting these actions as complete (`crates/core/src/application/monitoring_service.rs:146-185`; `crates/ui/src/monitoring_sessions_view.rs:163-168`; `monitoring_confirmation_view.rs:37-62`). This is source-derived; no database action was run.
2. **P2 freshness/load gap — make polling opt-in, bounded, and request-scoped.** Current default is on at a fixed five seconds; no in-flight guard; runtime spawns each request; reducers discard request IDs. This can overlap expensive snapshot queries and let late events overwrite fresh state. Add explicit intervals, stale-response protection, a last-updated/stale label, and failure backoff (`monitoring_state.rs:17-32`; `monitoring_activity_view.rs:61-73,198-205`; `runtime/src/worker.rs:1747-1762`; `ui/src/event_router.rs:38-39`).
3. **P2 feedback/permissions gap — distinguish no data, unsupported, partial visibility, and failure.** Current service has explicit unsupported drivers, but the UI polls all connected drivers. SQLite returns empty locks/sizes without per-category reason; PostgreSQL has no permission-limited flag; server-summary errors become `None`; Monitor-specific failure state is not set. Show structured empty/unavailable/error/partial states instead of silently missing groups (`monitoring_service.rs:38-48`; `sqlite/monitoring.rs:85-99`; `monitoring_snapshot.rs:30-52`; `monitoring_error` path in `monitoring-baseline.md`).
4. **P2 scale/completeness gap — make caps explicit and bound the session query.** `pg_stat_activity` has no SQL `LIMIT`; locks cap at 200, snapshot relation sizes at 40 and workload at 100; views display at most 30 blockers, 20 relations, and 40 workload statements without total/truncation disclosure. Bound server work and show visible totals/truncation state. Also decide whether one blocker per PID is an intentional summary or an incomplete blocker graph (`postgres/monitoring.rs:15-50,153-218,221-273,331-399`; `ui/monitoring_snapshot_view.rs:147-203`; `monitoring_workload_view.rs:35-55`).
5. **P2 IA gap — separate monitoring from unrelated administration.** Audit, FDW, replication, event triggers and PostgreSQL settings are appended to Monitor after the first snapshot. Give them separate ownership/navigation and keep Monitor focused on operational observation (`ui/sidebar_view.rs:152-200`).
6. **P2 provider/capability gap — model monitoring support independently.** `ServerSessions` remains coupled to UserService's server-user listing, while monitoring uses driver switches. Add provider/category capabilities and use them to avoid retrying unsupported monitor requests; do not silently reinterpret the existing user-management flag (`core/domain/capabilities.rs:151-169`; `core/application/user_service.rs:41-65`; `core/application/monitoring_service.rs:38-48`).
7. **P2 roadmap drift — refresh Phase D and capability matrix.** The goal's “PLANNING/no implementation” assertions and matrix's “missing” status are no longer source-true. Replace them with the present partial implementation and distinguish source wiring from automated, provider-runtime, and native-UI evidence; keep lifecycle state honest until all gates pass (`docs/goals/goal-phase-d-monitoring.md:10,105-120`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §8; `docs/plans/FEATURE_LIFECYCLE.md:51-78,80-94`).

## Recommended decision boundary

Retain the top-level Monitoring activity and phase-goal group model. First resolve the P1 administrative-action boundary and mark provider/session semantics, then update the Phase D current-baseline section and capability matrix from this exact SHA. Do not promote the feature to `COMPLETED`: this research collected source evidence only, did not run automated verification or the native app, and did not exercise PostgreSQL or SQLite. Current Monitoring is **partially implemented in source** for both providers, with different support contracts; both providers remain runtime-unverified in this task.

## Research limits

No feature plan/status transition was made. No source code or tests were changed/run, and no native UI, PostgreSQL, or SQLite runtime scenario was exercised. The P1/P2 entries distinguish source safety/completeness gaps from reproduced incidents; none is presented as runtime evidence.
