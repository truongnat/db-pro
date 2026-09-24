# Saved Tasks — source baseline

- Source date: 2026-09-24
- Baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: native Rust/egui Saved Tasks UI, local persistence, scheduling, query/backup execution path, provider support, safety and roadmap drift. Source claims refer to this SHA.
- No source code, tests, plans/status, or runtime state were changed or exercised for this research.

## Finding

Saved Tasks is already a native, locally persisted task list with an in-process scheduler. It is not the runtime-owned, durable job system described by J01; scheduled runs occur only while the desktop app is open. The current source has four material source-level risks: the secret blacklist accepts common SQL password syntax that is then stored as task payload; destructive-task classification misses commands such as `CALL` that the shared SQL safety classifier considers destructive; run history records dispatch acceptance as success before async database/backup completion; and scheduled SQL replaces the active query buffer and switches the user's selected connection/workspace tab.

## Surface, state, and persistence

- `crates/ui/src/activity_bar_view.rs:85-96` exposes Saved Tasks as a top-level rail activity. `tasks_view.rs:23-55` renders and reduces actions; `saved_tasks_surface_view.rs:24-49,66-165,168-287` renders empty state, task editor, cards, schedules and recent runs.
- The domain supports SQL, Export, Backup, and Maintenance payloads (`crates/core/src/domain/saved_task.rs:28-48`). The current header only offers “New SQL task” and “New backup task”; cards offer Run, Schedule 60s/Disable schedule, and immediate Delete. There is no edit action or scheduler editor. Export tasks only dispatch a bounded SELECT and instruct the user to use Export on results; the saved-task path does not write an export file (`saved_tasks_surface_view.rs:77-95,221-260`; `tasks_view.rs:205-227`).
- Saved task records reference a connection ID and persist payload, schedule, last run, and run history (`saved_task.rs:161-200`). The UI stores serialized `SavedTaskStore` under `dbpro.native.saved-tasks-v1` using `eframe::Storage`; app `save` calls that persistence path and startup restores it (`tasks_view.rs:6-20`; `app_lifecycle.rs:3-15,25-37`; `app_state.rs:11-18`). Payloads are not stored in the connection secret store.
- The UI creates a SQL/backup draft with the active connection ID, or falls back to the first connection in the catalog if none is active (`tasks_view.rs:57-79`). Cards identify the target by raw connection ID, not name/environment (`saved_tasks_surface_view.rs:168-185`).
- The surface says the scheduler runs only while the app is open and there is no background daemon (`saved_tasks_surface_view.rs:66-75`). `prepare_frame` ticks it every frame and requests repaint every 50ms while any schedule is enabled (`app_lifecycle.rs:142-160`).

## Execution and safety

- Manual and scheduled runs pass through `SavedTaskState::prepare_run`, then `run_saved_task` immediately calls a dispatch function, records its synchronous `Result`, and sets feedback (`saved_task_state.rs:137-193`; `tasks_view.rs:104-143`). The record's start/finish time therefore measures dispatch work for async SQL and Backup tasks, not database execution duration.
- SQL saved tasks set the active connection, replace active query text, select Query workspace, and call `send_query_run` directly (`tasks_view.rs:159-182`). Normal query dispatch first calls `hold_destructive_run`; saved-task SQL does not (`events_query_dispatch.rs:134-142,184-203`). `set_active_query_text` changes the active query document (`query_documents.rs:79-83,327-329`). Since scheduled tasks tick from the app's frame loop, this can replace an unsaved user query even when the Tasks activity is not open.
- `SavedTaskPayload::is_destructive` is a keyword list (`INSERT`, `UPDATE`, `DELETE`, `DROP`, `TRUNCATE`, `ALTER`, `CREATE`, `GRANT`, `REVOKE`) and exact token split, not the shared SQL classifier (`saved_task.rs:60-82`). It omits `CALL`, `DO`, and `EXECUTE`, which the shared classifier marks `Destructive` (`crates/core/src/domain/safety.rs:45-79`). For a writable connection, QueryService builds `ConnectionSafetyPolicy::full_access`; that policy allows destructive work. A saved `CALL ...` is therefore not blocked by the task's destructive-scheduling/confirmation check and bypasses the regular UI confirmation path (`query_service.rs:129-160`; `domain/safety_policy.rs:23-44,53-76`; `tasks_view.rs:110-181`). This is a source-derived unapproved-schedule risk; no SQL was executed.
- The “no embedded secrets” guard scans serialized payload text for only `password=`, `password\":`, `secret=`, `api_key`, and `private_key` (`saved_task.rs:84-93`). PostgreSQL SQL such as `CREATE ROLE app LOGIN PASSWORD 'literal'` contains none of those exact markers; source logic therefore accepts it. The accepted payload is serialized into eframe storage. At-rest storage encryption was not inspected; no disk exposure was observed. Existing app test covers `password=x`, not PostgreSQL `PASSWORD '...'` syntax (`ui/src/app_tests.rs:6518-6551`).
- `dispatch_sql_task` and `dispatch_backup_task` return “Dispatched …” once the runtime command is accepted. `run_saved_task` records this as `Success` before the operation result arrives (`tasks_view.rs:132-143,159-202`). Runtime SQL query completion and backup completion are asynchronous events (`runtime/src/worker.rs:1560-1565,1644-1698`); their UI handlers update query/feedback state but no SavedTask run correlation/update was found (`ui/src/event_router.rs:33-37,84-86`; `operation_events.rs:18-20`; `SavedTaskRun` has no request ID, `saved_task.rs:161-172`). A later database/backup failure can therefore coexist with a “success” task history record; dispatch duration is not execution duration.
- Manual destructive confirmation is conditional on the general `confirm_destructive_queries` setting and task keyword classifier; scheduled destructive tasks require `allow_destructive`, but the UI has no control for that flag. `enable_schedule` rejects detected destructive payloads (`saved_task_state.rs:78-95,137-165`; `saved_task.rs:139-157,214-228`). Existing connection safety remains in QueryService for configured read-only connections (`query_service.rs:129-164`); it does not replace an unattended-task opt-in contract.
- SQL target quoting for export/maintenance validates at most three qualified identifier parts, rejects empty/control parts, and uses provider-specific quote escaping. Export format is restricted to CSV/TSV/JSON. Maintenance support is explicit by provider (`saved_task_sql.rs:5-98`).

