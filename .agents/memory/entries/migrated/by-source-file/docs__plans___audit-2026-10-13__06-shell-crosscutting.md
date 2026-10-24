---
schema_version: 1
id: "MEM-SRC-1245609935be9f2e"
title: "Source mirror: docs/plans/_audit-2026-10-13/06-shell-crosscutting.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/_audit-2026-10-13/06-shell-crosscutting.md"
source_sha256: "1245609935be9f2eab6c659c946d94e9c099503b51a1050077b0ef8ef2bb94d2"
mirror_body_sha256: "1245609935be9f2eab6c659c946d94e9c099503b51a1050077b0ef8ef2bb94d2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Audit — Shell & Cross-Cutting Plans (06)

Date: 2026-10-13. HEAD: `d73148cd`. Auditor: ShellCrosscutAudit. Method: plan-doc claims vs
code/tests on `main`, `scripts/check-ui-architecture.sh` executed, two native capture attempts
(both failed — host windowing produced no framebuffer, consistent with the recorded
window-server starvation in `restore-egui-ui`/`ui-stitch-parity-pass` histories).

## 1. Slice scope

Plans: ui-shell-hierarchy, ui-dialogs-forms-consistency, ui-explorer-polish,
ui-schema-workbench-polish, ui-files-ide-workspace-polish, ui-agent-workspace-polish,
ui-spacing-consistency, ui-stitch-parity-pass, settings-shell-visual-polish,
macos-app-menu-refresh, native-app-branding, connection-dialog-accessibility,
native-core-architecture, native-visual-redesign, egui-0-36-upgrade, restore-egui-ui,
rs-ui-runtime-integration, ux-friendliness-audit, rc1-full-product-qa, agent-workflow,
agent-key-secret-store, license-public-release.

Code areas: `crates/ui` shell (topbar/activity bar/sidebar/statusbar/dialogs),
`crates/native-app` (menu, branding, capture), `crates/runtime` (worker, agent, SecretStore),
`crates/core/src/domain/agent*`.

## 2. Per-plan verdicts

