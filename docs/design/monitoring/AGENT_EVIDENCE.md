# Agent evidence — Monitoring feature research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · product research lane |
| Issue(s) | n/a — user requested the next Monitoring topic after Transfers |
| Task state | Done — native source baseline, feature assessment, and handoff recorded |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | `docs/files-tab-design` / no PR |
| Scope interpretation | Source-grounded assessment of the native Monitoring activity, UI/runtime/service/provider paths, action-safety contract, Phase D goal drift, and reference workflows. |
| Out of scope | Product implementation, code fixes, phase-goal/status transition, automated testing, live provider testing, native UI runtime evidence, and independent external approval. |

## 2. Progress checkpoint

- Current HEAD/source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Completed acceptance rows: [x] traced activity, state, runtime and provider service; [x] compared PostgreSQL/SQLite behavior independently; [x] read official PostgreSQL/SQLite docs and DBeaver session workflow; [x] assessed IA, action safety, polling/freshness, provider boundaries and documentation drift; [x] wrote all three research artifacts.
- Remaining acceptance rows: none for this research task. Source fixes, plan lifecycle work, and provider/native-UI verification are follow-up work; no issue was created.
- Findings / risks (source claims refer to the Baseline SHA):
  - **P1 action-safety gap, source-derived, not runtime-reproduced:** `MonitoringService::terminate_backend` checks PostgreSQL and positive PID, but not the configured read-only policy or the current backend; the UI-only current-row exclusion is not a service invariant. Cancel is immediate and lacks the confirmation specified by Phase D. PostgreSQL documents that `pg_cancel_backend` cancels a query and that `pg_terminate_backend` signals the backend; with the current default timeout, `true` only means the signal was sent, not that the backend has exited (`crates/core/src/application/monitoring_service.rs:146-185`; `crates/ui/src/monitoring_sessions_view.rs:163-168`; `crates/infrastructure/src/postgres/monitoring.rs:123-151`; `crates/ui/src/management_events.rs:181-194`).
  - **P2 freshness/load gap:** polling is enabled by default at a fixed five seconds, requests can overlap, and the reducer discards request IDs; a late result can replace a newer snapshot (`crates/ui/src/monitoring_state.rs:17-32`; `monitoring_activity_view.rs:61-73,198-205`; `event_router.rs:38-39`; `crates/runtime/src/worker.rs:1747-1762`).
  - **P2 feedback/provider-visibility gap:** Monitor-specific failure state is not assigned; PostgreSQL server-summary errors become `None`; SQLite lock/size categories use empty vectors without specific unavailable reasons; PostgreSQL session visibility is role-limited without a partial-visibility state. UI also polls unsupported MySQL/SQL Server connections even though `MonitoringService` rejects them (`crates/core/src/application/monitoring_snapshot.rs:30-52`; `crates/core/src/application/monitoring_service.rs:38-48`; `crates/infrastructure/src/sqlite/monitoring.rs:85-99`).
  - **P2 completeness gap:** PostgreSQL session query has no SQL row limit; locks/relation/workload have bounds but UI does not consistently disclose truncation, and the lock query returns at most one blocker PID for a blocked session (`crates/infrastructure/src/postgres/monitoring.rs:15-50,153-218,221-273,331-399`; `crates/ui/src/monitoring_snapshot_view.rs:147-203`; `monitoring_workload_view.rs:35-55`).
  - **P2 IA gap:** after a snapshot, the Monitor page appends Audit, FDW, replication, event-trigger and PostgreSQL-settings surfaces (`crates/ui/src/sidebar_view.rs:152-200`).
  - **P2 capability/roadmap drift:** `ServerSessions` remains coupled to `UserService` user listing, while Monitor uses driver switches; Phase D and capability matrix still describe monitoring as absent (`crates/core/src/domain/capabilities.rs:151-169`; `core/application/user_service.rs:41-65`; `docs/goals/goal-phase-d-monitoring.md:10,105-120`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §8).
