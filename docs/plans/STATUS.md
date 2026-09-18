# Schema Runtime Verification — Status

Canonical lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`.

> **Status correction — 2026-09-14 (V01-06 evidence audit).** The V01-01…V01-05 closure
> claims recorded on 2026-09-14 by `53e89f5` are **not** backed by retrievable evidence and
> are downgraded here to what the evidence supports:
>
> | Gate | Recorded 2026-09-14 | Audited verdict | Missing evidence |
> |---|---|---|---|
> | V01-01 Native Visual Redesign | PASS | **`EVIDENCE_GAP`** | 0 of 50 cited captures survive; no 1920×1080 evidence; no PostgreSQL visual pass; no independent review |
> | V01-02 Query Editor | PASS | **`EVIDENCE_GAP`** | no live-provider record; 18 PG tests are `#[ignore]`d; cancellation over-stated |
> | V01-03 Large-Schema ER | PASS | **`PARTIAL`** | automated tests real; "smooth pan/zoom" and idle CPU/memory never measured |
> | V01-04 Schema Introspection | PASS | **`EVIDENCE_GAP`** | live-PG half on ignored tests; no native-UI traversal; CHECK/Unique uncovered |
> | V01-05 Integrated RC1 Smoke | PASS | **`EVIDENCE_GAP`** | 0 of 165 checklist items ticked; sign-off contradicts itself; three different SHAs |
>
> Authority: `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`. The recorded PASS
> marks are retained as history, not silently deleted.
>
> **Current measured workspace state on HEAD (2026-09-14):** `cargo test --workspace` →
> **815 passed / 0 failed / 19 ignored** at the candidate `85a7fa3` (811/0/19 before the `543b526`
> state-dir fix). The 19 ignored tests are 18 `#[ignore]`d
> `pg_integration` cases (require `DATABASE_URL`) and 1 `#[ignore]`d SSH backup test (requires
> the `DB_PRO_SSH_*` fixtures). Ignored tests are never counted as passing. Stale totals in the
> rows below (571 / 567 / 808 / 809 / 827) are corrected in place.
>
> **Runtime evidence update — 2026-09-14 (`v01-runtime` session, no code changed).** A live
> PostgreSQL 16.15 fixture is now a retrievable artifact: the 18 `pg_integration` tests pass
> **18/18** against it (`docs/release/evidence/v01-runtime/providers/07`), and with `DATABASE_URL`
> supplied the workspace suite is 815/0/19, i.e. an effective **833 passed / 0 failed / 1 ignored**.
> A deterministic SQLite runtime fixture, cancellation capability-gating verification, packaged
> state-directory branch checks, the keyring-stall classification (`P2`) and an error-log audit are
> in the same tree. **This changes no state below:** `COMPLETED` still requires the native-UI
> runtime evidence (`egui → UiCommand → … → UiEvent`) that this session could not produce — the
> GUI is unreachable here (`providers/21`) and no screenshot exists. The SSH suite stayed
> `BLOCKED` (nine `DB_PRO_SSH_*` unset). Transitions: **none**; per-item reasoning in
> `docs/release/evidence/v01-runtime/24-lifecycle-transitions.md`. Owner decisions still open:
> `docs/release/0.1.0-human-decisions.md`.

