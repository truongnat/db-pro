---
schema_version: 1
id: "MEM-SRC-9a0ea5fdf3741664"
title: "Source mirror: docs/plans/active/quick-open-action-visuals/AGENT_EVIDENCE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/quick-open-action-visuals/AGENT_EVIDENCE.md"
source_sha256: "9a0ea5fdf37416641003bf49a3c045acd05b0f3b7567f661449786b9e1498abf"
mirror_body_sha256: "9a0ea5fdf37416641003bf49a3c045acd05b0f3b7567f661449786b9e1498abf"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agent evidence — shell borders, Light/Dark, Quick Open regression pass

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex parent; ThemeToggle and ShellBorders implementation slices |
| Issue(s) | Owner's two screenshots, 2026-10-19 |
| Task state | Done for reachable implementation/verification; feature lifecycle RUNTIME_VERIFY |
| Baseline SHA | `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` |
| Branch / PR | main / no PR; no commit created |
| Scope interpretation | Soften shell boundaries, restore consistent rendered Light/Dark selection, keep Quick Open search/results/footer correctly sized |
| Out of scope | Provider logic, SQL, unrelated sample-seed and Clippy failures, broader Welcome layout |

Historical 2026-10-02 changes at `78d914887642dd400c7e2048db9a24d0403619c8` remain documented in PLAN/FINDINGS/VERIFICATION and the original evidence images.

## 2. Progress checkpoint