| Plan | Claimed state | Supported state | Verdict | Key evidence |
|---|---|---|---|---|
| ui-shell-hierarchy | All checkboxes done; tokens-only shell | Sidebar/topbar/activity bar refactored into `sidebar_surface_view.rs`, `shell_*_view.rs`; no hardcoded colors found in shell files | SUPPORTS (impl) / EVIDENCE_GAP (runtime) | `grep Color32` clean on `shell_*.rs`, `activity_bar_view.rs`; VERIFICATION test count (483) is stale vs current suite size |
| ui-dialogs-forms-consistency | Canonical `Button` everywhere; guard test extended | 0 raw `ui.button(`/raw Button in `query_dialogs_view.rs`, `connection/view.rs`, `table_metadata_view.rs`; guard test covers all listed files | SUPPORTS | `crates/ui/src/components/mod.rs:128-153` guard includes `query_dialogs_view.rs`, `connection/view.rs` |
| ui-explorer-polish | Folder colors behind theme tokens; canonical context menus | `explorer_schema_object_folders_view.rs:65,115,164` uses `theme.success`/tokens; `ctx_menu_item` used across 10 explorer/menu files | SUPPORTS | `grep ctx_menu_item` → `explorer_*_view.rs` et al. |
| ui-schema-workbench-polish | Canonical buttons in schema/table views | 0 raw buttons in `schema_workbench_form.rs`, `schema_object_view.rs`, `table_ddl_view.rs`, `table_metadata_view.rs`, `table_view.rs` | SUPPORTS | guard test `mod.rs:131-143`; `grep ui.button(` = 0 hits |
| ui-files-ide-workspace-polish | Canonical buttons in files/tasks views | `files_activity_view.rs`, `tasks_view.rs`: 0 raw buttons; both in guard test | SUPPORTS | `mod.rs:136,146` |
| ui-agent-workspace-polish | Agent surface standardized; guard includes agent_view.rs | `agent_view.rs` in guard (`mod.rs:147`); 0 raw buttons; `AgentSettingsAction::SaveKey/ForgetKey` wired (`agent_view.rs:111-124`) | SUPPORTS | `agent_view.rs:106-125` |
| ui-spacing-consistency | IMPLEMENTING; SP01–SP07 fixed; captures collected | `evidence/native/` holds normal+error PNGs at 3 widths (height-capped); 1 unchecked item (full-height display) | EVIDENCE_GAP | `evidence/native/*.png` exist; CHECKLIST last item open |
| ui-stitch-parity-pass | REVIEW; T1–T5 landed with commits | All 5 listed commits exist in history; history surface, filter workbench, status telemetry, compare lock present in code | SUPPORTS | `git log` confirms `7ede34a5` `73340682` `153756e2` `bb137c33` `34601996`; `shell_statusbar_view.rs:98,117`; `evidence/t1,t3,t4` PNGs exist; T5 capture + unchecked item at CHECKLIST:43 |
| settings-shell-visual-polish | RUNTIME_VERIFY; dedicated central surface, chrome suppressed | `app_lifecycle.rs:187-221` `settings_mode` suppresses topbar/tabs/sidebar/dock/statusbar; 4 captures incl. capped 1440/1920 exist | SUPPORTS | `screenshots/settings-{dark,light}*.png` |
| macos-app-menu-refresh | RUNTIME_VERIFY; menus + Refresh/Refresh Cache wired | `app_menu.rs` builds App/File/Edit/View/Window/Help with `Refresh`/`Refresh Cache` (`:103-104`) → `refresh_active_schema_from_menu(invalidate_cache)` (`app.rs:438`) → `UiCommand` → `RuntimeCommand` worker invalidates cache before introspect (`worker.rs:746-751`); no-connection message at `app.rs:440` | SUPPORTS (code) / EVIDENCE_GAP (live menu + translator test) | `app_tests.rs:2973,2995` cover dispatch; CHECKLIST: live observation + translator test unchecked ("No space left on device") |
| native-app-branding | IMPLEMENTING; icon embedded; packaging gap real | `main.rs:445` `include_bytes!("../assets/db-pro-logo.png")` → `with_icon`; decode test `main.rs:668-674`; `.icns`/`.ico`/`.desktop`/PNG assets exist | SUPPORTS | assets at `crates/native-app/assets/db-pro.{icns,ico}`, `packaging/linux/`; installer-pipeline verification still open (CHECKLIST:12) |
| connection-dialog-accessibility | RUNTIME_VERIFY; regressions repaired | `connection/logic.rs:10` rejects port 0 (`port > 0`); `CancelTestConnection` command + `connection.cancel_test` button (`view.rs:243,405`); runtime `CancelTestConnection` + pending-socket test (`worker.rs:167,1512,2754`) | SUPPORTS (code) / EVIDENCE_GAP (live cancel per provider, exact heights) | 3 unchecked: live cancellation PG/SQLite/MySQL/SQLServer, 1440×900/1920×1080 exact heights, independent review |
| native-core-architecture | IMPLEMENTING; feature-owned state + one-way transitions guarded | `scripts/check-ui-architecture.sh` exists, runs in CI (`ci.yml:116`), but **FAILS on HEAD**: `DbProApp` gained `dev_tools` + `capture_evidence_seeded` fields outside allowlist | EVIDENCE_GAP / guard RED | `bash scripts/check-ui-architecture.sh` → "DbProApp fields changed outside the allowlist" (fields added by `0bb549a5` dev-tools wave) |
| native-visual-redesign | IMPLEMENTING; 17 waves; Codex-parity contract | Waves 1–14 coded with captures; Waves 10/15/16 runtime verification + all-surface traversal + independent review open; Wave 17 STALE (says egui 0.29.1 reverted — code is now 0.36.2) | EVIDENCE_GAP + STALE-DOC | `crates/ui/Cargo.toml:9-10` egui 0.36.2; CHECKLIST Wave 17 items describe pre-upgrade state |
| egui-0-36-upgrade | IMPLEMENTING → "verification complete; pending regression triage" | egui/eframe = 0.36.2 in `crates/ui` + `crates/native-app`; upgrade commit `65cbcf75`; post-upgrade regression repair landed (`92746074`, `551a35f7`, `c02ec5ac`) | SUPPORTS | Cargo.tomls; no `CHECKLIST.md` exists (doc gap); no 0.29 leftovers in manifests |
| restore-egui-ui | RUNTIME_VERIFY; rs-ui removed, egui restored | **Zero** `rs-ui`/`rs_ui` hits in `Cargo.toml`, `Cargo.lock`, all `crates/*/Cargo.toml`, and `crates/**/*.rs`; 15 captures at 3 sizes exist | SUPPORTS | `grep -rn rs-ui Cargo.lock` = 0; `screenshots/{empty,error,loading,normal,table}-*.png`; stale doc `docs/rs-ui-integration-plan.md` remains (P2) |
| rs-ui-runtime-integration | BACKLOG — deferred by owner 2026-10-02 | Correctly marked historical; no rs-ui deps/code remain | OBSOLETE (as active work) — doc itself says historical | PLAN header; grep results above |
| ux-friendliness-audit | RUNTIME_VERIFY; audit + remediation on React/Tauri app | React frontend **deleted** from repo (archived by `d368cdc2`; no `frontend/` or `crates/tauri-app` on HEAD); all Wave A–D evidence cites `pnpm vitest`, Tauri commands, React paths | OBSOLETE (frontend-era) — findings may still inform native UX but are not verifiable on HEAD | `ls frontend` → absent; VERIFICATION cites `src/modules/...` vitest files that no longer exist |
| rc1-full-product-qa | Audit program; all P1s FIXED; sign-off gates open | 119 checklist boxes ticked, 37 open; FINDINGS all "Status: FIXED" — but many fixes target deleted Tauri/React paths; open gates are runtime/review/packaging/tag | EVIDENCE_GAP (v0.1 gate list below); partially OBSOLETE for frontend-targeted items | `git log -3 -- frontend` → `d368cdc2` archive; open items: independent review, provider/ER/packaging matrix, tag |
| agent-workflow | RUNTIME_VERIFY; orchestration + confirmations + PG/SQLite | `domain/agent.rs`, `agent_context.rs`, `agent_workflow/` exist; worker orchestrator (`worker.rs:1170-1256` run map, cancellations, confirmations); UI SaveKey/ForgetKey wired; STATUS.md correction notes live-E2E claims rest on tests+prose only | SUPPORTS (code) / EVIDENCE_GAP (retrievable live-provider artifact) | `worker.rs:683,1177,1208`; `agent_view.rs` confirmation cards; screenshots only `agent-panel-1280x800.png` |
| agent-key-secret-store | Done; SecretStore owns key lifecycle | `runtime/lib.rs:421-431` `load/store/delete_agent_api_key` via `self.secrets`; no `keyring::` call in native-app; `UiCommand::SaveAgentApiKey`/`ForgetAgentApiKey` dispatched (`agent_view.rs:115,120`); `configure_request` cleared on failure | SUPPORTS | `grep keyring::` in native-app/runtime = 0; keyring dep only in `crates/infrastructure` |
| license-public-release | Done except final notice bundle | `LICENSE` exists; `workspace.package.license = "MIT"` (`Cargo.toml:29`); all 5 crates `license.workspace = true` | SUPPORTS | only open item: dependency notice bundle for the exact release binary (CHECKLIST:8) |