| Feature | Branch | PR | State | Notes |
|---|---|---|---|---|
| Core Safety Hardening | main | — | RUNTIME_VERIFY | Explain/export/mutation safety including lossless large-BIGINT Excel export, validated table counts, checked cross-connection diffs, deterministic schema diffs, provider-specific connection credentials, capability-gated user management, provider-aware reconstructed table DDL with CHECK preservation and schema-correct indexes, consistent connection validation across create/update/test, lexical SQL statement boundaries, token-aware destructive SQL classification, atomic multi-statement mutation routing, SQLite and PostgreSQL operation deadlines with transaction rollback, SSH host-key/tunnel safety and matching password-auth test path, SSH credentials separated from metadata, provider-correct backup credential handling, connection-scoped schema-cache invalidation, SSH-aware backup with atomic no-overwrite publication, safe PostgreSQL user-management SQL, compensating connection lifecycle/secret cleanup, duplicate-connect cleanup retry, persisted backup secret references, and checked pagination; PostgreSQL fixture live evidence PASS, SQLite integration PASS, ~~571 workspace tests passing~~ **[CORRECTED 2026-09-14]** measured today 811 passed / 0 failed / 19 ignored. The live-PostgreSQL fixture claim is not part of the retrievable V01 evidence set and is not re-asserted here. **[UPDATED 2026-09-14, `v01-runtime`]** A retrievable live-PostgreSQL run now exists — 18/18 PASS on server 16.15 (`docs/release/evidence/v01-runtime/providers/07`), covering the transaction-rollback, batch-failure/timeout and typed-value paths (SSH half remains `BLOCKED`: `DB_PRO_SSH_*` unset). State stays `RUNTIME_VERIFY` — no UI runtime evidence exists. |
| Table Data Editor Hardening | main | — | RUNTIME_VERIFY | Stable RowIdentity/ChangeSet mutations, typed filters, multi-sort, scoped layout migration, pending-change review, 3-way conflict resolution (Original/Local/DB Current), Keep Mine / Use Database actions, composite PK targeted reload, insert-delete safety, atomic rollback preservation, and precomputed coordinate maps; SQLite automated runtime paths PASS; ~~SQLite + PostgreSQL runtime paths PASS, 571 workspace tests passing~~ **[CORRECTED 2026-09-14]** the PostgreSQL half is not evidenced by the V01 evidence set, and the workspace total measured today is 811 passed / 0 failed / 19 ignored. State stays `RUNTIME_VERIFY`. **[UPDATED 2026-09-14, `v01-runtime`]** Live PostgreSQL **automated** coverage now exists for the mutation/rollback/batch-failure paths (18/18 PASS, `providers/07`); the staged-edit/3-way-conflict **interactive** walkthrough is still absent, so no transition. |
| Query Editor Intelligence Layer | main | — | RUNTIME_VERIFY | Automated lifecycle/multi-result hardening passes; completion, delimiter UX, structured diagnostics, snapshot dirty state, saved-query updates, local draft/history, explicit result kinds/errors, and ordered per-document output implemented. **[CORRECTED 2026-09-14]** live provider and native viewport evidence remain **missing, not merely pending** — the appended V01-02 PASS table is not evidence-backed (audit §3), and the 18 PG integration tests are `#[ignore]`d (0 passed / 18 ignored). No transition. **[UPDATED 2026-09-14, `v01-runtime`]** The 18 tests now **run live and pass 18/18** (`providers/07`), but that is provider-level automated evidence, not the native-viewport record this plan needs; no transition. |
| Agent Workflow | main | — | RUNTIME_VERIFY | Commits 435ccfa..fa50b4b; typed provider tool orchestration, document-scoped sessions, UTF-8 patch version safety, mutation/destructive confirmation, call_id idempotency and failure replay, database query cancellation wired to Stop/close-tab, PostgreSQL and SQLite runtime paths PASS, live provider E2E PASS, native UI compact panel PASS, Vietnamese IME native input/undo PASS, and multi-tab isolation PASS. **[CORRECTED 2026-09-14]** the live-provider/E2E and native-panel PASS claims rest on automated tests and prose only; no live-provider artifact is retrievable, and ~~567 workspace tests~~ today measures 811 passed / 0 failed / 19 ignored (the "246 UI" figure is stale; `db_pro_ui` = 359). Agent remains **Preview** by product policy regardless of verification state. **[UPDATED 2026-09-14, `v01-runtime`]** Still no live-provider run: no provider key was available (`GROQ_API_KEY` unset), so the live E2E claim remains unverified; Agent stays Preview. |
| S1 Columns | main | 741a18d | RUNTIME_VERIFY | implementation complete; PG introspection via CI (automated); UI evidence pending |
| S2 Indexes | main | aa77ece | RUNTIME_VERIFY | PR merged; PG introspection via CI (automated); UI evidence pending |
| S3 Relations | main | #7 (7facb95) | RUNTIME_VERIFY | merged; composite FK identity + DDL + UI grouping; CI integrated PASS; PG live + UI pending |
| S4 Triggers | main | #8 (7facb95) | RUNTIME_VERIFY | merged; introspection + lifecycle + DDL via CI; enable/disable not yet exercised in live PG |
| S5 DDL | main | #8 (7facb95) | RUNTIME_VERIFY | merged; view DDL + dialect quoting + trigger DDL ops; CI integrated PASS |
| S6 ER Diagram | main | #9 (89f11a9) | RUNTIME_VERIFY | merged; schema-level workspace tab; explicit schema prop; composite FK edge grouping; ~~position persistence~~ **[CORRECTED 2026-09-14]** position persistence does **not** exist in the shipping UI (`crates/ui/src/diagram/` rebuilds a grid layout each load; no `dbpro.native.diagram*` keys); the "workspace migration v2→v3" recorded here is a frontend-era (archived) workspace-store migration, not a meta-store migration — the meta store is at `schema_version = 2` with no v3 |
| S7 Full Schema Regression | main | #9 (89f11a9) | RUNTIME_VERIFY | merged; regression matrix complete; 39 Rust + ~~1324 FE~~ **[CORRECTED 2026-09-14]** frontend suite archived 2026-09-11 — the FE count is historical and gates nothing; the PG "18/18 PASS" rows in `schema-regression/VERIFICATION.md` are `EVIDENCE_GAP` (the tests are `#[ignore]`d). **[UPDATED 2026-09-14, `v01-runtime`]** The same `pg_integration` suite now has a retrievable live run — **18/18 PASS** on PostgreSQL 16.15 (`providers/07`), filed under `v01-runtime`; the original record's own artifact is still absent, and no native-UI schema traversal exists, so the state does not change. |
| IT0-101 BIGINT Precision & Staged State | main | #11 (e5c4c9b) | COMPLETED | merged; i64 lossless IPC contract, staged-changes close guard, preview promotion, SQLite metadata; 1483 FE + 20 Rust tests PASS *(historical: FE suite archived 2026-09-11)* |
| New Connection Secret & Input Fix | fix/new-connection-secret-and-input | — | COMPLETED | P1 keyring empty-service misclassification (new-connection "Configuration Error") + P2 input-frame click-steal (password eye no-show, double-click no-select) fixed and revert-verified; 514 workspace tests pass, clippy/build/clean-code green; **UI runtime screenshots CAPTURED** at 1280×800 / 1440×900 / 1920×1080 via the `capture` feature — New Connection dialog renders with password input + eye toggle, no Configuration Error, on the disabled-keyring path. Plan: docs/plans/completed/new-connection-secret-and-input/ |
| Sidebar Content Full Width | fix/sidebar-column-overflow | — | RUNTIME_VERIFY→COMPLETED | P1 tree-row overflow (driver badge hidden) + P1 1px border loss on flush widgets fixed; runtime evidence captured at 1280×800 / 1440×900 / 1920×1080 via the `capture` feature (PNGs in `screenshots/`) — badges fully readable, filter + `New query` borders closed on all four sides, tree fills the sidebar column; five revert-verified guards (clip / badge width / field border / row width / scrollbar-narrow). Plan: docs/plans/completed/sidebar-content-full-width/ |
| Sidebar Header + Delete Session | fix/sidebar-header-and-delete-session | — | IMPLEMENTING | P2 header name vs search affordance split (Connections-scoped switcher vs Command Palette); P1 delete-non-active no longer tears down / reconnects the live session. Unit 4/4 PASS; UI runtime evidence pending. Plan: docs/plans/active/sidebar-header-and-delete-session/ |
| Query Editor Zed Feel | feature/query-editor-zed-feel | — | IMPLEMENTING | Smooth scroll (was unused content size), blinking caret, softer gutter/line/selection, keep-caret-in-view. Renderer unit PASS; UI runtime pending. Plan: docs/plans/active/query-editor-zed-feel/ |

