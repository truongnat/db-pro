# Monitoring — source baseline

- Source date: 2026-09-24
- Baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: native Rust/egui Monitor activity, UI/runtime/service/provider flow, operational actions, Phase D goal drift, and separate PostgreSQL/SQLite support. All source claims below refer to the stated SHA.
- No source code, tests, plans, or runtime state were changed or exercised for this research.

## Finding

Monitoring is already a wired native activity with PostgreSQL sessions, query workload, summary metrics, locks, relation sizes, heuristics, maintenance and administrative actions. SQLite reports local PRAGMA/page state and deliberately fabricates no server sessions. This is materially beyond the Phase D document's “no implementation” snapshot, but remains one long vertical activity rather than the planned groups, and several completion/safety/error-state requirements are not met.

## Surface and call path

- `crates/ui/src/activity_bar_view.rs:92-96` includes `Activity::Monitor`; `crates/ui/src/sidebar_view.rs:123-155` renders the activity and dispatches the active connection, driver, and connection name into the Monitor view. The palette also routes to Monitor (`crates/ui/src/palette_actions.rs:20`).
- The visible Monitor body is a single vertical composition: health/local state, session cards, server summary, blockers, relation sizes, and PostgreSQL workload (`crates/ui/src/monitoring_surface_view.rs:19-24,52-84`; `monitoring_snapshot_view.rs:5-38,107-203`; `monitoring_workload_view.rs:20-55`).
- After a snapshot exists, `sidebar_view.rs:152-200` appends Audit, FDW, replication, event-trigger, PostgreSQL-settings, and maintenance surfaces to the same Monitor activity. These are separate management concerns; the source does not give them a dedicated Monitor sub-navigation.
- Session cards expose Open SQL, direct Cancel, and a Terminate action that opens a confirmation (`crates/ui/src/monitoring_sessions_view.rs:85-171`; `monitoring_activity_view.rs:137-184`). The terminate dialog names only the PostgreSQL PID and says the client will disconnect (`crates/ui/src/monitoring_confirmation_view.rs:37-62`). Maintenance and statistics reset also have confirmation dialogs (`:64-121`).
- The header shows connected state, connection name/driver, manual Refresh and Auto-refresh (`crates/ui/src/monitoring_header_view.rs:18-67`). Polling defaults on and requests a snapshot immediately, then every fixed five seconds while the activity is drawn and connected (`monitoring_state.rs:17-32`; `monitoring_activity_view.rs:61-73`). There is no interval selector.
- UI commands travel through the task bridge to `RuntimeCommand`; `crates/runtime/src/worker.rs:1747-1880` spawns asynchronous monitoring snapshot, action and workload calls, then emits typed events. `MonitoringApi` delegates to `MonitoringService` (`crates/runtime/src/api.rs:1005-1071`). Runtime constructs separate PostgreSQL and SQLite ports (`crates/runtime/src/lib.rs:267-273`).
- Snapshot event IDs are discarded by the UI reducer (`crates/ui/src/event_router.rs:38-39`); `management_events.rs:20-44` updates state without a request freshness check. Snapshot dispatch updates the poll timestamp on send (`monitoring_activity_view.rs:198-205`), but does not track an in-flight request. Slow polls can therefore overlap and complete out of order; a later-arriving stale snapshot can replace a newer one. This is source-derived, not runtime-reproduced.
- Snapshot-load errors become a generic `RuntimeEvent::Failed` (`crates/runtime/src/worker.rs:1753-1761`). The Monitor error field is cleared on success (`management_events.rs:20-31`) and displayed if present (`monitoring_surface_view.rs:43-47`); no Monitor-specific failure reducer assignment was found. The page can retain an older snapshot without an inline stale/error state.

## Implemented domain and provider paths

