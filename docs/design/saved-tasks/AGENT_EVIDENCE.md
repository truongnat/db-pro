# Agent evidence — Saved Tasks feature research

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Main · product research lane |
| Issue(s) | n/a — user requested Saved Tasks after Security |
| Task state | Done — native source baseline, feature assessment, and handoff recorded |
| Baseline SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Branch / PR | `docs/files-tab-design` / no PR |
| Scope interpretation | Source-grounded assessment of Saved Tasks UI, local persistence, in-app scheduler, SQL/backup/maintenance execution, safety, provider matrix, workflow references, and J01 roadmap drift. |
| Out of scope | Product implementation, code fixes, roadmap/status transition, automated tests, live provider/task execution, native UI runtime evidence, and independent external approval. |

## 2. Progress checkpoint

- Current source baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Acceptance rows: [x] traced task payload/state/storage and scheduler; [x] traced SQL/backup async paths, destructive guard and result events; [x] compared official DBeaver task list/scheduler workflow; [x] assessed provider differences, task editor, persistence, safety and roadmap drift; [x] wrote all three artifacts.
- Remaining rows: none for this research task. Implementation, J01 plan/status work, and provider/native-UI runtime verification remain separate.
- Findings / risks (source claims refer to the baseline SHA):
  - **P1 destructive-task policy bypass, source-derived, not runtime-reproduced:** the task predicate is a token blacklist that misses `CALL`, `DO`, and `EXECUTE`, while the shared SQL safety classifier rates them destructive. Scheduled-task validation relies on the weaker predicate, saved SQL bypasses normal UI confirmation, and writable QueryService full-access policy allows execution (`crates/core/src/domain/saved_task.rs:60-82`; `core/src/domain/safety.rs:45-79`; `crates/ui/src/saved_task_state.rs:78-95,137-165`; `ui/src/tasks_view.rs:159-181`; `core/src/domain/safety_policy.rs:23-44,53-76`).
  - **P1 task outcome misreporting:** SQL/backup task handlers record success when dispatch is accepted, before async query/backup completion; no task/run request correlation or completion reducer was found (`crates/ui/src/tasks_view.rs:132-143,159-202`; `crates/runtime/src/worker.rs:1560-1565,1644-1698`; `crates/ui/src/event_router.rs:33-37,84-86`; `core/src/domain/saved_task.rs:161-172`).
  - **P1 interactive SQL loss risk:** scheduled SQL selects the target connection, replaces active query text and changes workspace tab while scheduler ticks from the app frame (`crates/ui/src/tasks_view.rs:159-178`; `crates/ui/src/app_lifecycle.rs:142-160`; `crates/ui/src/query_documents.rs:79-83,327-329`).
  - **P1 credential persistence risk:** the secret guard scans only five substrings; a common PostgreSQL `PASSWORD 'literal'` statement does not match them, while accepted payloads are serialized to eframe storage (`crates/core/src/domain/saved_task.rs:84-93,214-235`; `crates/ui/src/tasks_view.rs:6-20`). Storage encryption is unverified; no disk leak was observed.
  - **P2 scheduler/lifecycle and UX gaps:** scheduler is app-open only; scheduler fields are not user-configurable beyond a fixed 60-second interval; no task edit; immediate delete; current export task is only a SELECT handoff; no timeout/cancel/in-flight state; missed-run policy branches behave the same; UI falls back to first connection if no active target (`crates/ui/src/app_lifecycle.rs:142-160`; `tasks_view.rs:57-101,104-143,205-227`; `saved_tasks_surface_view.rs:66-75,77-95,221-260`; `core/src/domain/saved_task.rs:267-306`).
  - **P2 provider and roadmap drift:** backup is wired for PostgreSQL/SQLite but explicitly unsupported for MySQL/SQL Server; Maintenance is provider-specific; J01 docs say no task entity/scheduler though the native activity exists (`core/application/backup_service.rs:39-60`; `ui/saved_task_sql.rs:17-39`; `docs/goals/goal-full-product.md:84-90,238-258,897-926,1178-1180`).