> **Note (2026-09-11):** rows below that reference frontend/FE test counts, shadcn design
> tokens, React Flow, or `frontend/` paths describe work performed against the now-archived
> React frontend. They are retained as history. The product UI is native `eframe`/`egui`
> (`crates/ui` + `crates/native-app`); see `_archive/README.md`.

## P3 — UI Foundation & Scale Hardening

Pre-release hardening program. Blocks v0.1.

| Sub-program | State | Notes |
|---|---|---|
| P3.1 Design Token Contract | REVIEW | shadcn tokens alias --app-* canonical layer; drift check passes *(frontend-era)* |
| P3.2 shadcn Integration Safety | REVIEW | `npm run check:tokens` detects drift; contract in globals.css *(frontend-era)* |
| P3.3 ER Diagram Algorithm | REVIEW | Pre-indexed maps; benchmark 500t=16ms, 1000t=26ms *(frontend-era benchmark)* |
| P3.4 ER Diagram Duplicate Layout | REVIEW | Single layoutGraph() in useMemo; edge highlight separated *(frontend-era)* |
| P3.5 ER Diagram Rendering LOD | REVIEW | 3-tier zoom LOD; MiniMap disabled >200 nodes *(frontend-era)* |
| P3.6 ER Diagram Large Schema Mode | REVIEW | Neighborhood BFS for 200+ tables; search-first default needs RC1 QA correction before release |
| P3.7 Performance Budgets | REVIEW | Fixtures at 20/100/500/1000; automated regression test |
| P3.8 Data Grid / Metadata List Audit | REVIEW | Explorer O(S×T) fixed; deeper RC1 QA found additional state/performance issues |
| **P1 Large-Schema ER Architecture** | **RUNTIME_VERIFY** | full architecture implemented (ErGraph model + precomputed edge bbox + ErSpatialIndex uniform grid + ErViewport coordinate engine + 3-tier ErLod + BFS neighborhood exploration); async layout worker with coalescing; bounded spatial index (max_span=32); worker degraded mode on spawn failure; version overflow fixed (saturating_add); 5 P1/P2 findings fixed; ~~98 diagram tests + 808 workspace tests PASS~~ **[CORRECTED 2026-09-14]** measured today: **99** diagram tests passed, workspace 811 passed / 0 failed / 19 ignored; native runtime evidence pending (audited `PARTIAL` — smooth pan/zoom and idle CPU/memory have no artifact). State stays `RUNTIME_VERIFY`. |

