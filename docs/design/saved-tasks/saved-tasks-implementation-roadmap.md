# Saved Tasks — implementation roadmap

- Source baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Inputs: [saved-tasks-baseline.md](saved-tasks-baseline.md), [saved-tasks-feature-research.md](saved-tasks-feature-research.md), [AGENT_EVIDENCE.md](AGENT_EVIDENCE.md)
- Evidence boundary: source behavior below is at the stated SHA; findings are source-derived, not reproduced. Preserve all four inherited P1 items and three P2 items from research. Provider paths are source-level only.

## Decision

Treat current Saved Tasks as an early local convenience surface, not as completion of J01's runtime-owned durable scheduler. Keep reusable task intent, but isolate execution from interactive workspace state and require authoritative destructive classification, secret-safe payload handling and completion-linked outcomes before unattended execution is expanded. Do not imply background scheduling: current scheduling runs only while the app is open.

## Current state

### Implemented in source

- Native activity, locally persisted task definitions/history, app-frame scheduler, SQL/Backup creation and manual/scheduled dispatch exist (`saved-tasks-baseline.md` lines 12–18, 22).
- Domain supports SQL, Export, Backup and Maintenance payload types; shared QueryService, backup and provider-specific maintenance paths exist (`saved-tasks-baseline.md` lines 14–16, 28, 37–46).
- Store retains bounded run records; scheduler stores interval/missed-run/retry/destructive fields, but UI exposes a fixed interval (`saved-tasks-baseline.md` lines 30–35).

### Not implemented / incomplete

- Runtime-owned task service/repository and J01 lifecycle; one-active-run, timeout/cancel, truthful result history are absent (`saved-tasks-feature-research.md` lines 7–13, 20–21, 58–64).
- Task editor offers SQL/Backup only; no edit, configurable schedule, explicit file export, target/environment presentation or confirmed deletion (`saved-tasks-feature-research.md` line 59; baseline lines 14–18, 30–35).
- Durable task/run management beyond local eframe storage is not established; current work does not provide real export-to-file (`saved-tasks-baseline.md` lines 15–18, 30–35).

### Needs fix (source-derived, not runtime-reproduced)

- **P1:** task destructive token list misses `CALL`, `DO`, `EXECUTE`; scheduled SQL bypasses normal destructive query confirmation (`saved-tasks-feature-research.md` line 54; baseline lines 23–27).
- **P1:** task run history records dispatch acceptance as Success before SQL/backup async result (`saved-tasks-feature-research.md` line 55; baseline line 26).
- **P1:** scheduled SQL replaces active query text, connection and workspace selection (`saved-tasks-feature-research.md` line 56; baseline line 23).
- **P1:** credential-bearing SQL such as `PASSWORD 'literal'` evades short blacklist and is serialized to local storage; storage encryption is unknown (`saved-tasks-feature-research.md` line 57; baseline line 25).
- **P2:** scheduler lifecycle has no in-flight/timeout/cancel controls; missed-run policies currently behave identically (`saved-tasks-feature-research.md` line 58; baseline lines 18, 30–35).
- **P2:** task target/editor/delete/schedule/export workflow is incomplete and fallback may choose first connection (`saved-tasks-feature-research.md` line 59; baseline lines 15–17, 32–35).
- **P2:** J01/current capability claims are stale against partial source and must not be equated with completion (`saved-tasks-feature-research.md` line 60; baseline lines 48–52).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (inherited) | fix | Weak destructive predicate and bypass: baseline lines 23–27; research line 54 | Use the shared authoritative safety classifier for saved SQL; route manual/scheduled execution through equivalent policy; make unattended destructive opt-in explicit, durable and target-bound, with safe policy recheck at dispatch. | Shared classifier contract; before enabling scheduled SQL | `CALL`, `DO`, `EXECUTE` and other classifier-destructive statements are blocked absent explicit opt-in; normal confirmation is applied for manual run; schedule cannot bypass current connection policy. |
| P1 (inherited) | fix | Success recorded at dispatch, async result uncorrelated: baseline line 26; research line 55 | Give each task run an ID carried through runtime request/completion; transition Pending/Running to Success/Failed/Cancelled only on matching terminal event; measure actual duration and retain redacted error. | Runtime request/result correlation | A queued run remains Running; injected async SQL/backup failure becomes Failed rather than Success; unrelated completion cannot alter its record; duration spans execution. |
| P1 (inherited) | fix | Scheduler mutates active editor/connection/tab: baseline line 23; research line 56 | Run saved SQL in isolated task execution context; do not alter active query document, selected connection or workspace tab. | Runtime task execution context; run correlation | With an unsaved editor buffer and different selected target, due task completes against its configured target and leaves buffer, selected connection and tab unchanged. |
| P1 (inherited) | fix | Secret blacklist accepts SQL password syntax and payload persists: baseline line 25; research line 57 | Replace blacklist as a security guarantee. Prefer secret references/parameterized task inputs; reject or safely handle credential-bearing SQL and define storage protections/redaction. Do not claim full secret detection absent a principled mechanism. | Decide task SQL/input model and local storage guarantee | A credential-bearing task cannot silently persist as ordinary task text; accepted secret reference resolves securely at execution; task listings/history/errors contain no credential value. |
| P2 (inherited) | fix | No in-flight marker/timeout/cancel and same task can overlap; identical missed-run branches: baseline lines 30–35; research line 58 | Implement one active run per task, explicit timeout/cancel and truthful missed-run semantics; reconcile app-open versus runtime/background lifecycle with J01. | Runtime ownership and async completion | While a task is Running, scheduler does not dispatch another; timeout/cancel produces terminal state and accurate committed effects; SkipMissed skips overdue run while RunOnce emits at most one as specified. |
| P2 (inherited) | missing | No editor, fixed interval, immediate delete, raw connection ID, fallback target and SELECT-only “export”: baseline lines 14–17, 32–35; research line 59 | Add explicit target selection and target/environment labels, safe no-target state, edit/delete confirmation, schedule editor and task-specific export configuration/file destination. | Task-type workflow decision; safety policies | No target is silently selected; card identifies connection/environment; edits persist; deletion requires confirmation and states history effect; export run produces selected file rather than workspace handoff. |
| P2 (inherited) | upgrade | J01 describes runtime-owned scheduler/repository, scoped policy/history: research lines 7–13, 64; baseline lines 48–52 | If J01 is adopted, migrate local convenience tasks into versioned runtime-owned repository/executor with explicit retention and task-specific configuration, preserving truthful app-open/background promise. | Product decision on J01/process lifetime; safety and lifecycle fixes | Restart retains versioned definitions/history according to policy; scheduler behavior matches explicitly documented lifetime; task run state survives restart only if promised by contract. |
| P2 (inherited) | verification | Provider support varies: baseline lines 37–46; research lines 41–50 | Verify each task type/provider independently; keep backup rejects and maintenance support matrix explicit. | Core task lifecycle behavior | PostgreSQL/SQLite/MySQL/SQL Server outcomes are separately recorded for supported SQL paths; backup works only where explicitly supported; unsupported maintenance/backup never reports success. |
| P2 (inherited) | verification | Stale J01/current-state claims and no runtime evidence: research line 60; baseline lines 48–58 | Refresh J01/current capability claims against source, and record source, automated, provider-runtime and native UI evidence separately. | Work and verification gates | Docs say early local implementation until all gates pass; no completion inference from activity presence or local persistence. |

