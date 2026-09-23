# DB Pro — Feature Status Audit (2026-09-23)

- **Audited HEAD:** `db864d6f` (branch `cursor/setup-cloud-environment-304e`)
- **Base of truth:** `docs/plans/FEATURE_LIFECYCLE.md`, `docs/plans/STATUS.md`, and each feature's
  `PLAN.md` / `CHECKLIST.md` / `FINDINGS.md` / `VERIFICATION.md` (+ `AGENT_EVIDENCE.md` where present).
- **Method:** document-based audit of every plan directory under `docs/plans/active/` (57) and
  `docs/plans/completed/` (5). Where a plan carries an explicit `State:` field it is used verbatim;
  otherwise the state is inferred from `VERIFICATION.md` / `CHECKLIST.md` / `FINDINGS.md` and marked
  `(inferred)`. This is **not** an independent runtime re-verification — it reports what the plan
  evidence currently supports.
- **Lifecycle:** `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`
  (`BLOCKED` from any non-completed state).

> Evidence legend — **PASS**: provider/runtime evidence recorded; **AUTOMATED-ONLY**: unit/CI tests
> only (no live provider or native-UI run); **NOT VERIFIED**: claimed/needed but no retrievable
> evidence; **PENDING**: native egui (`UiCommand → … → UiEvent`) runtime evidence not yet captured;
> **N/A**: not applicable to this feature.

---

## 1. Executive summary

| Metric | Value |
|---|---|
| Features tracked | 62 (57 active + 5 completed) |
| Completed | 5 |
| Active — `RUNTIME_VERIFY` | 32 |
| Active — `IMPLEMENTING` | 13 |
| Active — `REVIEW` | 9 |
| Active — `PLANNING` | 3 |
| Active — `BLOCKED` | 0 (2 have live-provider blockers noted in-row: `sqlserver-provider`, SSH suite) |
| Open **P0** (tracked) | 4 — all in `ui-core-audit` |
| Open **P1** (tracked) | ~11 (see §5) |