## 3. UI findings

- **P1 — Architecture boundary guard red on HEAD.** `scripts/check-ui-architecture.sh` fails:
  `DbProApp` has `dev_tools` (`app.rs:45`) and `capture_evidence_seeded` (`app.rs:66` usage) fields
  not in the script's allowlist (added in `0bb549a5`). CI step `ci.yml:116` will block. Either the
  allowlist needs owner-approved extension or the fields must move into a feature aggregate.
- **P2 — Hardcoded chart palette.** `chart_view.rs:24-37` holds 14 literal `Color32::from_rgb`
  values (Google-chart palette) outside `DbProTheme`. Defensible for a data-viz scale, but it
  violates the "no hardcoded colors" letter of AGENTS.md; no token exists for categorical series.
- **P2 — Dev-tools debug overlay hardcoded colors.** `dev_tools/mod.rs:311-317` uses literal
  orange/red debug rects. Acceptable for a diagnostics overlay; noted for completeness.
- **P2 — `shell_topbar_view.rs:528,580` `.expect("root viewport output")`** — inside
  `Context::run`-style plumbing, not a data path; low risk.
- No other hardcoded colors found in `shell_*`, `settings_*`, `agent_*`, `connection/`,
  `files_activity_view`, `tasks_view`, `query_dialogs_view` (all clean).

