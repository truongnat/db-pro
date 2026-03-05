# Sprint 1 - PostgreSQL Real-World Test Matrix (S1-01)

- Owner: `truongnat`
- Ticket: `S1-01`
- Project board: `DB Pro MVP`
- Goal: verify query lifecycle reliability on real PostgreSQL workloads before closing stability blockers.

## 1) Scope

This matrix focuses on failure-prone runtime behaviors:
- connect reliability and latency
- statement timeout enforcement
- lock timeout under contention
- wrapped pagination/filter/sort behavior used by DB Pro result grid

Out of scope for this slice:
- transaction UX features (`auto-commit`, manual commit/rollback controls)
- UI visual redesign

## 2) Test Environments

Run the matrix at least on these environments:

| Env ID | Type | Purpose |
|---|---|---|
| PG-ENV-01 | Local PostgreSQL | fast baseline, deterministic behavior |
| PG-ENV-02 | Remote PostgreSQL (same region) | network latency + auth variability |
| PG-ENV-03 | Remote PostgreSQL (high latency or throttled link) | timeout/cancel resilience under slower links |

## 3) Executable Scenario Set

Use script:

```bash
DATABASE_URL='postgresql://user:pass@host:5432/db' ./scripts/pg_matrix_run.sh
```

Outputs:
- summary markdown: `artifacts/pg-matrix/pg-matrix-<timestamp>-summary.md`
- detailed logs: `artifacts/pg-matrix/pg-matrix-<timestamp>.log`

Scenarios executed:

| Case ID | Scenario | Expected |
|---|---|---|
| PGM-01 | connectivity ping (`SELECT 1`) | success |
| PGM-02 | server version introspection | success |
| PGM-03 | statement timeout success path (`sleep` within timeout) | success |
| PGM-04 | statement timeout enforcement (`sleep` beyond timeout) | failure with `statement timeout` |
| PGM-05 | wrapped paging/filter/sort compatibility query | success |
| PGM-06 | lock timeout under table lock contention | failure with `lock timeout` |

## 4) Manual Scenario Extensions

These are manual (not fully automated in current script) and should be executed from DB Pro UI:

| Case ID | Steps | Expected |
|---|---|---|
| PGM-07 | Run query in DB Pro, click `Stop` while query is running | status exits `Running...`, no stuck UI |
| PGM-08 | Execute same query repeatedly (`>=20` cycles) with cancel on random cycles | no dead state, no orphan running indicator |
| PGM-09 | Run paging (`next/prev`) while quick filter/sort enabled | deterministic page transitions, no hang |
| PGM-10 | Run DDL under load then refresh navigator | no infinite navigator loading state |

## 5) Pass Criteria

- 100% pass on `PGM-01..PGM-06` for each target environment.
- `PGM-07..PGM-10` have no unresolved running state or infinite loading behavior.
- Any failed case includes:
  - DB URL target class (`local`/`remote`)
  - timestamp
  - reproduction SQL/steps
  - observed vs expected

## 6) Evidence Checklist

- [x] Attach summary markdown from script run.
- [ ] Attach detailed log file for each failed case.
- [ ] Attach UI capture for manual hang/cancel scenarios if observed.
- [ ] Update issue `#1` with evidence links and pass/fail verdict.

## 7) Execution History

| Run Timestamp (UTC) | Env | Result | Summary |
|---|---|---|---|
| 2026-03-05T15:03:11Z | PG-ENV-01 (local) | PASS 6/6 | `artifacts/pg-matrix/pg-matrix-20260305-220311-summary.md` |
| 2026-03-05T15:09:54Z | PG-ENV-01 (local) | PASS 6/6 | `artifacts/pg-matrix/pg-matrix-20260305-220954-summary.md` |

Remote execution status:
- `PG-ENV-02`: pending remote URL.
- `PG-ENV-03`: pending remote URL or latency-injected network profile.

## 8) Notes

- Script intentionally uses non-destructive setup (`CREATE TABLE IF NOT EXISTS dbpro_lock_test`).
- Artifacts are ignored by git (`artifacts/` in `.gitignore`).
