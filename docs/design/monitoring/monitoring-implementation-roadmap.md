# Monitoring — implementation roadmap

- Source baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Inputs: [monitoring-baseline.md](monitoring-baseline.md), [monitoring-feature-research.md](monitoring-feature-research.md), [AGENT_EVIDENCE.md](AGENT_EVIDENCE.md)
- Evidence boundary: current-state statements below are source findings at the stated SHA, not claims of runtime behavior. Priorities P1/P2 are retained from feature research; new work without a prior severity is marked proposed priority.

## Decision

Keep Monitoring as a first-class top-level activity. Refocus it into operational groups (Overview, Sessions, Active Queries, Locks & Transactions, Size & Stats, Maintenance); move unrelated administration to its owning surfaces. Close service-level session-action safeguards first, then correctness/freshness and provider feedback, before expanding UX. Monitoring is partially implemented in source, not complete: native UI, automated, PostgreSQL runtime, and SQLite runtime evidence are distinct gates.

## Current state

### Implemented in source

- Native rail/palette routing and a vertical Monitor surface with snapshot, session cards, metrics, blockers, relations, workload, maintenance and administrative surfaces (`monitoring-baseline.md` §Surface and call path, lines 12–19; §Implemented domain, lines 23–32).
- PostgreSQL provider path queries sessions, summary, locks, relation sizes, optional `pg_stat_statements`; parameterized cancel/terminate and identifier-quoted maintenance exist (`monitoring-baseline.md` lines 27–31).
- SQLite provides local PRAGMA/page facts and explicitly lacks server sessions; maintenance actions exist (`monitoring-baseline.md` lines 27–31).
- Confirmation/read-only protections cover maintenance and statistics reset (`monitoring-baseline.md` lines 35–39).

### Not implemented / incomplete

- Focused monitoring groups, opt-in/configurable polling, explicit freshness, and structured partial/unsupported/error states are missing or incomplete (`monitoring-feature-research.md` lines 26–46, 70–78).
- Session query has no SQL row cap; existing caps and UI display limits are not consistently disclosed; blocker query returns at most one blocker per PID (`monitoring-feature-research.md` lines 52–60, 75).
- Provider/category capability modeling, separate from user-management `ServerSessions`, is missing (`monitoring-feature-research.md` line 77).
- MySQL/SQL Server monitoring is unsupported by service, with no unavailable UI surface (research lines 50–57).

### Needs fix (source-derived, not runtime-reproduced)

- **P1:** session cancel/terminate do not enforce read-only/current-backend policy in service; cancel lacks confirmation; termination confirmation identifies PID only. Preserve this as a blocker to claiming action safety (`monitoring-feature-research.md` line 72; baseline lines 17, 35–40).
- **P2:** fixed, on-by-default polling has no in-flight guard; event reducer discards request IDs, allowing stale completion overwrite by source control flow (`monitoring-feature-research.md` lines 44–46, 73).
- **P2:** Monitor failure state is not assigned, server-summary errors silently collapse to `None`, SQLite empty categories lack reasons, and PostgreSQL permission-limited session visibility is not represented (`monitoring-feature-research.md` line 74; baseline lines 20–21, 33, 41).
- **P2:** silently limited results obscure completeness (`monitoring-feature-research.md` line 75).
- **P2:** unrelated Audit/FDW/replication/event-trigger/settings management is appended to Monitor (`monitoring-feature-research.md` line 76; baseline lines 15–16).
- **P2:** Phase D and capability matrix current-state claims are stale; update from evidence without treating source wiring as completion (`monitoring-feature-research.md` line 78).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (inherited) | fix | Service guard gap and UI-only current-session exclusion: research line 72; baseline lines 17, 35–40 | Enforce provider, positive PID, read-only policy and active-backend prohibition at service boundary; confirm cancel as well as terminate with target, database/environment and consequence. Reflect signal-vs-completion semantics. | None; first | Service rejects invalid/read-only/current-backend requests regardless of UI; UI confirmation identifies selected target/action; successful signal is not presented as proof session exited, and refresh verifies state. |
| P2 (inherited) | fix | Poll/freshness concurrency: research lines 44–46, 73; baseline lines 18–21 | Default polling off; add Off/2s/5s/10s/30s, one in-flight request per kind/coalescing, request freshness rejection, last-updated/stale indication and failure retry/backoff. | P1 action contract independent; event/request correlation | With delayed responses, an older response cannot replace newer state; no overlapping poll of same kind; failures retain prior snapshot labeled stale with inline retry; polling stops when inactive. |
| P2 (inherited) | fix | Partial/error/unsupported states: research line 74; baseline lines 20–21, 33, 41 | Represent category result as available/empty/unsupported/permission-limited/partial/error and preserve server-summary failure details. Stop polling unsupported providers. | Capability contract below; freshness/error path | SQLite explains absent server categories; PostgreSQL can label restricted visibility/partial results; failed summary is visibly failed, not silently absent; unsupported provider causes no repeated request. |
| P2 (inherited) | missing | Unbounded sessions and silent truncation: research line 75; baseline lines 27–29 | Bound session SQL; expose returned/total/truncated counts consistently for sessions, locks, relations and workload; resolve whether blocker graph is summary or complete. | Structured result state | High-volume fixture/provider response stays within stated limits; each capped panel says “showing X of Y” or explicitly “total unknown”; multiple blocker behavior matches documented contract. |
| P2 (inherited) | missing | Mixed admin composition: research lines 26–40, 76; baseline lines 15–16 | Organize activity into Overview, Sessions, Active Queries, Locks & Txns, Size & Stats, Maintenance; move Audit/FDW/replication/event-trigger/server settings out, retain navigable links. | None | Monitor navigation exposes these focused groups; unrelated management panels no longer render within the Monitor composition; current connection/database context remains visible for actions. |
| P2 (inherited) | missing | Existing user capability reused while Monitor switches on provider: research line 77; baseline line 40 | Add independent provider/category monitoring capability contract; do not repurpose `ServerSessions` user-list capability. | State model/provider outcomes | Each provider/category is gated explicitly; user listing behavior remains independent; unsupported categories are not dispatched. |
| Proposed P2 | upgrade | SQLite local facts omit `PRAGMA optimize`; current PG/SQLite facts differ: baseline lines 27–31; research lines 52–57 | Decide and, if accepted, add SQLite-appropriate optimization action and clearly scoped local-file presentation without implying server metrics. | Provider capability/state contract; product decision | SQLite view labels file/local facts and exposes only supported actions; any optimization action is confirmed and returns visible success/failure. |
| P2 (inherited) | verification | Source-document drift and lifecycle evidence requirements: research lines 78, 82; baseline lines 43–48 | Reconcile Phase D and capability matrix current-state claims; record source, automated, PostgreSQL runtime, SQLite runtime, and native UI evidence separately. | Feature behavior and gates above | Docs state partial source status accurately; no completion claim until all required evidence categories are recorded independently. |
| Proposed P2 | verification | No tests/app/provider runs in source research: baseline lines 50–54; research line 86 | Add targeted automated coverage for service action invariants, stale response handling, limits and provider-state rendering; run native UI and provider scenarios independently. | Corresponding changes | Automated cases prove observable action rejection/freshness/limits; native UI confirms stale/error/unsupported states; PostgreSQL and SQLite each have separately recorded runtime outcomes. |

