# Saved Tasks — feature research and recommendation

- Research date: 2026-09-24
- Repository baseline: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Scope: product role, task lifecycle, in-app scheduling, provider workflow, safety, reference patterns, and J01 roadmap drift. No implementation is included.

## Decision

**Treat current Saved Tasks as an early local convenience surface, not as completion of J01 Task/Job Scheduler.** Keep its reusable-task idea, but reconcile the already-wired top-level activity with the product roadmap before further expansion: J01 is deferred and specifies a runtime-owned scheduler/repository, target policy, one active run per task, timeout, cancellation, and outcome-linked history (`docs/goals/goal-full-product.md:897-926,1178-1180`). Do not claim the current UI's in-process tick satisfies that contract.

**Preserve task intent without mutating the user's query workspace.** Execute saved work from an isolated task context, retain request/task correlation, and never overwrite the active editor or change the selected connection just because a schedule became due. Target name, environment, SQL/action summary, and schedule must be explicit at create/apply time, especially for unattended work.

**Fix the confirmation, secret, and outcome boundaries before enabling unattended execution.** The existing shared read-only policy and quoted identifiers are useful controls, but they do not compensate for a weak task classifier, insufficient secret screening, success-on-dispatch history, or missing in-flight lifecycle.

## User model and task lifecycle

1. **Create from a concrete workflow.** A task should capture a specific SQL/script, backup, import/export, compare, or maintenance configuration from its source wizard; it should not turn every database action into a hand-edited string form.
2. **Review identity and risk.** Show the human connection name, environment, database, action, resolved schedule, timeout, destructive classification, and what unattended execution means. Never infer a target from the first connection without displaying and confirming it.
3. **Run deliberately.** Manual runs and scheduled runs use the same backend policy. Destructive operations require a target-bound confirmation; unattended destructive work requires a separate explicit policy opt-in. Disabled, deleted, disconnected, read-only, and unsupported-target states need visible outcomes.
4. **Track actual execution.** Run starts at accepted dispatch, remains Running while in flight, and reaches Success/Failed/Cancelled only from the matching completion event. Record actual start/end/duration and a redacted error. Prevent overlap for the same task; make timeout/cancel behavior explicit.
5. **Persist and inspect.** Keep definitions/history local or in the selected job store with versioned migration, bounded retention, searchable task list, edit/delete confirmation, next-run and last-run status. Secrets stay in secret storage; arbitrary SQL text is not a credential store.

## Recommended structure

```text
Tasks / Jobs
  Task list      type, target name/environment, state, next run, last result
  Task editor    task-specific workflow config, policy and schedule preview
  Run detail     actual output, structured error, cancellation/timeout, timestamps
  History        filter by task/target/status/time; retained under an explicit policy
```

Use task-specific source workflows for backup/export/maintenance and preserve those configurations, rather than storing a pseudo-export that only opens an interactive query. The current page has two creation buttons, no edit action, a fixed 60-second schedule button, immediate Delete, and cards showing the raw connection ID (`saved-tasks-baseline.md`).

## Reference workflow comparison

- DBeaver's official [Database Tasks documentation](https://github.com/dbeaver/dbeaver/wiki/Task-Management) provides a dedicated task list, task-specific creation wizards, edit/delete/run, grouping, run history, and per-task timeout. Its [Task Scheduler documentation](https://dbeaver.com/docs/dbeaver/Task-Scheduler/) exposes recurrence/start-time configuration and schedule edit/remove. The scheduler is backed by Windows Task Scheduler or `cron` on macOS/Linux; it has platform limitations. These are workflow references, not an instruction to adopt OS scheduling.
- DB Pro currently stores task definitions in local eframe storage and ticks from the egui app frame loop. The UI explicitly warns that schedules run only while the app is open (`saved_tasks_surface_view.rs:66-75`; `app_lifecycle.rs:142-160`). This should be an explicit product promise if retained; it does not meet J01's planned runtime worker architecture or durable background-job expectations.
- DBeaver distinguishes task definition, scheduler controls, manual run, task edit, and run logs. DB Pro currently places those concerns in one card list and records the dispatch response as a terminal run outcome (`tasks_view.rs:110-143`).

## Provider contract

| Workflow | PostgreSQL | SQLite | MySQL | SQL Server |
|---|---|---|---|---|
| Saved SQL | Shared QueryService; read-only policy source applies. Runtime unverified. | Shared QueryService. Runtime unverified. | Shared QueryService. Runtime unverified. | Shared QueryService. Runtime unverified. |
| Backup | PgDumpEngine source exists. | SqliteBackupEngine source exists. | BackupService explicitly rejects. | BackupService explicitly rejects. |
| Export | Current payload opens a bounded SELECT in Query workspace; user must manually export results. No scheduled file output. | Same source-only limitation. | Same source-only limitation. | `TOP 1000` source query; same no-file limitation. |
| Maintenance | VACUUM/ANALYZE. | VACUUM and ANALYZE with SQLite-specific target restrictions. | ANALYZE TABLE only; VACUUM rejected. | Explicitly unsupported. |

