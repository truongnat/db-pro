# `human-session` — results template (**NOT RUN**)

**Status: NOT RUN. This directory currently contains no run record and no screenshots.**

This file is a template for the human operator who executes
`docs/release/0.1.0-interactive-verification-runbook.md`. It is deliberately committed empty so the
output location and format are unambiguous. **Do not fill this file with anything other than real
observations from a real session.** If you have not run the runbook, leave it as-is.

Fill in the blocks below and keep the headings, so a reviewer can diff an empty record against a
completed one.

---

## Session header

| Field | Value |
|---|---|
| Status | **NOT RUN** (replace with `RUN`) |
| Date / time | — |
| Tested SHA | — (the SHA the binary under test was built from; use `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20` only if you tested the CI artifact) |
| Artifact under test | — (e.g. `db-pro-v0.1.0-macos-arm64.tar.gz`, sha256 `8141d6b7…`, from CI run `34860902181`; or a local build with its own sha256) |
| Host / OS | — |
| Display resolution | — (1920×1080 is required for the V01-01 acceptance claim; state what you actually used) |
| Agent provider key used? | — (yes/no; if no, step 16 is `not run`) |
| Keychain prompt appeared? | — (which step, and what you answered) |

## Per-step results

| Step | Expected (runbook) | Observed | Result | Capture |
|---|---|---|---|---|
| 1 Launch | window renders, dark theme | — | not run | `01-launch-dark.png` |
| 2 Shell / activity bar | each activity opens, tooltips, status bar context | — | not run | — |
| 3 Settings / light-dark | immediate full re-theme to Light | — | not run | `12-settings-light.png` |
| 4 PostgreSQL connection | `Connection Verified`; bad password → clean error, no crash | — | not run | — |
| 5 Explorer (PG) | schema/tables/views expand; preview vs pinned tab | — | not run | `02-explorer-pg.png` |
| 6 PG query + Stop gate | 4 orders; no Stop control offered for PostgreSQL | — | not run | — |
| 7 SQLite connection | Browse works; Test/Save; connection in Explorer | — | not run | — |
| 8 Completion popup | `users` offered while typing | — | not run | `03-query-completion.png` |
| 9 Queries + grid | `SELECT 1;` → 1; `users` → 60 rows; filter/sort/page | — | not run | `04-query-results.png` |
| 10 Edit/insert/delete + apply | staged set applied exactly once; verified in `sqlite3` | — | not run | `05-table-edit.png` |
| 11 Conflict | 3-way dialog (Original/Local/DB); chosen action wins | — | not run | `06-conflict.png` |
| 12 Read-only attempt | write blocked at backend boundary; row unchanged | — | not run | — |
| 13 Destructive confirmation | prompt first; reject → nothing; confirm → executes | — | not run | — |
| 14 Introspection | columns/indexes/relations/triggers/DDL match fixture | — | not run | `07-schema-columns.png`, `08-schema-indexes.png` |
| 15 ER pan/zoom/LOD/search | LOD transitions; pan/zoom smooth; state preserved | — | not run | `09-er-compact.png`, `10-er-detailed.png` |
| 15b Large-schema (optional) | 200+ table pan/zoom + CPU/memory noted | — | not run | — |
| 16 Agent panel | `Preview` badge; inert without key; no fake success | — | not run | `11-agent.png` |
| 17 Export | file written, contents match the grid | — | not run | — |
| 18 Quit + relaunch | clean exit; connections + theme persist; no tab restore (`R-015`) | — | not run | — |

## Deviations and findings

Every step whose observation differed from the expectation, with severity (P0/P1/P2/P3 per the
brief's policy). If everything matched, write "no deviations observed" — do not leave it blank.

| # | Step | Observation | Severity | Notes |
|---|---|---|---|---|
| 1 | — | — | — | — |

## Screenshot index

Capture files themselves go in `docs/release/evidence/v01-runtime/screenshots/` (the sibling
directory, which is empty until a real session fills it). This table just records what was
captured.

| Filename | Step | Notes (resolution, annotation) |
|---|---|---|
| `01-launch-dark.png` | 1 | — |
| `02-explorer-pg.png` | 5 | — |
| `03-query-completion.png` | 8 | — |
| `04-query-results.png` | 9 | — |
| `05-table-edit.png` | 10 | — |
| `06-conflict.png` | 11 | — |
| `07-schema-columns.png` | 14 | — |
| `08-schema-indexes.png` | 14 | — |
| `09-er-compact.png` | 15 | — |
| `10-er-detailed.png` | 15 | — |
| `11-agent.png` | 16 | — |
| `12-settings-light.png` | 3 | — |

## What was updated after this run

- [ ] `docs/release/0.1.0-manual-smoke.md` — ticked only the observed items (`[~]` → `[x]`) and
      updated the tallies block + sign-off
- [ ] `docs/release/0.1.0-readiness.md` — GUI rows / `R-GUI-SMOKE`
- [ ] `docs/release/0.1.0-handoff.md` §5
- [ ] `docs/release/risk-register.md` — `R-GUI-SMOKE` (and `R-KEYRING-STALL` if the prompt recurred)
- [ ] `docs/release/evidence/v01-runtime/README.md` and `24-lifecycle-transitions.md` (re-evaluate
      candidates once the UI-runtime gate is satisfied)
- [ ] `docs/release/0.1.0-human-decisions.md` — `HD-007` may be closed by the owner