| Native UI Foundation | COMPLETED | native egui workspace, shared runtime facades and Tauri command boundary implemented; SQLite UI runtime evidence and isolated PostgreSQL fixture coverage pass *(recorded 2026-09-11; not re-audited in this run)* |
| Native IDE Redesign | COMPLETED | goal-1.md P0–P7 source/runtime slice verified for PostgreSQL and SQLite; future provider surfaces remain explicitly out of scope *(recorded; not re-audited in this run)* |
| Native Visual Redesign | RUNTIME_VERIFY | goal-2.md Waves 1–14 native shell, full-window startup, shared data-grid, context-aware status, full ER canvas, live cursor status, keyboard grid focus, ER search recovery, staged-grid interaction correctness, Codex-aligned native token/component calibration, metadata empty-state composition, query output empty-state composition, statement-complete output state and compact Query overflow actions. **[CORRECTED 2026-09-14]** Transitioned from the previously recorded state: the closure plan's `PASS / VERIFIED` is downgraded to `RUNTIME_VERIFY` because the all-surface light/dark traversal, 1920×1080 acceptance, provider/runtime review and independent review have **no retrievable evidence** (audit §2, `EVIDENCE_GAP`). No `COMPLETED` transition is justified. |
| React Frontend Archival | COMPLETED | React/Vite/Tauri-webview frontend moved to `_archive/frontend/` on 2026-09-11, together with the React-era ER benchmark harness (`_archive/bench/`); CI, release pipeline, AGENTS.md and docs switched to native UI; `crates/tauri-app` kept as legacy transitional host and marked for removal at cutover |