## Rollout / dependency order

1. Close the P1 service action boundary and establish confirmation/result semantics before exposing further operational controls.
2. Introduce request identity/freshness and error-state contracts, then make polling opt-in and bounded.
3. Add provider/category capabilities and explicit partial/unsupported states; bound queries and disclose truncation.
4. Refactor IA into focused groups and remove unrelated surfaces; then consider optional provider-specific upgrades.
5. Update current-state documents and complete automated, native UI, PostgreSQL and SQLite gates independently. Do not transition lifecycle state on source evidence alone.

## Provider/support matrix

| Capability | PostgreSQL | SQLite |
|---|---|---|
| Overview | Source has version, DB, connections, size; summary failure may silently yield none. Runtime not verified. | Source has PRAGMA/page/file estimates; no server uptime/connections. Runtime not verified. |
| Sessions / active queries | Source via `pg_stat_activity`; unbounded query and permission-limited visibility not modeled. Runtime not verified. | No server sessions; empty by design; query interrupt is not this feature's session action. |
| Locks / transactions | Partial source: 200 lock rows and at most one blocker per blocked PID. Runtime not verified. | Unsupported category currently returns empty vector without specific reason. |
| Size / workload | Top 40 relations and optional `pg_stat_statements` slice; UI limits also apply. Runtime not verified. | No relation/workload rows; local PRAGMA facts only. |
| Maintenance | VACUUM / ANALYZE / VACUUM ANALYZE source paths, database-level current UI. Runtime not verified. | VACUUM / ANALYZE / VACUUM ANALYZE source paths. Runtime not verified. |
| Session control | Cancel/terminate source paths, service invariant gap remains; no action runtime evidence. | Unsupported explicitly. |

MySQL/SQL Server: MonitoringService rejects unsupported providers; UI capability-specific unavailable behavior remains missing (`monitoring-feature-research.md` lines 50–57). No other provider support is proposed.

## Verification gates still needed (not run)

- Automated verification of service-level provider/read-only/PID/current-backend checks and cancel/terminate result meaning.
- Automated delayed-response test proving stale snapshots cannot overwrite newer results, plus polling stop/coalescing and failure-state behavior.
- Automated bounded-result and truncation/partial/unsupported state behavior.
- Native UI inspection of group navigation, target context, confirmations, stale/error/permission states.
- PostgreSQL runtime verification of session visibility, actions, limits, workload permissions and maintenance; SQLite runtime verification of PRAGMA/local metrics, unsupported categories and maintenance. Keep outcomes separate.
- Baseline and research explicitly report no tests/builds/app launch/provider scenarios run (`monitoring-baseline.md` lines 50–54; `monitoring-feature-research.md` line 86).

## Out of scope / unresolved decisions

- MySQL/SQL Server monitoring implementation; no support contract is established.
- Autonomous remediation or composite health score; findings remain evidence-linked advisory items (`monitoring-feature-research.md` lines 38–40).
- Whether cancellation is allowed on configured read-only connections; whether blocker detail must be a full graph; capability granularity; whether polling interval choice persists and repeated-failure backoff policy (`monitoring/AGENT_EVIDENCE.md` lines 70–72).
- Other administration belongs to separate surfaces; Phase F/product status updates must not be inferred as feature implementation.
