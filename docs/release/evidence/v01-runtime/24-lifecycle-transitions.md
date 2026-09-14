# `v01-runtime` — Lifecycle transition record (brief §44)

- Assessed: 2026-09-14, after the `v01-runtime` evidence session.
- Authority: `docs/plans/FEATURE_LIFECYCLE.md` — state machine
  `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`, the four evidence
  levels (source / automated / provider runtime / UI runtime, never mixed), the provider matrix,
  and the completion gates.
- Code state: evidence was collected at HEAD `8e1d63f` (`RUNTIME_SESSION.md` header) and committed
  as `cd91e5f` (SQLite fixture), `2792176` (live PostgreSQL), `0dea3ae` (backup/restore +
  cancellation), `08c3e76` (packaged launch/state-dir/keyring/error-log), `0d7f6f1` (session log).
  `git diff --name-only 8e1d63f..0d7f6f1` shows **only** `docs/**` and `fixtures/**` — **no
  production code, manifest, script or workflow changed**. Artifact under test:
  `db-pro-v0.1.0-macos-arm64.tar.gz` (10,045,965 B, sha256 `8141d6b7…`), byte-identical to CI run
  `34860902181` for candidate `85a7fa3`.
- **Result: 0 transitions to `COMPLETED`.** Every candidate that was `RUNTIME_VERIFY` stays
  `RUNTIME_VERIFY`. The reason is uniform and stated once: `COMPLETED` requires **UI runtime
  evidence** — the native flow `egui → UiCommand → runtime worker → backend service → database →
  introspection → UiEvent → refreshed UI state` — and this session produced none, because the GUI
  is unreachable here (`providers/21`: Orca `runtime_unavailable`, AppleScript `-1743`,
  `screencapture` "could not create image from display", no accessibility tree). **No screenshot
  exists and none is claimed** (`screenshots/README.md`). Provider-level runtime evidence did
  improve materially (see the PostgreSQL column) — that is evidence level 3, not level 4, and it
  must not be written as UI runtime evidence. The lifecycle's separate **REVIEW** requirement
  (independent review; the V01 commits went straight to `main`, the Kilo reviewer is PR-only) is
  also still unsatisfied for the V01 candidates and is noted per row.

## Evidence levels actually reached by this session

| Feature group | Source | Automated | Provider runtime | UI runtime |
|---|---|---|---|---|
| PostgreSQL-facing work | yes | yes | **yes — new: 18/18 live on server 16.15** (`providers/07`) | **no** |
| SQLite-facing work | yes | yes | partially: fixture proven correct/deterministic (`providers/02`–`05`); no app-driven run | **no** |
| SSH/backup work | yes | 1 SSH test still `#[ignore]`d | **`BLOCKED`** — nine `DB_PRO_SSH_*` unset (`providers/07`, `09`) | **no** |
| ER/large-schema work | yes | yes (99 diagram tests) | n/a (synthetic) | **no** — frame-time/idle-resource claims still unmeasured |
| Agent work | yes | yes | **no** — no provider key was available (`GROQ_API_KEY` unset) | **no** |
| Visual/redesign work | yes | yes | n/a | **no** |

## Per-candidate record