## RC1 Full Product QA

Audit baseline: `main@6e0a04ad675eaa85cae08e1a066270596a18db`  
Audit branch: `qa/rc1-static-audit`

| Program | State | P0 | P1 | P2 | Notes |
|---|---|---:|---:|---:|---|
| RC1 Full Product QA — Static Audit & Remediation | RUNTIME_VERIFY | 0 | 0 | 25 | All 14 P1 findings (W1..W5) resolved & verified with regression test coverage; ~~workspace quality gates green (571 tests PASS); runtime smoke and live provider verification pending~~ **[CORRECTED 2026-09-14]** measured today: workspace 811 passed / 0 failed / 19 ignored; the P1 matrix `Status: PASS` column is a requirement list, not evidence; runtime smoke and live provider verification remain **pending**. The programme's own P2 gate checkbox ("P2 accepted/fixed/deferred explicitly") is **unchecked**, so the P2 gate is formally unsatisfied. P2 dispositions for v0.1: 24 `OBSOLETE` (targets archived with `frontend/`), 1 `DEFERRED` (i18n), 20 native carry-over items `CARRIED_OVER_UNVERIFIED`; no P2 promoted to P0/P1; `QA-D1` (saved-query rename data-loss class) is `FIXED` in the shipping code path — `docs/release/evidence/v01-06/06-rc1-p2-dispositions.md`. State stays `RUNTIME_VERIFY`. **[UPDATED 2026-09-14, `v01-runtime`]** The live PostgreSQL `pg_integration` run (18/18 PASS, `providers/07`) is provider-level automated evidence only: the RC1 runtime smoke and live-provider Agent verification remain pending, and `0.1.0-manual-smoke.md` is still 0 passed / 165 blocked. No transition. |

Release rule for this QA program:

- no `v0.1.0` tag while confirmed P1 findings remain open;
- source evidence does not close provider/runtime checks;
- BIGINT precision, staged mutation identity, connection lifecycle, orphan recovery, large-schema ER and provider type mapping are highest-priority P1 waves;
- no new product/Agent/MCP feature work during RC remediation.

## Rules

- A plan with pending runtime/provider evidence stays under `docs/plans/active/`.
- `COMPLETED` requires P0=0, P1=0 and all applicable provider/UI runtime evidence.
- PostgreSQL and SQLite are verified independently; one provider never proves the other.
- An `#[ignore]`d test is never evidence of passing. A count that includes ignored tests must
  say so.
- **Archival check (2026-09-14):** no feature in this file reached `COMPLETED` in this run, so
  nothing was moved to `docs/plans/completed/`. The per-feature transition and archival reasoning
  (evidence, provider, runtime state, missing evidence, REVIEW requirement) is recorded in
  `docs/release/evidence/v01-06/10-lifecycle-and-archival-record.md` and, for the `v01-runtime`
  pass, in `docs/release/evidence/v01-runtime/24-lifecycle-transitions.md`; the RC1 P2 dispositions
  are in `06-rc1-p2-dispositions.md` and are folded into `docs/release/risk-register.md`
  (`R-RC1-P2`). **The `v01-runtime` pass made zero transitions** — see that record for the
  item-by-item missing evidence.
