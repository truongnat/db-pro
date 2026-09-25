# Transfers — implementation roadmap

- Source baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Inputs: [transfers-baseline.md](transfers-baseline.md), [transfers-feature-research.md](transfers-feature-research.md), [AGENT_EVIDENCE.md](AGENT_EVIDENCE.md)
- Evidence boundary: all current-state claims are source findings at the baseline SHA. P1 product gaps and P2 safety/completeness items are carried from research; no runtime outcome is asserted. PostgreSQL and SQLite support must be qualified separately.

## Decision

Keep Transfers as a top-level activity and replace harness-first content with real Import, Export, Backup, Restore and Jobs entry points. Keep current-result quick export as a distinct convenience; reuse BackupService rather than treating backup as generic row transfer. Fix file destination ownership before connecting adapters to user paths. Phase C does not include DB-to-DB transfer; Phase F compare/migration remains a related later destination, not a fake enabled workflow.

## Current state

### Implemented in source

- Transfers rail/palette route and harness-oriented surface exist; harness actions use synthetic data/temp files and in-memory transfer endpoints (`transfers-baseline.md` lines 14–20).
- Domain job model, synchronous generic row-batch loop, streaming CSV/TSV and JSONL adapters, XLSX target, and native loaded-result export exist (`transfers-baseline.md` lines 22–32).
- Backup/restore has a Settings UI/runtime path, with separate PostgreSQL and SQLite adapters (`transfers-baseline.md` lines 33, 41–49).

### Not implemented / incomplete

- Real file-to-table import and integrated live table/full-query streaming export have no end-to-end UI/runtime path (`transfers-feature-research.md` lines 84–87; baseline lines 41–45).
- Transfers has no durable jobs, integrated row/byte progress/cancel or persisted history; current harness list is transient (`transfers-baseline.md` lines 20, 26–34; research lines 57–61).
- XLSX target buffers all rows; generic row loop has no byte budget (`transfers-baseline.md` lines 27–30; research line 90).
- Backup/restore remain a Settings detour; PostgreSQL restore is non-transactional and may leave partial state; SQLite restore requires disconnect and staged validation (`transfers-baseline.md` lines 33, 46).

### Needs fix (source-derived, not runtime-reproduced)

- **P2 safety:** transfer adapters' cancellation/failure cleanup unconditionally removes final destination; existing user file may be removed if adapter is pointed at it (`transfers-baseline.md` lines 35–38; research line 88). Current use is synthetic harness; do not wire to user paths first.
- **P2 provider UX:** COPY output is PostgreSQL syntax but visible without provider gate (`transfers-feature-research.md` line 91; baseline lines 31, 44–45).
- **P2 documentation drift:** Phase C/current capability docs stale on rail/job/readers/query export; correct claims without over-crediting scaffolding (`transfers-feature-research.md` line 92; baseline lines 51–60).
- **P1 product gaps:** import into PG/SQLite and export beyond loaded result are unimplemented user outcomes, not regressions (`transfers-feature-research.md` lines 86–87).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P2 (inherited) | fix | Cleanup removes final destination during failure/cancellation: baseline lines 35–38; research line 88 | Establish explicit no-overwrite/replace policy and ownership-safe staging; cleanup may remove only artifacts created by this job; preserve existing destination on any pre-publication failure/cancel. Apply to delimited and JSONL/XLSX writers before user paths. | First; before wiring any destination picker | With preexisting destination, cancelled/failed job leaves original bytes unchanged; without existing destination, partial artifacts are cleaned; successful publication follows explicit overwrite confirmation/policy. |
| P2 (inherited) | fix | COPY format exposed on non-PG: baseline lines 31, 44–45; research line 91 | Gate COPY export to PostgreSQL and label script semantics; hide or explain unsupported on SQLite and other non-PG drivers. | Provider capability source | PostgreSQL offers COPY; SQLite cannot generate it; unsupported selection is unavailable with explanation and does not emit PG syntax. |
| P1 (inherited product gap) | missing | No live database import UI/runtime; generic readers are not DB adapters: baseline lines 28, 41–44; research lines 39–43, 86 | Implement CSV/TSV/JSONL (and XLSX only if deliberately selected) file-to-table flow: parse/preview, target/schema/table, column/type mapping, explicit create-table DDL preview, conflict/error policy, parameterized writes, transaction/commit disclosure, bounded error-row report. | File safety fix; provider mutation/parameterized path; format commitment | User selects real file and PG/SQLite target; preview/validation maps columns; values use parameters not interpolated SQL; imported rows appear in target with declared transaction/conflict outcome and actionable row errors. |
| P1 (inherited product gap) | missing | Current result quick export is only loaded rows; no live source/stream in Transfers: baseline lines 31–32, 41–45; research lines 45–49, 87 | Implement export from loaded result, table, selection or query with full-scope streaming, explicit format/destination and row-scope disclosure; preserve quick export as shortcut and safe atomic publication. | File safety fix; provider read/stream APIs; byte-budget policy | Large table/query exports all rows without materializing whole result; UI states source/scope and produced row count; cancellation preserves prior destination and reports completion state. |
| P2 (inherited) | missing | Transient harness jobs, synchronous core progress, backup events reduce to feedback: baseline lines 20, 26–34; research lines 57–61, 89 | Build runtime job lifecycle for Import/Export/Backup/Restore with durable bounded history, correlation, rate-limited event progress, cancellation, redacted details and actual provider commit/rollback state. | Real operation endpoints; runtime cancellation map; truthful per-provider operation semantics | Active and completed work appears in Jobs; restart preserves retained history; cancel/failure status matches actual committed/rolled-back prefix; no row data, secrets or unnecessary absolute paths appear in history. |
| P2 (inherited) | fix | Row-only ceiling and XLSX all-row buffering: baseline lines 27–30; research line 90 | Add byte budget and bounded-memory writer/read policies; replace or explicitly constrain XLSX buffering with a clear preflight size limit. | Format support decision; streaming paths | A dataset exceeding configured byte/memory limit is rejected before unbounded accumulation or processed within measured bound; XLSX never silently buffers arbitrarily large input. |
| P2 (inherited) | missing | Backup/restore deep-link to Settings, no integrated job/progress: baseline lines 33, 46; research lines 51–61 | Add in-activity provider-aware Backup/Restore workflow or tightly integrated route to shared BackupService; display target, prerequisites, destructive scope, active SQLite disconnect requirement and real limitations. | Job lifecycle; existing BackupService capabilities | Restore review names provider/connection/database/file and irreversible scope; PG prerequisite/non-transactional risk and SQLite disconnect/validation requirements shown before dispatch; job outcome tied to worker completion. |
| P2 (inherited) | upgrade | Harness is primary surface despite being synthetic: baseline lines 16–20; research lines 9–13, 26–37 | Replace normal user path with Overview/Import/Export/Jobs and backup/restore entry; move any retained synthetic harness to clearly developer-only location; link Phase F destinations without fake enabled actions. | Real entry workflows; route migration | Normal Transfers surface offers operational workflows only; synthetic controls cannot be mistaken for user data movement; related deferred features show explicit status. |
| P2 (inherited) | verification | Stale Phase C/capability/limitations docs: baseline lines 51–60; research lines 92–96 | Update current-state evidence while retaining limitations (especially PG restore non-transactionality); track source, automated, PG runtime, SQLite runtime and native UI evidence separately. | Work and provider gates | Docs distinguish core adapters/harness from end-to-end outcomes; no completion statement until Phase C DoD and both provider-runtime evidence sets pass. |
| Proposed P1 | verification | Current research no test/build/UI/provider run: baseline lines 62–66; research lines 98–100 | Before real-path release, verify destination safety, parameterized import and cancellation/commit behavior with targeted automated cases and provider scenarios. | Safety fix and import/export implementation | Pre-existing file survives every failure/cancel; imported data and committed prefix match declared policy; PG and SQLite results are separately recorded. |