Do not advertise one provider's path as another provider's runtime guarantee. Every provider remains unverified in this research.

## Prioritized open work

1. **P1 — enforce destructive unattended-task policy with the shared classifier.** `SavedTaskPayload::is_destructive` is a small token blacklist, missing `CALL`, `DO`, and `EXECUTE`, which the shared safety classifier marks destructive (`core/domain/saved_task.rs:60-82`; `core/domain/safety.rs:45-79`). `enable_schedule`/`prepare_run` rely on the task predicate, and saved SQL calls `send_query_run` without the normal UI `hold_destructive_run` (`ui/saved_task_state.rs:78-95,137-165`; `ui/tasks_view.rs:159-181`; `ui/events_query_dispatch.rs:134-142,184-203`). With writable `full_access`, QueryService allows that statement. A saved `CALL` can therefore be scheduled without `allow_destructive` and without the normal destructive confirmation. Use one authoritative classification and reject or explicitly approve destructive tasks at both UI and service boundaries. Source-derived; no procedure was run.
2. **P1 — report completion, not dispatch, as task outcome.** SQL and backup task functions return success once a command is queued; `run_saved_task` immediately writes terminal run status/duration. Runtime SQL and backup completion are later events without a SavedTask correlation/update path (`ui/tasks_view.rs:132-143,159-202`; `runtime/worker.rs:1560-1565,1644-1698`; `ui/event_router.rs:33-37,84-86`; `core/domain/saved_task.rs:161-172`). Add a task/run ID to the async request lifecycle and transition through Running → Success/Failed/Cancelled using actual completion. Otherwise run history, retries, and duration are not truthful.
3. **P1 — isolate task execution from the interactive query workspace.** Every due SQL task changes active connection, replaces active query text, and switches workspace tab to Query (`ui/tasks_view.rs:163-178`). The scheduler is ticked from `prepare_frame`, irrespective of the selected activity (`app_lifecycle.rs:142-159`); `set_active_query_text` writes into the active document (`query_documents.rs:79-83,327-329`). This can overwrite unsaved user SQL. Execute in a dedicated task context and leave user workspace/connection untouched.
4. **P1 — make secret rejection cover actual task payloads.** The guard checks a short substring blacklist, while arbitrary SQL is serialized into local `eframe::Storage` (`core/domain/saved_task.rs:84-93,214-235`; `ui/tasks_view.rs:6-20`). A PostgreSQL statement `CREATE ROLE app LOGIN PASSWORD 'literal'` does not match any current needle and passes the source predicate; no disk encryption claim is made. The task store must not silently accept credential-bearing SQL under a “secrets stay on the connection” contract. Prefer secret references/parameterized task inputs and explicit storage guarantees; do not promise complete secret detection from a blacklist.
5. **P2 — align scheduler semantics and lifecycle.** The scheduler only runs while DB Pro is open, requests a 50ms repaint while any schedule is enabled, has no visible timeout/cancel or in-flight guard, and `SkipMissed` and `RunOnce` both emit one overdue run. Fix missed-run semantics, concurrency, cancellation, timeout, and sleep/wake behavior; reconcile the desired process lifetime with J01 before expanding scheduling (`saved_task.rs:267-306`; `ui/app_lifecycle.rs:142-160`; `goal-full-product.md:902-926`).
6. **P2 — make the task editor and target safe/useful.** UI can create only SQL and Backup tasks, cannot edit, exposes only a fixed 60-second schedule, and immediately deletes task plus history. Export payload only opens a query; Maintenance/Export drafts have no create button. The new-task fallback selects the first connection when no active connection exists; cards show connection ID and no environment. Provide target selection, connection/environment display, schedule options, task editing, explicit export output, and delete confirmation (`ui/tasks_view.rs:57-101`; `saved_tasks_surface_view.rs:77-95,112-165,168-260`; `core/domain/saved_task.rs:238-243`).
7. **P2 — reconcile J01 and capability claims.** The full-product goal says there is no task entity/scheduler/runtime loop and defers J01; current code has a task entity, local persistence, top-level activity and in-process scheduler. Refresh current-state docs without treating source wiring as completion, and keep provider/runtime evidence distinct (`goal-full-product.md:84-90,238-258,897-926,1178-1180`).

## Recommended decision boundary

Treat the current source as an early local-task implementation, not as a production scheduler. If the product adopts J01, use the roadmap's TaskService/TaskRepository/runtime-owned bounded executor and verify SQLite and PostgreSQL independently; MySQL/SQL Server support must follow explicit capabilities. Prioritize shared safety classification, truthful completion-linked run records, workspace isolation, and secret-safe payloads before allowing any unattended SQL. Do not claim tasks run in the background: the current contract is app-open only.

## Research limits

No code, tests, plans/status, or persistent task data changed. No build, automated test, native app, provider, task run, or SQL/backup/maintenance action was executed. P1/P2 entries are source-derived risks; no actual mutation, secret persistence, history misreport, or unsaved-query loss was reproduced.