## Scheduler and run-history semantics

- `TaskSchedule` stores interval, next/last scheduled timestamps, missed-run policy, retry count/limit, and `allow_destructive`; UI enables only a fixed 60-second interval (`saved_task.rs:113-157`; `tasks_view.rs:92-101`; `saved_tasks_surface_view.rs:242-250`).
- `SavedTaskStore` retains up to 200 run records; the surface renders at most 12 recent runs and truncates messages to 80 characters (`saved_task.rs:195-205,309-333`; `saved_tasks_surface_view.rs:263-287`). Deleting a task also deletes its run records (`saved_task.rs:238-243`) and has no confirmation in the current card action.
- Both `MissedRunPolicy::SkipMissed` and `RunOnce` branches set the next time to `now + interval` and push one task ID (`saved_task.rs:267-306`). Thus their current observable behavior is identical for overdue tasks; `SkipMissed` does not skip the overdue execution described by its name.
- There is no in-flight-task marker, timeout, or Saved Tasks cancellation/result correlation in the model/UI path. An asynchronous operation that outlasts its interval can be dispatched again. The roadmap's one-active-run limit is not implemented here (`saved_task.rs:161-172,267-329`; `tasks_view.rs:104-143`; `goal-full-product.md:902-920`).

## Provider matrix

| Workflow | PostgreSQL | SQLite | MySQL | SQL Server |
|---|---|---|---|---|
| Saved SQL task | Uses shared QueryService; runtime not exercised. Configured read-only policy still applies, but task-specific destructive gate is keyword-based. | Same shared QueryService path; runtime not exercised. | Same shared QueryService path; runtime not exercised. | Same shared QueryService path; runtime not exercised. |
| Backup | `PgDumpEngine` source wired (`runtime/src/lib.rs:237-252`). | `SqliteBackupEngine` source wired. | Explicitly unsupported by BackupService. | Explicitly unsupported by shared BackupService (`backup_service.rs:39-60`). |
| Export payload | Builds SELECT with quoted identifier and row limit, then prompts manual export; not an unattended file export. | Same behavior with SQLite identifier quoting. | Same query-source behavior. | SQL Server `TOP 1000` query-source behavior. |
| Maintenance | VACUUM/ANALYZE, optional identifier target. | VACUUM database-only; ANALYZE optional target. | ANALYZE TABLE with required target; VACUUM unsupported. | Explicitly unsupported (`saved_task_sql.rs:17-39`). |

Provider support above is source-level only. No PostgreSQL, SQLite, MySQL, or SQL Server runtime scenario was run.

## Source-document drift

- `docs/goals/goal-full-product.md:84-90,897-926,1178-1180` says no scheduler/task entity/runtime loop exists and defers J01, with a runtime-owned service/repository, single active run, timeout, policy, history and cancellation contract. Current source already has a top-level in-process Saved Tasks activity and local scheduler.
- `docs/goals/goal-full-product.md:238-258` explicitly says Tasks/Jobs is deferred and has no scheduler. This current-state statement is stale; J01's target architecture and safety gates remain unfulfilled.
- No J01 plan/status transition was made. This baseline is not runtime verification or a claim that the feature is lifecycle-complete.

## Verification

- Source-only inspection at SHA `b0500b9a7ecbe37b454f3d917881154c5f7a403c`.
- Official DBeaver Database Tasks and Task Scheduler documentation read; URLs are in `saved-tasks-feature-research.md`.
- No tests/builds run; no native UI launched; no database task, scheduled SQL, backup, or maintenance operation exercised. All behavior and risk statements are source-derived.