## Rollout / dependency order

1. Fix destination ownership/publication and COPY gating before any user-selected file output.
2. Implement bounded streaming/source-target primitives, parameterized import and explicit PG/SQLite transaction/error semantics; then wire the import wizard.
3. Add full-source streaming export, explicit row scope and destination policy; retain loaded-result quick export.
4. Build correlated runtime job lifecycle and persist truthful progress/history across import/export and existing backup/restore work.
5. Integrate Backup/Restore safely, preserving provider-specific limitations; replace harness-first content and reconcile docs only as actual workflows land.
6. Complete automated, PostgreSQL, SQLite and native UI gates independently before any Phase C completion claim.

## Provider/support matrix

| Workflow | PostgreSQL | SQLite |
|---|---|---|
| File import to live table | No end-to-end path; pending. | No end-to-end path; pending independently. |
| Table/full-query export | Pending integrated streaming source; loaded-result quick export exists only. | Same source-level quick export; full streaming pending independently. |
| COPY | PostgreSQL-specific output exists in quick export; currently not visibly gated. | Unsupported as SQLite import command; must hide/explain. |
| Backup | Source uses external pg_dump tools; client tools required on PATH. Runtime not verified. | Source uses VACUUM INTO with staged publication. Runtime not verified. |
| Restore | Source uses psql/pg_restore; non-transactional, partial DB state possible on failure. Runtime not verified. | Requires active connection disconnect; staged file `quick_check` validation before publication. Runtime not verified. |
| Durable transfer jobs | None found at baseline. | None found at baseline. |

MySQL: not in Phase C supported-provider contract; shared BackupService rejects it (`transfers-baseline.md` line 49). Do not imply additional provider coverage.

## Verification gates still needed (not run)

- Automated destination safety checks covering existing destination, cancellation, failure before publication, overwrite confirmation and cleanup ownership.
- Automated import/export contract checks for parameterized cell writes, row/byte bounds, source row scope, cancellation and truthful job outcomes.
- PostgreSQL runtime verification of import/export full row scope, COPY gating, backup prerequisite, restore partial-failure consequences and job progress/cancel.
- SQLite runtime verification independently for import/export, no COPY, VACUUM INTO backup, disconnect/staged restore validation and job outcomes.
- Native UI review of import mapping/DDL/conflict preview, export scope, backup/restore target confirmation, Jobs history and error reports.
- Baseline/research report no tests/build, app launch or PG/SQLite runtime scenarios (`transfers-baseline.md` lines 62–66; `transfers-feature-research.md` lines 98–100).

## Out of scope / unresolved decisions

- Database-to-database transfer in Phase C; research explicitly reserves it outside first delivery (`transfers-feature-research.md` lines 26–37).
- Phase F Schema Compare/Data Compare/Migration implementation; Transfers may expose related destinations only.
- File format commitments and initial import support scope (CSV/TSV/JSONL/XLSX), conflict policy, whole-file versus batch commit, and create-table policy details (`transfers-feature-research.md` lines 39–43, 45–49).
- Exact byte budgets, cancellation semantics and whether XLSX is bounded or explicitly limited; no large-run claim before measurement (`transfers-feature-research.md` line 90).
- Do not claim PostgreSQL restore is transactional/atomic or cancellable; current source limitation remains until separately changed and verified (`transfers-baseline.md` lines 46, 58).