| Area | PostgreSQL | SQLite |
|---|---|---|
| Snapshot/session inventory | `MonitoringService` routes to `PostgresMonitoringPort`; reads client backends from `pg_stat_activity`, with pid, DB/user/app/address/state/wait/query, durations and current-session marker (`core/application/monitoring_service.rs:38-48,60-62`; `infrastructure/postgres/monitoring.rs:15-50,86-117`). Query text is truncated to 4,000 chars. The session SQL has no `LIMIT`. | `SqliteMonitoringPort::list_sessions` returns an empty vector; local state reads `journal_mode`, `page_count`, `page_size`, `freelist_count`, and estimates file bytes as pages × page size (`infrastructure/sqlite/monitoring.rs:47-70`). The explanatory note says SQLite has no server sessions. |
| Server / lock / relation metrics | Server summary includes version, current DB, max/current connections, database size (`postgres/monitoring.rs:275-302`). Locks are capped at 200 rows, with one blocker pid at most per blocked pid (`:153-218`). Relation rows are sorted by total relation size and bounded to 40 in snapshot (`:221-273`; `core/application/monitoring_snapshot.rs:30-50`). | Server summary is `None`; locks and relation sizes return empty vectors (`sqlite/monitoring.rs:85-99`). Those categories do not carry an individual unsupported reason in the returned model. |
| Workload | Optional `pg_stat_statements`, no automatic extension installation; extension missing yields a DBA-facing message. Query slice is bounded (snapshot asks for 100; adapter clamps 1–500), sorts by total/mean/calls/rows, and truncates query text to 4,000 chars (`postgres/monitoring.rs:331-455`). | `workload` is `None`; stat statements identify PostgreSQL-only support (`sqlite/monitoring.rs:124-138`). |
| Session actions | `pg_cancel_backend($1)` and `pg_terminate_backend($1)` use bound PID parameters (`postgres/monitoring.rs:123-151`). The service verifies PostgreSQL and positive PID only (`monitoring_service.rs:146-185`). The UI hides both actions for its current-session row, and confirms termination, but Cancel is immediate (`monitoring_sessions_view.rs:163-168`; `monitoring_activity_view.rs:146-151`). | Cancel/terminate return explicit `Unsupported` errors; the cancel error suggests query interrupt on the active connection (`sqlite/monitoring.rs:73-83`). |
| Maintenance | `VACUUM`, `ANALYZE`, `VACUUM ANALYZE`; the adapter safely quotes optional identifiers (`postgres/monitoring.rs:304-329,516-521`). Current UI dispatches database-level actions with schema/table unset (`monitoring_activity_view.rs:121-127` and `maintenance_activity_view.rs:16-28`). | `VACUUM`, `ANALYZE`, `VACUUM ANALYZE`; optional table identifier is quoted (`sqlite/monitoring.rs:101-122,147-152`). No `PRAGMA optimize` action is exposed. |

The snapshot service degrades individual metrics when lock/relation queries fail and logs warnings, but `server_summary` uses `.unwrap_or(None)` without logging (`crates/core/src/application/monitoring_snapshot.rs:30-52`). `MonitoringState.monitoring_error` has no Monitor failure assignment in the inspected UI call path; generic runtime feedback does not preserve a per-panel stale-data indication.

## Safety and capability boundaries

- Maintenance and `pg_stat_statements` reset require the caller's `confirmed` flag and reject a connection whose stored configuration is read-only (`crates/core/src/application/monitoring_service.rs:77-97,115-144`). The UI provides confirmation for both.
- Cancel and terminate validate PostgreSQL provider and positive PID, but do not read the connection's `readonly` setting or independently protect the active backend in the service (`monitoring_service.rs:146-185`). The UI hides the active row; that UI restriction is not repeated at the service boundary. Terminate has only a PID-and-disconnect confirmation; Cancel has no confirmation. These are source-level action-boundary gaps, not reproduced destructive outcomes.
- Postgres SQL uses bound parameters for session actions. Maintenance identifiers are quoted in the adapter; maintenance SQL is not composed in the UI (`postgres/monitoring.rs:123-151,304-329,516-521`; `ui/maintenance_activity_view.rs:11-15`).
- `FeatureCapabilities.server_sessions` / `CapabilityFeature::ServerSessions` is still used by `UserService` to gate user listing (`core/domain/capabilities.rs:151-169`; `core/application/user_service.rs:41-65`). It is not the gate used by `MonitoringService`; session monitoring currently relies on the provider routing, not an explicit monitor capability.
- PostgreSQL session visibility depends on server permissions. The official PostgreSQL statistics documentation says ordinary users see full details for their own sessions but restricted/null information for other users unless granted `pg_read_all_stats` (or equivalent privileges). The current `MonitorSession` has no permission-limited/hidden-row state. The UI does not identify a partial session list as permission-limited.

## Source-document drift

- `docs/goals/goal-phase-d-monitoring.md:10,22-30,105-120` says PLANNING/no implementation, no service/catalog queries, and a placeholder Monitor. Those statements refer to an older code snapshot: the current rail, provider service/ports, real queries, and native Monitor body exist.
- The same goal's target UX still captures useful unfulfilled requirements: groups, bounded rows with a total/truncation indication, no polling by default, interval choice, request-id freshness, explicit permission/error/unsupported states, strong cancel/terminate confirmations, and honest provider metrics (`:122-189,191-228`). The current source does not satisfy several of these.
- `docs/goals/goal-full-product.md:221-258` lists Monitoring in the final rail and group structure, but §2's code baseline and `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §8 still classify the feature as missing. The expected final IA is a target; it is not evidence that current Monitor already has its separate groups.
- No Monitoring plan/status transition was made. `docs/plans/FEATURE_LIFECYCLE.md:51-78,80-94` requires source, automated, provider-runtime, and native-UI evidence to remain distinct and separately account for PostgreSQL and SQLite.

## Verification

- Source-only review at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Official provider docs read: PostgreSQL Monitoring Statistics (`https://www.postgresql.org/docs/current/monitoring-stats.html`) and SQLite PRAGMA statements (`https://www.sqlite.org/pragma.html`).
- No tests or builds run; no app launched; no PostgreSQL/SQLite runtime scenario exercised. Findings about query concurrency, error presentation, permissions, or action effects remain source-derived.
