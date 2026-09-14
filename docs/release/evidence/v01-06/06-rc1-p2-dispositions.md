# V01-06 — RC1 P2 Findings Disposition Audit

- Audited at: HEAD `3e0b0775c2248d37253d7325eddebd952e26d101`
- The 25 findings are recorded in `docs/plans/active/rc1-full-product-qa/FINDINGS.md:295-477` (`QA-P2-01` … `QA-P2-25`), with counts declared at `FINDINGS.md:13` (`| P2 | 25 |`) and `PLAN.md:32` (`**P2: 25 tracked UX/performance/consistency issues**`). **The recorded list is exactly 25, confirmed.**
- This audit **fixes nothing** and does not touch visual polish.

## 1. The structural finding, first

Every one of the 25 findings is marked `**Status:** FIXED` in `FINDINGS.md`, and every single fix targets a file under `frontend/`.

`OBSERVED`: **`frontend/` does not exist in the shipping path.** Its files were moved to `_archive/frontend/` when the React/Vite frontend was archived (`CHANGELOG.md`, `docs/plans/STATUS.md:44`), and the product UI is now native `eframe`/`egui` in `crates/ui` + `crates/native-app`.

I checked all 13 distinct target paths named across the 25 findings:

```
ARCHIVED  _archive/frontend/src/commons/components/tab-context-menu.tsx
ARCHIVED  _archive/frontend/src/commons/components/shell/topbar.tsx
ARCHIVED  _archive/frontend/src/commons/components/ide/agent-panel.tsx
ARCHIVED  _archive/frontend/src/modules/data-grid/components/data-toolbar.tsx
ARCHIVED  _archive/frontend/src/modules/unified-grid/components/unified-grid.tsx
ARCHIVED  _archive/frontend/src/commons/components/shell/sidebar-views/search-view.tsx
ARCHIVED  _archive/frontend/src/commons/components/shell/sidebar-views/explorer-view.tsx
ARCHIVED  _archive/frontend/src/modules/query/components/query-tab-content.tsx
ARCHIVED  _archive/frontend/src/modules/query/components/query-command-bar.tsx
ARCHIVED  _archive/frontend/src/modules/connection/components/connection-dialog.tsx
ARCHIVED  _archive/frontend/src/modules/connection/components/connection-editor.tsx
ARCHIVED  _archive/frontend/src/modules/er-diagram/components/er-diagram.tsx
ARCHIVED  _archive/frontend/src/commons/components/welcome-view.tsx
```

None exist at `frontend/…`. The plan folders themselves are React-era artifacts: e.g. `docs/plans/active/qa-p2-readonly-connection-grid/VERIFICATION.md` verifies via `npm test src/modules/data-grid/__tests__/data-grid.test.tsx`, and `qa-p2-favorite-rollback/VERIFICATION.md` verifies via `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`.

### Consequence for disposition

The 25 `FIXED` marks are **not false** — the fixes were really made, in the code that existed at the time. But that code is **archived and not shipped**. So for the v0.1 release:

- The finding **as written** cannot occur in the shipped product, because the code it describes is gone → `OBSOLETE` for the finding's identity.
- The **user-facing intent** behind each finding is a requirement that must hold in the *native* UI → needs a separate native-side verdict, which for almost all of them is currently **`CARRIED_OVER_UNVERIFIED`**.

That second column is what matters for release. Reporting "25/25 FIXED" without it would let a real native-UI regression hide behind a React-era fix.

Only **three** findings can be positively resolved on the native side from the repository alone, and only **one** has a fix in the shipping code path. Both are called out below.

## 2. Disposition table (all 25)

`Disposition` = state of the finding's recorded fix relative to what ships.
`Native carry-over` = whether the same user-facing defect is possible in the shipped egui UI, and whether the repo proves otherwise.