## Rollout / dependency order

1. Close destructive-classification/policy bypass and secret persistence boundary before allowing unattended SQL.
2. Add task/run correlation and truthful async terminal outcome; isolate execution from the interactive workspace before scheduled SQL is considered safe.
3. Move scheduling into an explicitly chosen lifecycle owner; then implement in-flight exclusion, timeout/cancel and distinct missed-run semantics.
4. Add safe target/editor/delete/schedule/export workflows and truthful labels; only then expand task types/providers.
5. Reconcile J01/current-state docs and collect focused automated, provider-runtime and native UI evidence independently.

## Provider/support matrix

| Workflow | PostgreSQL | SQLite | MySQL | SQL Server |
|---|---|---|---|---|
| Saved SQL | Shared QueryService source path; task-specific destructive gate is weak; runtime unverified. | Shared QueryService source path; runtime unverified. | Shared QueryService source path; runtime unverified. | Shared QueryService source path; runtime unverified. |
| Backup | PgDumpEngine source wired; runtime unverified. | SqliteBackupEngine source wired; runtime unverified. | Explicitly unsupported by BackupService. | Explicitly unsupported by BackupService. |
| Export payload | Bounded SELECT handoff to Query workspace; no scheduled file output. | Same source limitation. | Same source limitation. | SQL Server TOP 1000 source-query behavior; no file output. |
| Maintenance | VACUUM/ANALYZE source support. | VACUUM and ANALYZE with SQLite target constraints. | ANALYZE TABLE only; VACUUM rejected. | Explicitly unsupported. |

Provider distinctions are source-only; no runtime scenario was run (`saved-tasks-baseline.md` lines 37–46; `saved-tasks-feature-research.md` lines 41–50).

## Verification gates still needed (not run)

- Automated tests for classifier parity and unattended/manual policy; credential-bearing SQL handling and secret redaction; run-ID correlation and terminal transitions; workspace preservation; non-overlap/missed-run/timeout/cancel boundaries.
- Native UI review of target identity/environment, schedule configuration, deletion confirmation, run status and app-open/background promise.
- Provider runtime separately: PostgreSQL and SQLite saved SQL/backup/maintenance; MySQL/SQL Server saved SQL and supported/unsupported maintenance/backup behavior. No provider is runtime-qualified by source paths alone.
- Persistence migration/restart test if durable J01 repository is adopted.
- Source inputs explicitly report no tests/builds/native app/task or provider execution (`saved-tasks-baseline.md` lines 54–58; `saved-tasks-feature-research.md` lines 66–68).

## Out of scope / unresolved decisions

- Whether scheduling remains app-open-only or becomes background/runtime-owned; J01 is deferred and target architecture is not an implementation claim (`saved-tasks-feature-research.md` lines 7–13, 64; `AGENT_EVIDENCE.md` line 69).
- Which task types/providers enter initial J01, supported intervals/timeouts/missed-run/retry behavior, local storage encryption guarantee, and whether SQL tasks may include non-secret literals (`AGENT_EVIDENCE.md` line 69).
- Backup for MySQL/SQL Server; DB-to-DB transfer is not implied by saved task export.
- Do not promise task secrets can be found by a broader blacklist or that current local task storage is encrypted; research found encryption unverified (`saved-tasks-feature-research.md` line 57).