- Tests already run: none. Existing saved-task unit/app tests were inspected but not run. No build, native app, SQL, backup, maintenance, or provider scenario was exercised.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` |
| Commit list | none — research artifacts are uncommitted |
| File / surface inventory | `saved-tasks-baseline.md` — current surface, scheduler, execution, provider and safety paths; `saved-tasks-feature-research.md` — product workflow, DBeaver references, provider matrix, prioritized work; this file — evidence handoff. |
| Acceptance mapping | Saved Tasks topic → three artifacts; persistence/execution/provider → baseline; DBeaver task/scheduler workflow → feature assessment; async status, editor side effects, destructive classifier, secret check → P1/P2 findings; J01 current-state drift → assessment and handoff. |
| Commands and counts | Source reads/search and official DBeaver docs reads only. No build/test/runtime command executed. |
| CI run IDs / status | not run |
| Known limitations | No runtime proof of mutation, secret storage, async task outcome, or unsaved editor loss; P1/P2 are source-derived. No independent review. |
| Migrations / config implications | none — no source or persistent task data changed |
| Out-of-scope changes | No code, tests, J01 goal, capability matrix, plan/status, runtime, database or UI behavior changed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `b0500b9a7ecbe37b454f3d917881154c5f7a403c` (source baseline; research artifacts uncommitted) |
| Verdict | n/a — research handoff, not independent code review or implementation approval |
| P0 / P1 / P2 counts | introduced by this documentation-only change: 0 / 0 / 0; present at source baseline: 0 / 4 / 3. Findings are source-level risks, not reproduced incidents. |
| Findings | See §2 and `saved-tasks-feature-research.md` “Prioritized open work”; no code verdict is asserted. |
| CI disposition | not run — documentation-only research |
| Next task(s) unblocked | Reconcile task work with deferred J01; enforce one destructive-task classifier; correlate run IDs to actual async completion; isolate scheduled execution from interactive editor; strengthen secret-safe task inputs/storage; implement target/environment, schedule, timeout/cancel, non-overlap and truthful missed-run semantics. Verify providers separately. |

## 5. Research / audit handoff

- Source date: 2026-09-24.
- Repository references, all at exact SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`:
  - UI/state/persistence: `crates/ui/src/activity_bar_view.rs:85-96`; `tasks_view.rs:6-253`; `saved_task_state.rs:7-193`; `saved_tasks_surface_view.rs:24-287`; `app_lifecycle.rs:3-15,25-37,142-160`; `app_state.rs:11-18`; `query_documents.rs:79-83,327-329`.
  - Domain/safety/provider: `crates/core/src/domain/saved_task.rs:28-93,113-157,161-333`; `core/src/domain/safety.rs:45-79`; `core/src/domain/safety_policy.rs:23-76`; `core/src/application/query_service.rs:129-164`; `core/src/application/backup_service.rs:39-60`; `crates/ui/src/saved_task_sql.rs:5-98`; `crates/runtime/src/lib.rs:237-252`.
  - Async completion: `crates/runtime/src/worker.rs:1560-1565,1644-1698`; `crates/ui/src/runtime_protocol.rs` query/backup event variants; `ui/src/event_router.rs:33-37,84-86`; `ui/src/operation_events.rs:18-20`; `core/domain/saved_task.rs:161-172`.
  - Product documents: `docs/goals/goal-full-product.md:84-90,238-258,897-926,1178-1180`; `docs/notes/PRODUCT_CAPABILITY_MATRIX.md` §13 (no J01 row found there; task scope is defined by full-product goal J01).
- External references read 2026-09-24:
  - https://github.com/dbeaver/dbeaver/wiki/Task-Management
  - https://dbeaver.com/docs/dbeaver/Task-Scheduler/
- Factual findings: current code persists tasks locally, schedules by in-app frame ticks, exposes SQL/Backup creation, and dispatches saved work into shared query/backup paths. PostgreSQL/SQLite have backup implementations; MySQL/SQL Server are rejected by shared BackupService. Runtime completion is separate from the task's immediate run record. No provider task was executed.
- Inference: missed async failures can leave a false Success in the task record; `CALL`/`DO`/`EXECUTE` can perform hidden mutations despite not meeting the local destructive predicate; scheduled SQL may overwrite user edits. These are directly derived from current control flow, not observed runtime events. Storage-at-rest encryption is unknown.
- Decision / recommendation: treat source as an early local-task surface, not completion of J01; block unattended task expansion until destructive-policy parity, truthful async result tracking, workspace isolation, and secret-safe storage are addressed.
- Unresolved questions: whether product intends the scheduler to be app-open-only or background; which task types/providers belong in first J01; storage encryption guarantee; supported schedule intervals, timeout and missed-run behavior; whether all SQL task types may contain non-secret literals by design.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã khảo sát Saved Tasks tại SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c` và ghi ba tài liệu. Mã nguồn hiện có danh sách task lưu cục bộ và scheduler chạy trong tiến trình app, nhưng chưa tương đương kiến trúc J01 runtime-owned. Có bốn rủi ro P1 source-level: bộ nhận diện destructive bỏ sót `CALL`/`DO`/`EXECUTE` nên có thể cho chạy lịch không có opt-in; lịch sử đánh dấu thành công ngay khi dispatch chứ chưa chờ kết quả; SQL task theo lịch ghi đè query đang mở và đổi connection; bộ lọc secret bỏ sót cú pháp `PASSWORD '...'` trong SQL được lưu. Các khoảng trống P2 gồm scheduler chỉ chạy khi app mở, policy/interval chưa cấu hình đầy đủ, miss-run policy trùng hành vi, thiếu timeout/cancel/non-overlap, task editor và provider matrix chưa hoàn chỉnh, cùng tài liệu J01 đã lỗi thời. Không chạy test/build, không khởi chạy UI và không thực thi SQL, backup hay maintenance.