- Tests already run: none. No build, unit/integration test, native app launch, PostgreSQL/SQLite provider test, or destructive action was executed. Source tests were not run.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | none — research artifacts are uncommitted |
| File / surface inventory | `docs/design/monitoring/monitoring-baseline.md` — current implementation, provider split, action boundary, source drift; `monitoring-feature-research.md` — product model, IA, provider matrix, reference workflows and prioritized follow-up; `AGENT_EVIDENCE.md` — this handoff. |
| Acceptance mapping | “Monitor” topic → all three research artifacts; Monitoring top-level role and groups → feature-research decision; evidence and safety → baseline plus P1/P2 items; PostgreSQL/SQLite → separate source matrix; reference workflow → official DBeaver/PostgreSQL/SQLite sources. |
| Commands and counts | No build/test/runtime commands executed. Source read/search and external documentation reads only. |
| CI run IDs / status | not run |
| Known limitations | No runtime/provider evidence; P1 session-action boundary is a source finding only; no independent review; implementation follow-up remains. |
| Migrations / config implications | none — no source or persistent data changed |
| Out-of-scope changes | No code, capability matrix, Phase D goal, release limitations, plan status, UI, or runtime behavior changed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (source baseline; research documents are uncommitted) |
| Verdict | n/a — research handoff, not independent code review or implementation approval |
| P0 / P1 / P2 counts | introduced by this documentation-only change: 0 / 0 / 0; present at source baseline: 0 / 1 / 6. Findings are source-level product/safety gaps, not reproduced incidents. |
| Findings | See §2 and `monitoring-feature-research.md` “Prioritized open work”; no code verdict is asserted. |
| CI disposition | not run — documentation-only research |
| Next task(s) unblocked | Reconcile Phase D's stale “no implementation” table and the capability matrix; plan service-level session-control invariants, polling freshness/error states, provider-specific empty/permission states, bounds, and focused Monitoring IA. Keep PostgreSQL, SQLite and native UI runtime gates separate. No issue activated in this task. |

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Repository references, all at exact SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`:
  - UI route and composition: `crates/ui/src/activity_bar_view.rs:92-96`; `sidebar_view.rs:123-155,152-200`; `monitoring_surface_view.rs:19-85`; `monitoring_snapshot_view.rs:5-38,107-203`; `monitoring_sessions_view.rs:85-171`; `monitoring_header_view.rs:18-67`; `monitoring_confirmation_view.rs:37-121`; `monitoring_state.rs:17-32`; `monitoring_activity_view.rs:61-73,137-205`; `monitoring_workload_view.rs:20-55`.
  - Core model and service: `crates/core/src/domain/monitoring.rs:5-173`; `application/monitoring_service.rs:38-185`; `application/monitoring_snapshot.rs:30-72`; `domain/capabilities.rs:151-169`; `application/user_service.rs:41-65`.
  - Provider/runtime: `crates/infrastructure/src/postgres/monitoring.rs:15-50,86-218,221-329,331-455,516-521`; `infrastructure/src/sqlite/monitoring.rs:47-152`; `crates/runtime/src/api.rs:1005-1071`; `runtime/src/worker.rs:1747-1880`; `crates/ui/src/event_router.rs:38-39`; `management_events.rs:20-44,181-194`; `crates/runtime/src/lib.rs:267-273`.
  - Product documents: `docs/goals/goal-phase-d-monitoring.md:10-12,105-120,122-189,191-228`; `docs/goals/goal-full-product.md:221-258`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §8; `docs/plans/FEATURE_LIFECYCLE.md:51-78,80-94`.
- External references read 2026-09-24:
  - https://github.com/dbeaver/dbeaver/wiki/Session-Manager-Guide
  - https://dbeaver.com/docs/dbeaver/Database-driver-PostgreSQL/
  - https://www.postgresql.org/docs/current/monitoring-stats.html
  - https://www.postgresql.org/docs/current/functions-admin.html#FUNCTIONS-ADMIN-SIGNAL
  - https://www.sqlite.org/pragma.html
- Factual findings: Monitor is a real wired activity; PostgreSQL and SQLite have distinct provider paths; session management, workload display, local SQLite facts, maintenance and administrative actions exist in source. The source does not meet the Phase D document's full interaction/safety/error-state target. No runtime result is claimed.
- Inference: polling overlap can produce stale overwrites because dispatch has no in-flight guard and the reducer discards IDs; no reproduction was performed. The service-level read-only/current-session omission is a source safety-boundary finding; actual authorization and backend effects depend on provider roles and were not exercised. The empty/error/permission findings describe missing UI distinctions, not proven inaccessible server data.
- Decision / recommendation: keep Monitoring on the top-level rail; refactor its IA around Overview, Sessions, Active Queries, Locks & Transactions, Size & Stats, Maintenance; move unrelated admin surfaces out; close service-level action checks and fix polling/freshness/feedback before declaring the phase complete.
- Unresolved questions: product policy for cancel actions on read-only connections; whether Monitor supports multi-blocker detail or a summarized blocker; which providers should receive explicit monitoring capabilities; exact policy for polling interval persistence and repeated errors.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã khảo sát Monitoring tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và ghi lại ba tài liệu: baseline mã nguồn, đánh giá sản phẩm, và evidence handoff. Monitoring đã có luồng native cho PostgreSQL và một tập số liệu file/PRAGMA riêng cho SQLite; tài liệu Phase D và capability matrix đã lỗi thời. Phát hiện một khoảng trống P1 ở ranh giới an toàn của thao tác cancel/terminate, cùng sáu khoảng trống P2 về polling/freshness, lỗi/quyền, giới hạn dữ liệu, IA, capability và roadmap. Không sửa code, không chạy test/build, không khởi chạy UI và không kiểm thử provider; mọi nhận định hành vi là bằng chứng source, không phải runtime evidence.