| ID | Finding (short) | React fix | Disposition | Native carry-over | Evidence |
|---|---|---|---|---|---|
| QA-P2-01 | Pinned tab visual/store order diverge | `workspace.store.ts`, `workspace-tab-bar.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — native has `crates/ui/src/workspace_view.rs`; no pinned-order invariant test found | `FINDINGS.md:297-304` |
| QA-P2-02 | Tab context-menu shortcuts hardcoded `Ctrl` on macOS | `tab-context-menu.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — `0.1.0-manual-smoke.md:225` still requires *"macOS labels use `⌘`; Windows/Linux labels use `Ctrl`"*, untested | `FINDINGS.md:306-311` |
| QA-P2-03 | Topbar reserves macOS traffic-light space on every OS | `shell/topbar.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — `0.1.0-manual-smoke.md:256-257` still requires platform-correct top chrome | `FINDINGS.md:313-318` |
| QA-P2-04 | Agent panel not visibly marked Preview/Coming Soon | `ide/agent-panel.tsx` | `OBSOLETE` | **`SATISFIED_IN_NATIVE`** | `crates/ui/src/agent_view.rs:224` renders `badge(ui, "Preview", …)`. This is the one carried intent I can positively confirm. |
| QA-P2-05 | Agent panel uses macOS-only shortcut hint | `agent-panel.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:327-332` |
| QA-P2-06 | Agent/Connection strings bypass i18n (42 strings → `t()`) | `agent-panel.tsx`, `connection-editor.tsx`, `en.json` | `DEFERRED` (scope reduction) | **`NOT_CARRIED`** — `OBSERVED`: the native UI has **no i18n framework at all** (no `i18n`, no translation tables in `crates/ui`). The native UI is hard-coded English. The React-era i18n requirement was silently dropped in the rewrite. | `FINDINGS.md:334-339`; `crates/ui/src/` has no i18n module |
| QA-P2-07 | Columns picker double-toggle on checkbox click | `data-grid/components/data-toolbar.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:341-348` |
| QA-P2-08 | Read-only connections still show editable grid affordances | `data-section.tsx`, `unified-grid.tsx`, `table_data_service.rs` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — **safety-adjacent, see §3** | `FINDINGS.md:350-355` |
| QA-P2-09 | Grid context menu can render off-screen; no desktop menu behaviour | `unified-grid/components/unified-grid.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — `0.1.0-manual-smoke.md:254` requires no broken menu/dialog layering | `FINDINGS.md:357-362` |
| QA-P2-10 | Column/shell resize handles mouse-only | `unified-grid.tsx`, `app-shell.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:364-369` |
| QA-P2-11 | Resize drag cleanup not guaranteed on unmount | `unified-grid.tsx`, `app-shell.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:371-376` |
| QA-P2-12 | Large query-result sorting synchronous on main thread | `query/components/query-tab-content.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED`, but partially mitigated — a criterion bench exists (`crates/ui/benches/result_grid_benchmarks.rs`) and previously measured 1M-row projection at 2.48 ms | `FINDINGS.md:378-383`; bench noted at `query-editor-intelligence/VERIFICATION.md:84-86` |
| QA-P2-13 | Sidebar search scans/renders everything per keystroke | `sidebar-views/search-view.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:385-390` |
| QA-P2-14 | Explorer mounts every row for large expanded schemas | `sidebar-views/explorer-view.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — native has `explorer_tree.rs`; large-schema Explorer behaviour is one of the untested V01-05 items | `FINDINGS.md:392-397` |
| QA-P2-15 | Connection test result goes stale after form edits | `connection-dialog.tsx`, `connection-editor.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:399-404` |
| QA-P2-16 | Test Connection hides backend error detail | `connection-dialog.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:406-411` |
| QA-P2-17 | SQLite Browse has no user-visible error path | `connection-editor.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — native uses the `rfd` crate for file dialogs, a different mechanism from the Tauri dialog plugin the fix targeted | `FINDINGS.md:413-418` |
| QA-P2-18 | `driverChanged` means "ever changed", not "differs from original" | `connection-editor.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:420-425` |
| QA-P2-19 | Duplicate connection silently omits credentials | `connection.queries.ts` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:427-432` |
| QA-P2-20 | Favorite optimistic update has no rollback | `connection.queries.ts` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:434-439` |
| QA-P2-21 | SQLite recent subtitle renders meaningless host/port | `welcome-view.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` | `FINDINGS.md:441-446` |
| QA-P2-22 | Export enablement tied to SQL text, not result state | `query-command-bar.tsx`, `query-tab-content.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — `0.1.0-manual-smoke.md:205` still requires *"Export availability depends on actual result state, not only editor text"* | `FINDINGS.md:448-453` |
| QA-P2-23 | Dirty replacement uses native `window.confirm` | `query-tab-content.tsx` | `OBSOLETE` | **`NOT_APPLICABLE_IN_NATIVE`** — `window.confirm` is a browser/Tauri-WebView API; there is no equivalent in egui/Rust, so the defect mechanism cannot exist | `FINDINGS.md:455-460` |
| QA-P2-24 | ER search auto-picks first substring match | `er-diagram.tsx` | `OBSOLETE` | `CARRIED_OVER_UNVERIFIED` — native has `crates/ui/src/diagram/` + `diagram_view.rs` with its own search; a `diagram_search_mode_can_leave_explicit_show_all` test exists but does not cover disambiguation | `FINDINGS.md:462-467` |
| QA-P2-25 | ER derived state does unnecessary work + LOD runtime gaps (3 sub-items: memoize `tablesInSchema`; strip handle IDs per LOD; Fit View via React Flow API) | `er-diagram.tsx`, `table-node.tsx` | `OBSOLETE` | **Split**: memoization intent = `CARRIED_OVER_UNVERIFIED`; *"Handle IDs stripped dynamically based on node LOD"* and *"Fit View calls React Flow API"* = **`NOT_APPLICABLE_IN_NATIVE`** (React Flow is part of the archived stack; native uses `crates/ui/src/diagram/`) | `FINDINGS.md:469-476` |

### Disposition counts

| Disposition | Count | IDs |
|---|---|---|
| `OBSOLETE` (code archived; finding as written cannot occur) | 24 | all except QA-P2-06 |
| `DEFERRED` (requirement dropped in the rewrite, must be an explicit decision) | 1 | QA-P2-06 |
| `FIXED` | 0 | — |
| `ACCEPTED` | 0 | — |
| `STILL_OPEN` (as literally written) | 0 | — |

### Native carry-over counts (this is the release-relevant column)

| Native verdict | Count |
|---|---|
| `CARRIED_OVER_UNVERIFIED` | 20 |
| `SATISFIED_IN_NATIVE` | 1 (QA-P2-04) |
| `NOT_APPLICABLE_IN_NATIVE` / `NOT_CARRIED` | 3 (QA-P2-23, QA-P2-25 partial, QA-P2-06) |
| positively **verified fixed in shipping code** | 0 |

**20 of 25 user-facing intents are unverified in the product that actually ships.** This is not a claim that 20 bugs exist — it is a claim that the repo contains no evidence either way. Several are almost certainly fine (e.g. P2-23's class and P2-25's React-Flow specifics genuinely do not carry).

## 3. Promotion review — is any P2 actually a crash / data loss / credential leak / destructive-SQL misclassification?

Goal-3 §24 requires promoting any such item to P0/P1. I reviewed all 25 for those four categories.

**Result: no P2 requires promotion. Zero P0/P1 promotions from this list.**

Reasoning for the two candidates that came closest:

| ID | Why it looked promotable | Why it does not promote |
|---|---|---|
| **QA-P2-08** — read-only connections still present editable Data Grid affordances | Safety-adjacent: offering edit/delete UI on a read-only connection could lead a user to believe a mutation succeeded, or (worse) to believe a connection is writable. | `FINDINGS.md:353-355` states the backend **correctly blocks mutation for read-only connections**, and the fix made `DataSection` derive editability from PK presence + readonly state. So the enforcement boundary held; this was a defence-in-depth/UX-deception defect, not a destructive-SQL misclassification. It stays P2. **But it is the highest-priority native carry-over to re-verify**, and the native side is untested. |
| **QA-P2-12** — large-result sorting on the main thread | Worst case is a UI freeze on a very large result set. | A freeze is a responsiveness defect, not a crash, data loss, credential leak, or SQL misclassification. Additionally a criterion bench exists for the native result grid. Stays P2. |

No finding in the 25 involves credentials at all. The one item that *sounds* credential-related — QA-P2-19, *"Duplicate connection silently omits credentials"* — is the opposite of a leak: it reports that credentials were **not** copied, which is the safe behaviour, and the fix only added a user-visible notice (`FINDINGS.md:427-432`).

### The adjacent data-loss item that is NOT in the 25 — and IS fixed in shipping code

`FINDINGS.md:480-493` records **`QA-D1` — "Saved query rename is delete-then-save"**, explicitly marked:

> `**Severity if exposed:** P1` — *"Current release reachability: hidden in v0.1 activity bar"*

This one genuinely has **data-loss** character (a non-atomic rename that could lose the query if the save half fails). It is the only item in the RC1 findings set with a real data-loss mechanism. I verified the fix is present **in the shipping Rust code**, not just the archived frontend:

| Element | Location | Verdict |
|---|---|---|
| Trait method | `crates/core/src/ports/saved_query_repository.rs:16` — `async fn rename(&self, id: &Uuid, name: &str) -> Result<(), DbError>` | **PRESENT** |
| Implementation | `crates/infrastructure/src/meta/saved_query_repo.rs:64` | **PRESENT** |
| Atomic SQL | `crates/infrastructure/src/meta/saved_query_repo.rs:68` — `"UPDATE saved_queries SET name = ?1 WHERE id = ?2"` (single atomic statement, no delete+insert) | **PRESENT** |
| Regression test | `crates/infrastructure/src/meta/saved_query_repo.rs:132` — `rename_rejects_missing_saved_query` | **PRESENT** |

**`QA-D1` → `FIXED` in the shipping code path, with evidence.** This is the single most significant positive disposition in this audit, because it is the only P2-severity item with a data-loss mechanism and it is the only one whose fix survives the frontend rewrite. (`docs/plans/active/rc1-full-product-qa/VERIFICATION.md:39` also records `QA-P1-10` — orphan dirty-query close guard — and the corresponding plan `docs/plans/active/qa-p1-10-orphan-tab-close-guard/PLAN.md` is still `State: IMPLEMENTING`; worth a separate look in the P1 pass, out of scope here.)

## 4. Process gap this audit exposes

`docs/plans/active/rc1-full-product-qa/CHECKLIST.md:239` — the programme's own final release gate — reads:

```
- [ ] P2 accepted/fixed/deferred explicitly
```

**`OBSERVED`: it is unchecked.** And `PLAN.md:223` requires *"all unresolved P2 are explicitly accepted/deferred with owner/reason"*. So by the RC1 programme's own rules, the P2 disposition gate is **not satisfied**, and the `Status: PASS` column added to the P1 matrix by `53e89f5` did not close it. The React-era `FIXED` marks were never re-issued against the native rewrite.

### Recommended handling (for the coordinator — not done here)

1. Keep the 25 findings as historical record with their React-era `FIXED` marks; add a **native carry-over column** rather than rewriting them.
2. Do **not** mass-close the 20 `CARRIED_OVER_UNVERIFIED` items. Verify them as part of native runtime QA — most are cheap visual/interaction checks that belong in the same pass that would re-establish V01-01/V01-05 evidence (see `04-v01-01-05-evidence-audit.md`).
3. Give **QA-P2-06 (i18n)** an explicit decision: either accept English-only for v0.1 as a documented limitation, or re-open it as a native requirement. Right now it is silently absent.
4. Re-verify **QA-P2-08** first — it is the only safety-adjacent carry-over.
5. Record `QA-D1` as `FIXED` in the shipping code path (evidence above); this closes the only data-loss-class item.
6. `ACCEPTED`/`DEFERRED` decisions must carry an owner and a reason per `PLAN.md:223`; this run deliberately assigns neither.
