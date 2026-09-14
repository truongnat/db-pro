# V01-06 — Lifecycle Transition & Active-Plan Archival Record

- Assessed at: candidate `fbf9fdab9100f08f12e29434983f32c18f14ac2f` (2026-09-14)
- Authority: `docs/plans/FEATURE_LIFECYCLE.md` (state machine, evidence levels, completion gates)
- Scope: goal-3 §22 (status transitions) and §23 (active-plan archival)
- **No transition was made in this pass.** This document records why, per feature, following the
  lifecycle's own rules. `COMPLETED` requires provider/runtime evidence; the V01 evidence audit
  found that evidence is not retrievable (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`).

## Evidence levels used (per FEATURE_LIFECYCLE.md, never mixed)

1. **Source** — code inspection only.
2. **Automated** — a command was executed and its result recorded.
3. **Provider runtime** — behaviour observed against the actual PostgreSQL or SQLite provider.
4. **UI runtime** — the user-facing lifecycle observed end-to-end in the native egui UI.

`COMPLETED` additionally requires: PLAN scope explicit, CHECKLIST factual, P0 = 0, P1 = 0,
quality gates executed, regression tests for invariants, **PostgreSQL and SQLite independently
accounted for**, unsupported operations capability-gated, the native UI flow verified
(`egui → UiCommand → runtime worker → service → DB → introspection → UiEvent → refreshed UI`),
VERIFICATION containing evidence rather than adjectives, and `STATUS.md` matching the plan folder.

## §22 — Per-candidate transition record

| # | Candidate (plan directory) | Recorded state before | Evidence present | Provider state | Runtime state | Transition made | Missing evidence / REVIEW requirement |
|---|---|---|---|---|---|---|---|
| 1 | Core Safety Hardening (`active/core-safety-hardening`) | `RUNTIME_VERIFY` | Automated: workspace test suite green (811/0/19) and the plan's own command list (prose). No captured output for the live-PG/SSH runs | PG: automated only; the `pg_integration` suite is `#[ignore]`d (**0 passed / 18 ignored**) so the plan's live-fixture claims are not re-asserted. SQLite: automated integration passes | **Not observed** — no UI/runtime artifact | **None** | Live PG + SSH run output captured durably; native UI traversal. REVIEW: no independent review artifact exists (V01 commits went straight to `main`; the Kilo reviewer is PR-only) |
| 2 | Table Data Editor Hardening (`active/table-data-editor-hardening`) | `RUNTIME_VERIFY` | Automated: staged mutation/conflict/rollback tests pass | PG: not evidenced (the plan's "SQLite + PostgreSQL runtime paths PASS" is not retrievable). SQLite: automated runtime paths PASS | **Not observed** | **None** | Live provider mutation walkthrough (3-way conflict, composite-PK reload, rollback), UI traversal |
| 3 | Query Editor Intelligence (`active/query-editor-intelligence`) | `RUNTIME_VERIFY` | Automated: `execute_multi`, completion (18 tests), diagnostics, saved-query repo tests pass | PG: the cited 18 tests are `#[ignore]`d; cancellation is capability-gated `Unsupported`. SQLite: automated | **Not observed** — the source doc's own "live provider evidence remains pending" paragraphs were never retracted | **None** | Live provider execution record; 1280×800/1440×900/1920×1080 state matrix; correct the appended PASS table (done in the docs, but the evidence itself is still absent) |
| 4 | Agent Workflow (`active/agent-workflow`) | `RUNTIME_VERIFY` | Automated: typed provider parsing, idempotency, confirmation tests pass | PG/SQLite: automated only | **Not observed** — no live-provider artifact | **None** | Live provider E2E run; native panel walkthrough. Agent remains **Preview** by product policy regardless of verification state |
| 5 | P1 Large-Schema ER Architecture (`active/er-hardening-verification`) | `RUNTIME_VERIFY` | Automated: **99** diagram tests (layout timing 20/100/500/1000, 3-tier LOD, BFS, spatial index, worker lifecycle) — the strongest automated set in the release | Provider-neutral (synthetic fixtures) | **Partially observed in source only** — "smooth" pan/zoom, idle CPU < 2% and memory stability were never measured or captured | **None** (audited `PARTIAL`) | Frame-time measurement and resource capture; the "live measured" µs/ms figures have no committed artifact |
| 6 | RC1 Full Product QA (`active/rc1-full-product-qa`) | `RUNTIME_VERIFY` | Automated: P1 regression tests exist. The appended P1 matrix `Status: PASS` column is a requirement list, not evidence | PG + SQLite: runtime smoke pending | **Not observed** — `0.1.0-manual-smoke.md` has 0 of 165 items ticked | **None** | Runtime smoke + live provider verification; the programme's own P2 gate checkbox ("P2 accepted/fixed/deferred explicitly") is **unchecked**, so the P2 gate is formally unsatisfied (dispositions recorded in `06-rc1-p2-dispositions.md` and folded into the risk register as `R-RC1-P2`) |
| 7 | Native Visual Redesign (`active/native-visual-redesign`) | `RUNTIME_VERIFY` in `STATUS.md`; the plan file's appended section claimed "Visual gates satisfied" while its own Wave 11–14 text said the traversal/review were pending | Automated: 359 `db_pro-ui` tests | PG: **no visual pass recorded**. SQLite: no captured traversal | **Not observed** — 0 of the 50 cited Orca captures survive | **None** (the earlier `PASS / VERIFIED` was downgraded in the docs; no lifecycle transition is justified) | All-surface light/dark traversal, 1920×1080 acceptance, provider review, independent review — with captures stored in-repo, not in a temp directory |
| 8 | S1–S7 schema work (`active/schema-regression`, `schema-columns-runtime`, `schema-indexes-runtime`, `schema-relations-runtime`, `schema-triggers-runtime`, `ddl-normalization`, `empty-schema-qualification`) | `RUNTIME_VERIFY` (S1–S7); `schema-regression` `REVIEW`; `ddl-normalization` `REVIEW` | Automated: SQLite introspection suites pass (columns/indexes/relations/triggers + 32 integration tests) | PG: the "18/18 PASS" rows are `#[ignore]`d tests → `EVIDENCE_GAP`. SQLite: automated PASS | **Not observed** — no native-UI schema traversal | **None** | Live PG introspection run + UI traversal; CHECK/Unique constraint coverage; `ddl-normalization` still lists frontend-era CI runs |

**Result: 0 transitions to `COMPLETED`. No feature was mass-completed, no review was fabricated,
and no `PASS` mark was moved without evidence.** The states above match `docs/plans/STATUS.md`.

## §23 — Active-plan archival record

Rule: a plan directory may move from `docs/plans/active/` to `docs/plans/completed/` **only** for a
feature that legitimately reached `COMPLETED`. Every other directory stays where it is.

**Archived in this pass: none.**

| Directory | Archived? | Why not |
|---|---|---|
| `active/core-safety-hardening` | No | pending live PG/SSH runtime evidence |
| `active/table-data-editor-hardening` | No | pending provider/UI runtime evidence |
| `active/query-editor-intelligence` | No | pending live-provider runtime evidence |
| `active/agent-workflow` | No | pending live-provider runtime evidence; Agent stays Preview |
| `active/er-hardening-verification` | No | `PARTIAL` — resource/frame-time claims unmeasured |
| `active/rc1-full-product-qa` | No | runtime smoke pending; P2 gate checkbox unchecked |
| `active/native-visual-redesign` | No | traversal + independent review pending (audited `EVIDENCE_GAP`) |
| `active/schema-*`, `active/ddl-normalization`, `active/empty-schema-qualification` | No | pending live-PG/UI evidence; `ddl-normalization` still at `REVIEW` |
| all other `active/*` (connection-list-reconnect-sideeffect, core-ui-modernization, native-core-ui-modernization, native-shadcn-ui-system, qa-p1-10-orphan-tab-close-guard, qa-p2-*, rc1-p1-workspace-recovery, rc1-p2-grid-connection-correctness, sidebar-dbeaver-codex-layout, table-details-workspace, ui-foundation-scale-hardening, ux-friendliness-audit, er-diagram-normalization, schema-*-runtime, …) | No | not candidates for `COMPLETED` in this pass; several are roadmap/post-v0.1 work or carry their own pending evidence. No mass archival. |

`docs/plans/completed/` still contains only the three pre-existing entries
(`native-ai-agent`, `native-ide-redesign`, `native-ui-foundation`); nothing was added or removed.

Re-evaluate archival in the same pass that closes the V01-01…V01-05 runtime evidence; that is the
cheapest moment to complete the native-facing features honestly.