## 4. Feature findings

- **P1 — `native-core-architecture`: guard regression** (same root cause as UI finding above;
  tracked once here). `dev_tools` feature state was added as a `DbProApp` root field, which is
  exactly the pattern the guard exists to forbid — either it is a legitimate composition-root
  state (update allowlist) or a boundary violation (move it).
- **P2 — Stale doc `docs/rs-ui-integration-plan.md`** describes rs-ui adapters as if live
  ("adapters under crates/ui", staged migration). All code is removed; doc should be archived or
  marked superseded by `restore-egui-ui`.
- **P2 — `native-visual-redesign` Wave 17 stale**: records "egui 0.29.1 Glow, 0.36.2 failed
  compilation/reverted" — superseded by `65cbcf75` (0.36.2 landed). Doc-only staleness.
- **P2 — `ux-friendliness-audit` + `rc1-full-product-qa` frontend-era evidence**: fixes verified
  via `pnpm vitest` on a frontend that no longer exists in-repo (`d368cdc2` archived it). The
  native-port counterparts of the same invariants (dirty guards, i64 precision, preview
  contamination) should be re-asserted against `crates/ui` state rather than inherited.
- **P2 — `egui-0-36-upgrade` has no CHECKLIST.md** (only PLAN/VERIFICATION) — doc completeness gap.
- Test gaps: several plans cite workspace test counts that are now stale (e.g. shell-hierarchy
  483, native-redesign 357/21 — the 21 was even the deleted Tauri package's count). The suite
  has grown (~976 ui tests at `34601996`); counts in VERIFICATION docs are historical, not
  red flags, but no plan should be treated as "fresh-gate-verified".

## 5. Captures taken

- `/tmp/audit-caps/shell/` — **empty**. Two attempts (`DB_PRO_CAPTURE_SETTINGS=1` and default
  welcome, 1280×800, settle 30–45) both timed out at 45–60 s with no framebuffer written and only
  the keyring-disabled log line emitted. This matches the previously recorded window-server
  starvation (invisible window → no `RedrawRequested`) in `restore-egui-ui` and the
  `ui-stitch-parity-pass` T5 note; it is an environment limitation, not retried further.
- Prior-run captures relied upon as existing evidence: `settings-shell-visual-polish/screenshots/`
  (4), `ui-stitch-parity-pass/evidence/` (9), `ui-spacing-consistency/evidence/native/` (12+
  manifest), `restore-egui-ui/screenshots/` (15), `agent-workflow/screenshots/` (1).

## 6. Missing evidence to reach COMPLETED (per plan)

- ui-shell-hierarchy / ui-dialogs-forms-consistency / ui-explorer-polish /
  ui-schema-workbench-polish / ui-files-ide-workspace-polish / ui-agent-workspace-polish:
  fresh native captures at 1280×800/1440×900/1920×1080 on a working window server (none of the
  six has surface captures on file); independent review.
- ui-spacing-consistency: one display that reaches logical 900/1080 heights (host caps at 838).
- ui-stitch-parity-pass: T5 compare safety-lock capture (`DB_PRO_CAPTURE_COMPARE`), light-theme
  captures for history/filter/compare, T6 decision record (deferred), then independent review.
- settings-shell-visual-polish: independent review only (captures already exist).
- macos-app-menu-refresh: live menu observation (Refresh, Refresh Cache, no-connection message,
  Help), native translator test run, final diff review.
- native-app-branding: packaged-bundle metadata once an installer pipeline exists.
- connection-dialog-accessibility: live test-cancel on PG/SQLite/MySQL/SQLServer, exact
  1440×900/1920×1080 captures, owner keyboard pass.
- native-core-architecture: fix/red the boundary guard (`dev_tools`,
  `capture_evidence_seeded`), then final review of reducer/event topology.
- native-visual-redesign: Waves 10/15/16 runtime verification, all-surface two-theme traversal,
  P0=P1=0 sign-off, Wave 17 doc refresh + diagnostics captures, independent review.
- egui-0-36-upgrade: regression triage decision; a CHECKLIST.md; note that follow-on repair
  plans (`connection-dialog-accessibility`) already absorbed known regressions.
- restore-egui-ui: full logical 900/1080 heights on capable display.
- rs-ui-runtime-integration: n/a (owner-deferred BACKLOG; historical doc only).
- ux-friendliness-audit: obsolete for frontend; if retained, re-issue findings against native
  egui surfaces and re-run the D7 matrix natively.
- rc1-full-product-qa: see v0.1 gate list below.
- agent-workflow: retrievable live-provider E2E artifact (STATUS.md correction stands), more
  viewport captures, Preview → GA decision.
- agent-key-secret-store: none for code scope; packaged-runtime secret-store qualification is
  an explicit non-goal.
- license-public-release: dependency notice bundle for the exact release binary.

### v0.1 remaining gate list (from rc1-full-product-qa + license-public-release)

1. Independent Kilo/Cubic (or equivalent) challenge of audit findings — P0=0/P1=0 confirmed.
2. PostgreSQL + SQLite integration suites on release candidate SHA.
3. Native runtime scenario matrix: launch/restart, connect, browse, query run/stop, dirty/staged
   close guards, preview-replacement safety, theme light/dark.
4. ER 500+ table runtime matrix (search-first, neighborhood, Show All, pan/zoom, LOD, memory).
5. PostgreSQL type matrix incl. BIGINT identity fixture; enum/arrays behavior decisions.
6. Packaged build smoke on macOS + Windows + Linux (or recorded delegation).
7. Platform chrome check (shortcut labels, menu/top chrome) — partially covered by
   macos-app-menu-refresh once its live evidence lands.
8. P0=0, P1=0, every P2 explicitly accepted/fixed/deferred; no unresolved review threads.
9. Final docs commit after runtime evidence; exact final SHA green in CI; **fix the currently
   red `check-ui-architecture.sh` first** (it runs in `ci.yml`).
10. Dependency notice bundle for the exact binary (license-public-release CHECKLIST:8), then tag
    `v0.1.0`.

## 7. P0/P1/P2 totals

- P0: 0
- P1: 1 (architecture boundary guard red on HEAD)
- P2: 6 (chart palette literals; dev-tools debug colors; topbar expects; stale rs-ui doc;
  stale Wave-17 doc; frontend-era evidence inheritance across ux-friendliness/rc1)

## 8. Tổng kết bằng tiếng Việt

- Phần lớn các plan polish UI (shell, dialogs, explorer, schema/files/agent workspace) **đúng như
  tài liệu mô tả**: không còn raw button trên các surface đã migrate, guard test
  `primary_native_surfaces_do_not_reintroduce_raw_buttons` phủ đủ file, folder explorer dùng
  theme token.
- `rs-ui` đã được gỡ **sạch hoàn toàn**: 0 tham chiếu trong Cargo.toml/Cargo.lock/mã nguồn;
  chỉ còn `docs/rs-ui-integration-plan.md` là tài liệu lỗi thời (P2).
- **Phát hiện P1 duy nhất**: `scripts/check-ui-architecture.sh` đang FAIL trên HEAD vì
  `DbProApp` có thêm `dev_tools` và `capture_evidence_seeded` ngoài allowlist — CI sẽ đỏ cho tới
  khi sửa.
- Menu macOS (Refresh/Refresh Cache), branding icon, SecretStore cho Agent key, và huỷ
  connection-test đều **có code thật**, chỉ thiếu bằng chứng runtime live.
- `ux-friendliness-audit` và phần frontend của `rc1-full-product-qa` là tài liệu thời React/Tauri —
  frontend đã bị xoá khỏi repo, nên các "FIXED" đó **không kế thừa được** cho native; các bất
  biến tương ứng cần khẳng định lại trên `crates/ui`.
- Gate còn lại trước khi tag `v0.1.0`: review độc lập, ma trận runtime native + provider + ER,
  packaged smoke 3 OS, sửa guard architecture, bundle notice dependencies.