> **Correction (2026-09-23).** This audit derived state largely from each plan's `*.md` docs, which
> can lag the actual code. Follow-up work found several items already implemented in `main`:
> `ui-core-audit` P0s (fixed in #307), the blank query-results grid (#308), the resize hit targets
> (#309, P2-18), and `sqlserver-transaction-begin-index` (already fixed in `61e97658`, now moved to
> `completed/`). Treat per-row states below as "as of the plan docs"; verify against code before acting.

**Headline findings**

1. ~~**`ui-core-audit` is the only active feature with open P0s**~~ **[RESOLVED 2026-09-23, #307]** the
   modal focus-trap / backdrop / Esc / z-order P0s are fixed; the "build failed" note was an artifact
   of the CLI audit box, not the code. `ui-core-audit`'s remaining work is P1-5 accessibility.
2. **Systemic gap: native UI runtime evidence is `PENDING` for every UI-facing active feature.**
   Prior sessions could not reach the GUI. **This is now unblocked** — in this environment the native
   `db-pro-native` binary builds and runs, and an end-to-end SQLite flow was demonstrated
   (connect → introspect → render data grids). UI runtime evidence can now be collected.
3. **A large cluster of active plans targets the archived React frontend** (see §4). As written they
   can never earn native-UI evidence and are candidates for triage: re-scope to native egui, or
   mark `OBSOLETE`.
4. **No v0.1.0 sign-off.** RC1 runtime smoke is still 0/165; `V01-01…V01-05` are `EVIDENCE_GAP`/`PARTIAL`
   per the STATUS.md 2026-09-14 correction. See §6.
5. **New runtime defect found while running the app (this session):** the Query **results** grid
   paints blank although the query executes and returns the correct row count (the table-data grid
   renders fine). Reproduces under hardware and software GL → application bug, not environment. Not
   yet tracked in any plan; recommend filing as P1/P2 (see §5).

**Quality gates measured on this branch (`db864d6f`)**

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| `cargo test --workspace --locked` | PASS — 1322 passed / 0 failed / 41 ignored |
| `cargo build --release --locked -p db-pro-native` | PASS |

(The 41 ignored are the `#[ignore]`d live-`pg_integration` and SSH-fixture tests; ignored is never
counted as passing.)

---

## 2. Completed features (`docs/plans/completed/`)

| Feature | State | Evidence highlight |
|---|---|---|
| native-ui-foundation | COMPLETED | Native egui workspace + typed runtime facades; SQLite UI runtime + isolated PG fixture recorded (2026-09-11). |
| native-ide-redesign | COMPLETED | goal-1 P0–P7 source/runtime slice verified for PostgreSQL + SQLite. |
| native-ai-agent | COMPLETED | Typed agent orchestration slice; Agent ships **Preview** by product policy. |
| new-connection-secret-and-input | COMPLETED | P1 keyring misclassification + P2 input click-steal fixed; UI screenshots captured at 1280×800/1440×900/1920×1080. |
| sidebar-content-full-width | COMPLETED | P1 tree overflow + border loss fixed; runtime captures at all three viewports; 5 revert-verified guards. |

All five are legitimately `COMPLETED` (no `RUNTIME_VERIFY` plan is parked under `completed/`).

---

## 3. Active features — master table (`docs/plans/active/`)

### 3.1 Core / safety / providers / schema

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| core-safety-hardening | RUNTIME_VERIFY | 0 | 0 | 0 | PASS | PASS | PENDING | Broad DB safety/security hardening (SQL policy, timeouts, SSH, backups, precision); native UI runtime pending. |
| multi-statement-transaction-control | REVIEW (inferred) | 0 | 0 | 0 | PASS | PASS | PENDING | Rejects batches carrying own BEGIN/COMMIT/ROLLBACK at dispatch; live PG+SQLite pass, UI verify pending. |
| ddl-normalization | RUNTIME_VERIFY | 0 | 0 | 1 | AUTOMATED-ONLY | AUTOMATED-ONLY | PENDING | Provider-aware view/trigger DDL normalization; CI PASS, live provider + UI pending. |
| empty-schema-qualification | REVIEW (inferred) | 0 | 0 | 0 | AUTOMATED-ONLY | AUTOMATED-ONLY | PENDING | Omits empty schema prefix to avoid `""."tbl"`; fix implemented, VERIFICATION unrecorded. |
| schema-columns-runtime (S1) | RUNTIME_VERIFY | 0 | 0 | 3 | NOT VERIFIED | NOT VERIFIED | PENDING | Column DDL atomicity + cache invalidation; live PG/SQLite + UI evidence missing. |
| schema-indexes-runtime (S2) | RUNTIME_VERIFY | 0 | 0 | 2 | NOT VERIFIED | AUTOMATED-ONLY | PENDING | Index create/drop introspection; SQLite CI PASS, PG runtime + UI refresh unverified. |
| schema-relations-runtime (S3) | RUNTIME_VERIFY | 0 | 0 | 0 | NOT VERIFIED | AUTOMATED-ONLY | PENDING | Composite FK grouping across introspection/DDL/UI; SQLite CI PASS, live PG + UI pending. |
| schema-triggers-runtime (S4) | RUNTIME_VERIFY | 0 | 0 | 0 | NOT VERIFIED | PASS | PENDING | Trigger introspection/CREATE/DROP/DDL; live PG enable/disable + UI pending. |
| schema-regression (S7) | RUNTIME_VERIFY | 0 | 0 | 0 | NOT VERIFIED | PASS | PENDING | S1–S6 regression matrix; PG tests `#[ignore]`d (EVIDENCE_GAP); UI traversal unverified. |
| sqlite-view-column-introspection | RUNTIME_VERIFY | 0 | 0 | 0 | N/A | AUTOMATED-ONLY | PENDING | SQLite view column introspection parity with PG; tests pass, UI runtime pending. |
| sqlserver-provider | RUNTIME_VERIFY | 0 | 1 | 0 | N/A | N/A | PENDING | SQL Server provider adapter; automated PASS, live SQL Server fixture + UI evidence blocked. |
| sqlserver-transaction-begin-index | COMPLETED | 0 | 0 | 0 | AUTOMATED-ONLY | N/A | N/A | Report `statement_index 0` on SQL Server Begin/Validation failure. **[CORRECTED 2026-09-23]** already implemented in `61e97658` with 3 passing unit tests (audit had read the stale plan doc); moved to `completed/`. |
| table-data-editor-hardening | RUNTIME_VERIFY | 0 | 0 | 0 | AUTOMATED-ONLY | PASS | PENDING | Staged mutation identity/ChangeSet/3-way conflict; SQLite verified, live PG + UI interaction pending. |
| table-data-mutation-index-mapping | REVIEW (inferred) | 0 | 0 | 0 | AUTOMATED-ONLY | AUTOMATED-ONLY | N/A | Remap mutation-failure index to original order; core unit-tested, PR published, no runtime evidence. |
| secret-fallback-coverage | REVIEW | 0 | 0 | 0 | N/A | N/A | N/A | Deterministic Argon2/AES-GCM + encrypted-fallback tests; provider/UI runtime not applicable. |
| agent-key-secret-store | RUNTIME_VERIFY (inferred) | 0 | 0 | 0 | N/A | N/A | PENDING | Agent API-key lifecycle into runtime SecretStore; automated gates PASS, UI runtime skipped by user direction. |
| agent-prepare-run-clean-code | IMPLEMENTING | 0 | 0 | 0 | N/A | N/A | N/A | Reduce `prepare_run` args for clippy `too_many_arguments`; resolved via DTO, gates green. |
| agent-workflow | RUNTIME_VERIFY | 0 | 0 | 2 | NOT VERIFIED | NOT VERIFIED | PENDING | Typed agent orchestration (sessions/tools/confirmations/cancel); live-provider + native UI not retrievable. |
| license-public-release | REVIEW (inferred) | 0 | 0 | 1 | N/A | N/A | N/A | MIT license + package metadata + trademark boundary; third-party binary notice bundle pending. |

### 3.2 ER diagram / performance

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| er-hardening-verification | RUNTIME_VERIFY | 0 | 0 | 0 | N/A | N/A | PENDING | Verify/harden large-schema ER worker + spatial index; 99 tests PASS; pan/zoom + CPU/memory unmeasured. |
| er-diagram-normalization | RUNTIME_VERIFY | 0 | 0 | 0 | AUTOMATED-ONLY | AUTOMATED-ONLY | PENDING | Group composite FK rows into one ER edge (stable IDs); frontend-era React Flow; native/live pending. |
| explorer-tree-perf | RUNTIME_VERIFY (inferred) | 0 | 0 | 0 | N/A | N/A | PENDING | Cache/defer/cull explorer tree for 1000+ tables; automated PASS, large-schema scroll runtime pending. |
| rc1-p2-result-grid-sort-perf | IMPLEMENTING (inferred) | 0 | 0 | 0 | N/A | N/A | PENDING | Zero-allocation result-grid sort/filter; VERIFICATION is a plan, no measured perf evidence. |
| ui-foundation-scale-hardening | RUNTIME_VERIFY | 0 | 0 | 2 | N/A | N/A | N/A | Frontend-era token contract + large-schema ER scaling; native egui verification n/a (archived React). |

### 3.3 Query editor / workspace

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| query-editor-intelligence | RUNTIME_VERIFY | 0 | 0 | 2 | NOT VERIFIED | AUTOMATED-ONLY | PENDING | Per-document query/completion/prediction determinism; live PG + native viewport evidence missing. |
| query-editor-hover-intelligence | RUNTIME_VERIFY | 0 | 0 | 0 | AUTOMATED-ONLY | AUTOMATED-ONLY | PENDING | Rich hover cards (tables/columns/keywords/functions); automated PASS, native visual verification pending. |
| query-editor-zed-feel | IMPLEMENTING | 0 | 0 | 2 | N/A | N/A | PENDING | Smooth scroll/caret + version-safe completion; implementation uncommitted, owner runtime verify pending. |
| query-workspace-zed-shell | IMPLEMENTING | 0 | 0 | 0 | N/A | N/A | PENDING | Editor-first Query shell (dock/find/status); most runtime captures pending. |
| ui-query-workspace | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Query header/search/snippet buttons → canonical components; gates pass, native screenshots pending. |
| saved-query-rename | IMPLEMENTING | 0 | 1 | 0 | N/A | NOT VERIFIED | PENDING | Atomic saved-query rename to stop delete-then-save data loss; targets archived frontend, checklist unchecked. |

### 3.4 Native UI system / redesign / component language

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| native-core-architecture | IMPLEMENTING | 0 | 2 | 0 | N/A | N/A | PENDING | Migrate native UI to feature-owned state + one-way transitions; gates green, 1920×1080 capture unstable. |
| native-core-ui-modernization | IMPLEMENTING | 0 | 0 | 8 | N/A | N/A | PENDING | Native egui primitives (tooltip/spinner/switch/toast); implementation not started. |
| native-shadcn-ui-system | RUNTIME_VERIFY (inferred) | 0 | 0 | 0 | N/A | N/A | PENDING | Shadcn-style native component system + gallery; automated gates pass, screenshots not retrievable. |
| native-visual-redesign | RUNTIME_VERIFY | 0 | 0 | 0 | N/A | N/A | PENDING | Dark-first native workbench redesign; runtime PASS downgraded to EVIDENCE_GAP, no retrievable captures. |
| core-ui-modernization | IMPLEMENTING | 0 | 0 | 5 | N/A | N/A | N/A | Polish React shadcn primitives; frontend-era (archived UI), no implementation done. |
| ui-core-audit | PLANNING (inferred) | **4** | **5** | 10 | N/A | N/A | PENDING | Native egui core-component audit; **P0 modal focus/backdrop/Esc/z-order unfixed**; verification build failed. |
| ui-component-language | RUNTIME_VERIFY (inferred) | 0 | 0 | 2 | N/A | N/A | PENDING | Canonical component migration on inspector/explorer/tasks/query; screenshot capture blocked; #288 inventory open. |
| ui-shell-hierarchy | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Shell/top bar/activity bar/sidebar hierarchy tokenized; gates pass, native screenshots pending. |
| ui-dialogs-forms-consistency | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Dialogs/forms/destructive confirmations → canonical buttons; gates pass, screenshots pending. |
| ui-data-grid-hardening | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Grid/table toolbar + pagination buttons → canonical components; gates pass, screenshots pending. |
| ui-explorer-polish | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Explorer folder colors tokenized + context menus standardized; gates pass, screenshots pending. |
| ui-schema-workbench-polish | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Schema/object/DDL workbench buttons → canonical components; gates pass, screenshots pending. |
| ui-files-ide-workspace-polish | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Files/IDE workspace action buttons → canonical components; gates pass, screenshots pending. |
| ui-agent-workspace-polish | RUNTIME_VERIFY (inferred) | n/a | n/a | n/a | N/A | N/A | PENDING | Agent workspace buttons → canonical components; gates pass, native screenshot evidence pending. |
| table-details-workspace | RUNTIME_VERIFY (inferred) | 0 | 1 | n/a | NOT VERIFIED | AUTOMATED-ONLY | PENDING | Rich table workspace (grid/edit/structure/DDL/query); live PG + multi-resolution runtime pending. |

### 3.5 Sidebar / explorer / connection

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| sidebar-header-and-delete-session | IMPLEMENTING | 0 | 0 | 0 | N/A | N/A | PENDING | Merge header launcher + preserve active session on sibling delete; unit 4/4, UI runtime pending. |
| sidebar-dbeaver-codex-layout | IMPLEMENTING | n/a | n/a | n/a | N/A | N/A | PENDING | Three-pane resizable DBeaver/Codex Explorer sidebar; only PLAN exists, no tests/verification. |
| connection-list-reconnect-sideeffect | PLANNING (inferred) | 0 | 1 | 0 | N/A | N/A | N/A | Decouple session restoration from connection-list refetch; frontend-era plan (archived UI). |

### 3.6 RC1 QA / QA fix waves (many target the archived React frontend)

| Feature | State | P0 | P1 | P2 | PG | SQLite | UI runtime | Summary |
|---|---|---|---|---|---|---|---|---|
| rc1-full-product-qa | RUNTIME_VERIFY | 0 | 0 | 25 | AUTOMATED-ONLY | AUTOMATED-ONLY | PENDING | Pre-release product QA; all P1 fixed but runtime smoke + live-provider verification pending. |
| rc1-p1-workspace-recovery | REVIEW (inferred) | 0 | 0 | 0 | N/A | N/A | N/A | Orphan close guard + provider-aware tab reassignment; archived React frontend, automated tests only. |
| rc1-p2-grid-connection-correctness | REVIEW (inferred) | 0 | 0 | 0 | N/A | N/A | N/A | Fix columns double-toggle, stale test badge, hidden error detail; archived React frontend. |
| qa-p1-10-orphan-tab-close-guard | IMPLEMENTING | 0 | 0 | 0 | N/A | N/A | N/A | Route orphan-tab close through shared dirty guard; archived React frontend, no native evidence. |
| qa-p2-favorite-rollback | IMPLEMENTING | 0 | 0 | 0 | N/A | N/A | N/A | Roll back optimistic favorite toggle on error; archived React frontend. |
| qa-p2-query-export-and-sqlite-subtitle | REVIEW (inferred) | 0 | 0 | 0 | N/A | N/A | N/A | Fix export enablement + SQLite recent-connection subtitle; archived React frontend. |
| qa-p2-readonly-connection-grid | REVIEW (inferred) | 0 | 0 | 0 | N/A | N/A | N/A | Read-only connection grid affordance + mutation guard; archived React frontend, automated tests only. |
| issue-300-ui14 | IMPLEMENTING | n/a | n/a | n/a | N/A | N/A | PENDING | Responsive density/overflow hardening across native egui; slices 1–3 done, runtime screenshots blocked. |
| ux-friendliness-audit | RUNTIME_VERIFY | n/a | n/a | n/a | PASS | PASS | PENDING | Frontend-era UX audit + wave fixes; keyboard-only + D7 stress runtime matrix pending. |

---

## 4. Archived-frontend triage list

The following active plans were written against the **React/Vite/Tauri-webview frontend archived on
2026-09-11** (`_archive/frontend/`). As worded they cannot earn native-UI runtime evidence and are
stale. Recommend an explicit disposition per item — re-scope to native egui, or mark `OBSOLETE` and
move out of `active/`:

`connection-list-reconnect-sideeffect`, `core-ui-modernization`, `saved-query-rename`,
`qa-p1-10-orphan-tab-close-guard`, `qa-p2-favorite-rollback`, `qa-p2-query-export-and-sqlite-subtitle`,
`qa-p2-readonly-connection-grid`, `rc1-p1-workspace-recovery`, `rc1-p2-grid-connection-correctness`,
`ui-foundation-scale-hardening`, `er-diagram-normalization` (React Flow), `ux-friendliness-audit`
(frontend-era waves).

> Note: STATUS.md already flags several of these as "frontend-era / historical." This audit lists
> them together so the cleanup is actionable in one pass.

---

## 5. Open P0 / P1 register (from plan `FINDINGS.md`)

| Sev | Feature | Item |
|---|---|---|
| P0 ×4 | ui-core-audit | Native modal focus trap / backdrop / Esc-to-close / z-order defects; plan notes verification build failed. |
| P1 ×5 | ui-core-audit | Companion correctness/interaction defects in the core-component audit. |
| P1 ×2 | native-core-architecture | Feature-state migration correctness items; 1920×1080 capture unstable. |
| P1 | saved-query-rename | Delete-then-save rename data-loss class (targets archived frontend — verify against native path). |
| P1 | sqlserver-provider | Live SQL Server fixture + UI evidence blocked (no server provisioned). |
| ~~P1~~ RESOLVED | sqlserver-transaction-begin-index | `statement_index 0` on Begin/Validation failure — **[CORRECTED 2026-09-23]** already implemented (`61e97658`) and regression-tested; not an open finding. |
| P1 | table-details-workspace | Live PG + multi-resolution runtime evidence outstanding. |

**Newly observed this session (not yet in any plan — recommend filing):**

| Sev (proposed) | Area | Item |
|---|---|---|
| P1/P2 | Query results grid | Results pane paints blank while query executes and returns correct row count; table-data grid renders correctly. Reproduces under hardware and software GL → application defect. Consistent with README noting interactive runtime smoke was never completed. |

---

## 6. Release readiness (v0.1.0)

Per `docs/plans/STATUS.md` (2026-09-14 correction) and `README.md`:

- **Not release-signed-off.** RC1 manual smoke = **0 passed / 165 blocked**.
- `V01-01` Native Visual Redesign, `V01-02` Query Editor, `V01-04` Schema Introspection,
  `V01-05` Integrated RC1 Smoke → **`EVIDENCE_GAP`**; `V01-03` Large-Schema ER → **`PARTIAL`**.
- Live PostgreSQL 16.15 fixture exists: the 18 `pg_integration` tests pass **18/18**
  (`docs/release/evidence/v01-runtime/providers/07`) — but that is **provider-level automated**
  evidence, not the **native-UI runtime** evidence completion requires.
- SSH suite `BLOCKED` (nine `DB_PRO_SSH_*` unset); Agent ships **Preview** by policy.
- Release rule: no `v0.1.0` tag while any confirmed P1 remains open (`ui-core-audit` currently
  violates this) and until the RC1 runtime smoke + live-provider checks are recorded.

---

## 7. Recommendations (priority order)

1. **Fix `ui-core-audit` P0s** (modal focus/backdrop/Esc/z-order) and unblock its build — it is the
   only active feature with open P0s and blocks any release rule.
2. **Capture the now-unblocked native-UI runtime evidence.** The GUI runs in this environment;
   drive the `egui → UiCommand → … → UiEvent` matrix for the `RUNTIME_VERIFY` schema/table/query
   features and attach captures at 1280×800 / 1440×900 / 1920×1080.
3. **File and fix the blank Query results grid** (§5) — it directly undermines the core SQL flow.
4. **Triage the archived-frontend cluster (§4)** — re-scope to native egui or mark `OBSOLETE` to
   stop it inflating the active backlog.
5. **Close provider gaps:** run the schema S1–S4 features' live PostgreSQL introspection (not just
   CI-ignored tests) and record the `sqlserver-provider` live-fixture path or keep it `BLOCKED`.
6. **RC1 smoke:** work down `0.1.0-manual-smoke.md` (0/165) now that the app is runnable.

---

## Tổng kết bằng tiếng Việt

- Đã kiểm toán **62 feature** (57 `active` + 5 `completed`) trên HEAD `db864d6f` dựa trên tài liệu kế
  hoạch; đây là audit theo tài liệu, không phải kiểm chứng runtime độc lập.
- Phân bố active: **32 `RUNTIME_VERIFY`, 13 `IMPLEMENTING`, 9 `REVIEW`, 3 `PLANNING`**. 5 feature đã
  `COMPLETED` nằm đúng thư mục.
- **Vấn đề lớn nhất:** `ui-core-audit` còn **4 P0 + 5 P1** (lỗi modal focus/backdrop/Esc/z-order,
  build verify thất bại) — ưu tiên số 1.
- **Khoảng trống hệ thống:** mọi feature UI đang `RUNTIME_VERIFY` đều **thiếu bằng chứng runtime UI
  egui**. Trước đây GUI không chạy được; **giờ đã chạy được** trong môi trường này nên có thể thu
  thập bằng chứng.
- Phát hiện mới khi chạy app: **lưới kết quả Query bị trắng** dù truy vấn chạy đúng và trả đúng số
  dòng (lưới table-data vẫn hiển thị tốt) → lỗi ứng dụng, nên mở issue.
- Một cụm lớn feature vẫn nhắm tới **frontend React đã archive** — cần quyết định re-scope sang egui
  hoặc đánh dấu `OBSOLETE`.
- **Chưa thể phát hành v0.1.0**: smoke thủ công 0/165, V01-01…05 còn `EVIDENCE_GAP`/`PARTIAL`.