- Source base: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` + uncommitted patch. Exact candidate file bytes: `evidence/2026-10-19/source-manifest.json`.
- Completed: [x] subtle shell boundaries; [x] toolbar/Settings/Gallery synchronization; [x] explicit egui theme and fresh root style; [x] compact SearchInput; [x] shrinking empty list and sticky keyboard-help footer; [x] native light/dark screenshots.
- Final app/UI tests: 1299 passed / 0 failed / 0 ignored, exit 0. Runtime suite separately: 42 passed / 0 failed / 0 ignored.
- Remaining: [ ] exact 900/1080 logical heights; [ ] independent review; [ ] actual OS-pointer resize/hover checks.
- Risks: no open defect found in the requested changed paths. Wider application failures and inherited visual limitations are not hidden; see VERIFICATION.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; no candidate commit SHA exists; SHA-256 source manifest binds uncommitted files |
| Commit list | none |
| Acceptance mapping | Borders → panel idle separators disabled, semantic frames retained; theme → pointer-driven lifecycle regression + context-style regression; Quick Open → SearchInput geometry regression + footer clipping regression; runtime → 36 final PNGs |
| Commands and counts | Final app/UI 1299/0/0 PASS; workspace fmt/check/UI architecture PASS; native dev-capture/release builds PASS; full workspace tests FAIL 1366/1/55 before abort; both strict Clippy runs FAIL in untouched paths; exact commands in VERIFICATION.md |
| CI run IDs / status | not run |
| Known limitations | Larger images cap at 838 logical px; actual native hover/resize not exercised; restored PostgreSQL connection missing its database; Schema scope therefore empty, not a populated-provider search demonstration |
| Migrations / config implications | No key/schema migration. Capture-only `DB_PRO_CAPTURE_AGENT_LIGHT=1` now supports light Agent evidence. |
| Out-of-scope changes | none |

File/surface inventory (full paths in source manifest):

- `crates/ui/src/components/input/search.rs`: allocate a compact left-to-right row; register `tests/search_tests.rs`.
- `crates/ui/src/components/input/tests/search_tests.rs`: first result remains visible in tall containers, light/dark.
- `crates/ui/src/components/input/API.md`: document compact row behavior.
- `crates/ui/src/theme/mod.rs`: explicitly select context theme and update visuals/style consistently.
- `crates/ui/src/theme/tests/mod_tests.rs`: light→dark→light rendering-context regression.
- `crates/app/src/shell/chrome/shell_topbar_view.rs`, `shell_statusbar_view.rs`: suppress duplicate idle separators.
- `crates/app/src/shell/chrome/shell_chrome_view.rs`: synchronize toolbar preference/repaint; suppress output-dock idle separator.
- `crates/app/src/agent/agent_surface_view.rs`: suppress Agent idle separator.
- `crates/app/src/settings/settings_appearance_view.rs`: repaint after selecting a theme, retain reduced-motion preference.
- `crates/app/src/component_gallery/component_gallery_view.rs`: synchronize theme toggle preference/repaint.
- `crates/app/src/app/app_lifecycle.rs`: refresh root Ui style before shell rendering; register tests.
- `crates/app/src/app/tests/app_lifecycle_tests.rs`: real control pointer events, rendered fills, repeated frames, save/reload.
- `crates/app/src/palette/palette_surface_view.rs`: shrinking results and fixed DialogFrame footer.
- `crates/app/src/palette/tests/palette_surface_view_tests.rs`: keyboard help stays inside the visible dialog.
- `crates/app/src/workspace/workspace_actions.rs`: light Agent capture mode.
- Existing plan's PLAN/CHECKLIST/FINDINGS/VERIFICATION/AGENT_EVIDENCE and `docs/plans/STATUS.md`: current evidence and limitations.
- `evidence/2026-10-19/`: source manifest, initial attempt metadata/logs, and final 36 light/dark native captures.

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus uncommitted manifest-bound patch |
| Verdict | n/a — implementation self-verification is not independent approval |
| P0 / P1 / P2 counts | Requested-path introduced open: 0 / 0 / 0 observed; baseline issues fixed: 4 P2 (separator, theme, search sizing, footer clipping). Wider-repository failures not independently classified. |
| Findings | Failed workspace test/Clippy and clipped-height gate remain recorded; no suppression or unrelated cleanup |
| CI disposition | CI not run; see local command evidence |
| Next tasks unblocked | Independent review and full-height/native-pointer verification |

## 5. Research / audit handoff

- Source date: 2026-10-19.
- References: manifest-owned files above; installed egui/eframe 0.36.2 `context.rs`, `ui.rs`, `containers/panel.rs`, `native/epi_integration.rs`; RULES.md; feature lifecycle; native migration visual gate.
- Baseline mechanisms: `set_visuals` does not choose egui context theme; eframe root Ui precedes App logic; centered unconstrained `with_layout` consumes remaining height; panel idle separators paint independently of frames.
- Measured: search first-result y=796 before fix; footer regression fails before/passes after; final pointer-driven theme and owned suites pass; final native matrix 36/36 exit 0.
- Inference explicitly rejected: settings did not overwrite the runtime theme each frame; applying saved settings happens during restore.
- Inference/limitation: exact relative contribution of egui theme/root snapshot/repaint issues to the original user session is not separately isolated; the regression exercises the complete repaired path.
- PostgreSQL: n/a presentation change. SQLite: n/a presentation change. No provider success inferred from fixtures or the other provider.
- Unresolved: larger requested heights, independent review, true OS hover/resize; saved-connection and sample-seed failures outside this task.
- Downstream issues activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã bỏ separator đen dư, sửa Light/Dark đồng bộ toolbar–Settings và style của egui, thu gọn ô tìm kiếm Quick Open và giữ hướng dẫn phím ở footer. Test app/UI cuối cùng 1299/0/0, build release thành công, đã xem 36 ảnh native Light/Dark. Gate workspace còn lỗi ở sample SQLite/Clippy ngoài phạm vi; ảnh rộng bị giới hạn chiều cao 838, chưa có review độc lập và thao tác resize/hover native nên plan vẫn RUNTIME_VERIFY.

### Border follow-up

Owner rejected the retained explicit frame strokes. At baseline
`95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus manifest-bound uncommitted follow-up,
outer topbar/statusbar/output/Agent/activity boundaries are now borderless.
Rendered-shell regression: 1/0/0; capture build passes; 12 native PNGs exit 0 and
were inspected. Evidence and exact source hashes are under
`evidence/2026-10-19/border-followup/`. Prior full-height/native-pointer limitations remain.
Đã bỏ cả viền frame còn sót, không chỉ separator egui; giữ nguyên focus và resize.