| # | Candidate (plan directory) | State before | Evidence now present (this session) | Provider state | Runtime state | Transition made | Missing evidence / REVIEW requirement |
|---|---|---|---|---|---|---|---|
| 1 | Core Safety Hardening (`active/core-safety-hardening`) | `RUNTIME_VERIFY` | Automated: workspace suite 815/0/19 (candidate) / effective 833/0/1 with `DATABASE_URL` (`providers/08`, `09`). **New provider-runtime evidence:** the 18 `pg_integration` tests pass against a live PostgreSQL 16.15 server — transaction rollback, batch-failure/timeout, typed decoding (`providers/07`) | PG: **live automated PASS**; SQLite: automated PASS; **SSH: `BLOCKED`** | **Not observed** — no UI artifact | **None** | SSH live run; native UI traversal; interactive persistence. REVIEW: no independent review artifact exists for the V01 candidates |
| 2 | Table Data Editor Hardening (`active/table-data-editor-hardening`) | `RUNTIME_VERIFY` | Automated staged-mutation/conflict/rollback tests pass; **new**: live-PG rollback/commit paths among the 18 (`providers/07`); SQLite fixture with composite PK + triggers built for the interactive pass (`fixtures/smoke/sqlite/runtime_fixture.sql`) | PG: live automated PASS (mutation/rollback paths); SQLite: automated PASS + fixture ready | **Not observed** — no staged-edit/3-way-conflict walkthrough | **None** | Interactive walkthrough per `0.1.0-interactive-verification-runbook.md`; UI traversal |
| 3 | Query Editor Intelligence (`active/query-editor-intelligence`) | `RUNTIME_VERIFY` | Automated completion/diagnostics/saved-query tests pass; **new**: the 18 live-PG tests run (18/18), and cancellation capability gating was verified correct on every shipping path (`providers/15`) — no bug, no code change | PG: live automated PASS; cancellation `Unsupported` by design and correctly gated. SQLite: automated | **Not observed** — no native viewport record | **None** | Native viewport execution/completion/diagnostics record with a live provider; the source doc's own pending-live-evidence paragraphs are still unretracted |
| 4 | Agent Workflow (`active/agent-workflow`) | `RUNTIME_VERIFY` | Automated typed-provider/idempotency/confirmation tests pass | PG/SQLite: automated only; **no live provider call** (`GROQ_API_KEY` unset, `providers/01`, RT-38) | **Not observed** | **None** | Live provider E2E run; native panel walkthrough. **Agent remains Preview** by product policy regardless of verification state |
| 5 | P1 Large-Schema ER Architecture (`active/er-hardening-verification`) | `RUNTIME_VERIFY` (`PARTIAL`) | Automated: 99 diagram tests (layout timing, 3-tier LOD, BFS, spatial index, worker lifecycle) | Provider-neutral (synthetic + fixture-scale) | **Not observed** — "smooth" pan/zoom, idle CPU < 2%, memory stability still unmeasured | **None** | Frame-time and resource capture from a real window; capture belongs in the runbook steps 10–11 |
| 6 | RC1 Full Product QA (`active/rc1-full-product-qa`) | `RUNTIME_VERIFY` | Automated P1 regression tests; earlier P2 dispositions unchanged (`R-RC1-P2`); **new**: live-PG 18/18 is provider-level only | PG: live automated PASS; SQLite: automated | **Not observed** — `0.1.0-manual-smoke.md` is 0 passed / 165 blocked | **None** | The full smoke; live provider verification; the programme's own P2 gate checkbox remains unchecked |
| 7 | Native Visual Redesign (`active/native-visual-redesign`) | `RUNTIME_VERIFY` | Automated: 359 `db_pro_ui` tests | none new | **Not observed** — 0 of the 50 cited captures survive; GUI probed again and unavailable (`providers/21`) | **None** | All-surface light/dark traversal, 1920×1080 acceptance, provider review, independent review — with captures stored in-repo. `0.1.0-ui-visual-description.md` is code-derived and does not close it |
| 8 | S1–S7 schema work (`active/schema-regression`, `schema-columns-runtime`, `schema-indexes-runtime`, `schema-relations-runtime`, `schema-triggers-runtime`, `ddl-normalization`, `empty-schema-qualification`) | `RUNTIME_VERIFY` (S1–S7); `schema-regression`/`ddl-normalization` `REVIEW` | Automated SQLite introspection suites pass; **new**: live-PG introspection among the 18 (tables, indexes, FKs, triggers, views — `providers/07`). `ddl-normalization` and `schema-regression` still at `REVIEW`, which `COMPLETED` requires exiting first | PG: live automated PASS; SQLite: automated PASS | **Not observed** — no native-UI traversal | **None** | Native-UI introspection traversal (runbook steps 7–8); CHECK/Unique coverage (`R005`); `ddl-normalization` REVIEW exit |
| 9 | Other `active/*` (connection-list-reconnect-sideeffect, core-ui-modernization, native-core-ui-modernization, native-shadcn-ui-system, qa-p1-10-orphan-tab-close-guard, qa-p2-*, rc1-p1-workspace-recovery, rc1-p2-grid-connection-correctness, sidebar-dbeaver-codex-layout, table-details-workspace, ui-foundation-scale-hardening, ux-friendliness-audit, er-diagram-normalization, …) | unchanged | none added by this session | — | not observed | **None** | Not candidates for `COMPLETED` in this pass; several are roadmap/post-v0.1 work or carry their own pending evidence. No mass archival |

## Completion-gate check (why nothing moved)

| Gate (FEATURE_LIFECYCLE.md) | Status for the candidates above |
|---|---|
| PLAN scope explicit / CHECKLIST factual | Already true |
| P0 = 0 | True — the `v01-runtime` session found **no P0 and no P1** (`providers/23`); findings are `F1` `P2`, `F2` `P3`, `F3` already-tracked `R-KEYRING-STALL` |
| P1 = 0 | Not true at the **release** level (`R-LICENSE`, `R-GUI-SMOKE`, `R-WINLINUX` are open P1s, `risk-register.md` §2) |
| Quality gates executed | True for the candidate (recorded; not re-minted here) |
| Regression tests for invariants | True for the automated surface |
| PostgreSQL and SQLite independently accounted for | Improved: PostgreSQL now has a live run; SQLite's live **app** run is still absent |
| Unsupported provider operations capability-gated | True — verified for cancellation (`providers/15`), no bug |
| **UI-facing flow verified in native egui** | **False for every UI-facing candidate** — this is the gate that blocks every transition |
| VERIFICATION contains evidence, not adjectives | True where the automated suites are cited; the visual/ER resource claims remain adjective-level |
| STATUS.md matches the plan folder | True — `docs/plans/STATUS.md` carries the same states and this record |

## Archival

Rule: a plan directory moves to `docs/plans/completed/` only for a feature that reached
`COMPLETED`. **Archived in this pass: none.** `docs/plans/completed/` still contains only
`native-ai-agent`, `native-ide-redesign`, `native-ui-foundation`.

Re-evaluate in the same pass that executes
`docs/release/0.1.0-interactive-verification-runbook.md`: that pass is the cheapest moment to
close the UI-runtime gate honestly for the native-facing features, because it is the one session
that produces evidence level 4 for the query editor, the data grid, schema introspection, the ER
canvas and settings.
