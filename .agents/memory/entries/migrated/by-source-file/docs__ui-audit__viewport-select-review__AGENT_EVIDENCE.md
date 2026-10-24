---
schema_version: 1
id: "MEM-SRC-2f7089b2034f6fe6"
title: "Source mirror: docs/ui-audit/viewport-select-review/AGENT_EVIDENCE.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/ui-audit/viewport-select-review/AGENT_EVIDENCE.md"
source_sha256: "2f7089b2034f6fe63abe042ce5dbf28df60146ea7190776990a98c545559c6fd"
mirror_body_sha256: "2f7089b2034f6fe63abe042ce5dbf28df60146ea7190776990a98c545559c6fd"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Viewport overflow and complex Select assessment

> **Follow-up (2026-10-23):** F1, F2 and F3 were fixed in
> `docs/plans/active/viewport-select-fit/` (implementation, tests, 12-cell runtime matrix). This
> file remains the pre-fix assessment record at `9b50e7a2`; its findings describe that SHA, not the
> current tree.

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp · source and native-runtime assessment; two read-only source scouts |
| Issue(s) | Owner-reported horizontal overflow and poor complex Select layouts; no issue number assigned |
| Task state | Done — assessment only; defects remain open |
| Baseline SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Branch / PR | `main`; no commit or PR created by this assessment |
| Scope interpretation | Identify causes, inspect real rendered surfaces, distinguish confirmed defects from source-only risks, and map a bounded remediation sequence to existing components/plans. |
| Out of scope | Product-code changes; database execution/mutations; a full application-wide responsive certification; unrelated semantic-audit errors. |

All source references below refer to the exact baseline SHA above. Runtime evidence was produced from that source using the capture-enabled native binary. Capture time is recorded in [capture-manifest.json](evidence/capture-manifest.json).

## 2. Progress checkpoint

- [x] Trace horizontal allocation from core controls to callers and shell panes.
- [x] Assess simple Select versus the grouped connection/schema popup.
- [x] Inspect actual Appearance and open context-picker surfaces in both Light and Dark.
- [x] Run the normal-width capture matrix, record its failed cell and actual viewport limitations, and retain representative screenshots plus raw reports.
- [x] Prioritize findings and map implementation ownership.
- Remaining implementation: all fixes below; this assessment changes no Rust source.
- Build: `cargo build --locked -p db-pro-native --features capture` — exit 0.
- Tests: not run; read-only investigation, no behavior changed.
- Runtime: 11/12 deterministic capture scenarios produced PNG + JSON; the remaining scenario timed out. Interactive captures additionally confirmed both reported classes of defect. See §5 for coverage and limits.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Commit list | n/a — no commit created |
| File / surface inventory | This report; `evidence/capture-manifest.json`; 11 raw matrix JSON reports and 12 launch logs; seven representative PNGs; four live accessibility snapshots. No product files changed. |
| Acceptance mapping | Width overflow → F1; complex picker layout → F2; popup identity defect → F3; broader source-only follow-up → candidate inventory below. |
| Commands and counts | Capture build exit 0; 12 capture launches: 11 report/PNG pairs, each exit 1 with audit findings; one timeout exit 124. No test counts claimed. |
| CI run IDs / status | Not run |
| Known limitations | Full requested heights 900/1080 were not achieved; open-popup and Appearance evidence is at the live window size, not all matrix sizes. No dense PostgreSQL schema list or long-list keyboard journey exercised. |
| Migrations / config implications | None. Isolated temporary runtime data and a uniquely identified temporary app bundle; review process stopped. Current eframe feature graph excludes persistence, so the review did not write the user's eframe preferences. |
| Out-of-scope changes | None |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `9b50e7a27b4d92de5b126e6ecaf3a518448ad97b` |
| Verdict | ACCEPT WITH P2 — assessment handoff, **not** approval to ship the affected layouts |
| P0 / P1 / P2 counts | Introduced by assessment: 0 / 0 / 0. Confirmed inherited findings in scope: 0 / 0 / 3. Source-only candidates are not counted as reproduced defects. |
| Findings | F1: Appearance clipping; F2: oversized sparse context picker; F3: duplicate scroll-area identity. |
| CI disposition | Not run; native capture build and actual runtime inspection only. |
| Next tasks unblocked | Fix core width allocation and grouped-popup geometry/identity, then verify the affected callers. No new issue/PR opened. |

This is self-verification plus source-scout input, not independent approval of an implementation.

## 5. Research / audit handoff

### Confirmed findings

| ID | Severity | Runtime observation | Source cause / ownership | Recommended correction |
|---|---|---|---|---|
| F1 | P2 | In Appearance, Light and Dark are separated across the entire card and the Dark label is cut at the right window edge in both themes. | `crates/ui/src/components/selection/ui.rs:325-355` gives each Radio the entire remaining row width. `components/radio_group/ui.rs:82-106` renders those controls in a horizontal wrapped group; `crates/app/src/settings/settings_appearance_view.rs:15-29` uses that group inside another horizontal row. | Make horizontal radio items allocate their content width or an explicit bounded share; keep full-width behavior for vertical descriptive choices. Verify the group inside a constrained parent, not only a full-window gallery. Do not hide the defect with a clip rectangle. |
| F2 | P2 | The open query context picker is roughly 600 logical pixels wide and has large empty vertical regions for only two connections and one schema. It obscures much of the editor and does not communicate a compact choice list. | `crates/app/src/query_workbench/query_context_picker_view.rs:27-95` sets only a minimum width and creates two max-height-180 scrollers. `crates/ui/src/components/scroll_area/config.rs:1-2` defaults to `auto_shrink([false, false])`. Compare the existing pinned-width/shrinking Select popup in `components/select/ui/select.rs:483-495`. | Pin a content-aware width capped by the viewport. Shrink short sections to content and bound only long lists. Retain clear Connection/Schema headings, compact rows, muted metadata, and sufficient room for the selected check. |
| F3 | P2 | The debug app paints “First use of ScrollArea ID 45C9” and “Second use of ScrollArea ID 45C9” directly over popup rows in both themes. | The two `ScrollArea::new(...)` calls at `query_context_picker_view.rs:43` and `:78` have no distinct `id_salt`. | Give the two scrollers stable, distinct IDs, or use one bounded grouped scroller. Do not suppress the debug warning. Shared scroll-state corruption was not reproduced and is not asserted. |

Evidence, inspected at runtime:

- F1: [Light](evidence/appearance-overflow-light.png), [Dark](evidence/appearance-overflow-dark.png); adjacent `.snapshot.json` files retain the accessible Light/Dark controls.
- F2/F3: [Light popup](evidence/query-context-open-light.png), [Dark popup](evidence/query-context-open-dark.png); adjacent snapshots retain the Connection/Schema groups and choices.
- Live screenshots are 2880×1740 framebuffer pixels at 2× scale: a 1440×870 outer window including native titlebar. They must not be described as 1280×800 captures.
- Debug ID warnings may not be painted in release builds. The duplicate identity and oversized geometry still require correction; release-visible warnings were not tested.

### Source-only candidates — not runtime-confirmed

| Area / source reference at baseline SHA | Evidence and risk | Next verification |
|---|---|---|
| `crates/app/src/workspace/workspace_shell.rs:17-23,74-80`; `app/app_types.rs:301-305`; `app/app_lifecycle.rs:214-288` | Rail, sidebar and agent pane independently reserve width. The agent-open collapse is not a general per-frame central-width policy. [INFERENCE] Certain pane combinations leave too little central space. | Resize the real app with sidebar/agent open and after reopening either pane; record minimum usable central width. |
| `crates/app/src/table/table_workspace_surface_view.rs:34-104,174-213`; `table/table_metadata_surface_view.rs:135-203` | Breadcrumb/actions and metadata tabs/search use fixed reservations and non-wrapping clusters. [INFERENCE] Labels, counts and narrow local panes can exceed their budget. | Long table names and counted tabs with inspector/agent visible. |
| `crates/app/src/query_workbench/query_layout_surface_view.rs:48-61`; `query_view.rs:55-83`; `result_grid/result_grid_view.rs:98-131` | Dock/editor and inspector/main floors compete with the local width. [INFERENCE] Fixed floors can exceed a narrowed center. | Exercise split/inspector state at narrow central widths, not just narrow outer windows. |
| `crates/ui/src/components/select/ui/select.rs:159-190,310-315` | A labelled Select sets its requested child width before rendering; the unlabelled trigger already clamps content width. [INFERENCE] Explicit labelled widths can enlarge a narrow parent. | Labelled Select with a requested width larger than its form column. Do not claim every `.width(...)` Select is broken. |
| `crates/ui/src/components/select/ui/select.rs:495-509` | The option's `selected` state is driven by keyboard draft index rather than separately representing committed value and active candidate. No long-list keyboard journey was exercised. | Arrows beyond eight visible options; verify active row visibility, current-value check, Enter commit and Escape cancel before changing the API. |
| `crates/app/src/connection/view.rs:227-273`; `connection/config.rs:28-46`; `schema_workbench/schema_workbench_form.rs:273-320,478-520,600-800` | Engine choices flatten metadata into strings and include many disabled forthcoming options; schema forms combine fixed-width selectors. These are design/width review candidates, not observed popup failures in this run. | Open the engine menu and dense schema forms; inspect names/metadata/disabled hierarchy and keyboard behavior. |

### Design direction and component mapping

1. **Width belongs to the local parent budget.** Fix the allocation contract in `selection` / `radio_group` and labelled `select`; then audit consumers. At shell level, define a usable central-pane minimum and a deliberate collapse policy rather than adding unrelated magic breakpoints to every view.
2. **Keep simple Select simple.** Short single-value choices need a compact trigger, bounded popup, readable options, a stable current-value check and separate active/hover/focus treatment. A mouse click must not be treated as keyboard-visible focus merely because it gives the widget focus; this assessment did not re-test the separate focus-outline issue.
3. **Treat context selection as a grouped picker.** Connection and schema are different intents. Reuse popup geometry/option-row primitives without flattening them into a single undifferentiated list or inventing a broad new selector framework. For large lists, assess search and keyboard navigation; keep primary names separate from muted environment/host metadata. Search is a recommendation, not an implemented feature.
4. **Responsive behavior should preserve tasks.** Wrap/stack form fields, truncate identity labels with access to the full value, and collapse secondary actions deliberately. Do not globally enable horizontal scrolling or clip controls to make screenshots look contained.

Existing plan ownership, not new duplicate plans:

| Work | Existing plan / code owner |
|---|---|
| Local width contracts and reusable responsive composition | `docs/plans/active/responsive-layout-primitives/PLAN.md`; core controls in `crates/ui/src/components/` |
| Theme/spacing consistency for selector rows | `docs/plans/active/core-component-style-system/PLAN.md`; semantic `DbProTheme` tokens |
| Query context-picker geometry, distinct IDs and row hierarchy | `crates/app/src/query_workbench/query_context_picker_view.rs`; use existing overlay/select primitives |
| Table-specific toolbar/form verification | `crates/app/src/table/` and `schema_workbench/`; `table-indexes-responsive-layout` covers Indexes only, not the entire app |
| File/folder ownership | Existing `component-layer-structure` and completed `repository-folder-mapping`; no structural move needed to fix these defects |

Recommended implementation order: F1 core width allocation → F2/F3 grouped popup → validate the listed narrow-pane consumers → only then expand complex-picker affordances supported by runtime evidence.

### Runtime coverage and limitations

- Native capture binary built successfully with the exact command in §2. No web frontend, Node detector or archived UI was used.
- Matrix: New Connection dialog and closed Select gallery, Light/Dark, requested 1280×800 / 1440×900 / 1920×1080. See the per-scenario manifest for exit codes, dimensions and screenshot hashes.
- At requested 1280×800, actual content was 1280×800. Larger requests were height-clamped to 1440×838 and 1920×838 by the host. Full 900/1080-height acceptance remains unverified.
- `connection-dark-1920x1080` timed out after 40 seconds (exit 124), with no report/image. This is a missing evidence cell, not an overflow diagnosis. Its launch log is retained; it was not silently reclassified as passing.
- The other 11 runs returned exit 1 with audit findings, **not a clean gate**. Raw JSON reports include unrelated semantic/zero-size/alignment findings; those were not promoted into viewport findings here. All 11 generated matrix screenshots were inspected; seven representative PNGs total, including four live defect screenshots, are retained to avoid duplicating every normal-width image.
- Closed gallery/dialog captures generally remained within their normal-width views. They do not establish that open popup, split-pane or long-label cases work.
- `crates/ui/src/dev_tools/audit.rs:395-398,477-480` explicitly skips clipped widgets in zero-size/overlap rules. A clean geometry subset is not proof that controls are not overflowing. Actual image inspection was necessary.
- Loading/error/empty popup states, hover transitions, dense long lists and the complete application matrix were not exercised. This assessment is not a UI release gate.
- SQLite: isolated sample fixture connected and introspected. PostgreSQL: a seeded connection label was visible but not selected/tested; no PostgreSQL runtime behavior is inferred from SQLite.
- No user database mutations or query execution were performed. Only the isolated review app was stopped; the user's running app/preferences were not edited.
- Unit/workspace tests, Clippy, formatting and release build: not run for this source-read-only investigation. They remain implementation gates when product code changes.

## 6. Tổng kết (Vietnamese summary)

Đã xác nhận hai gap trên app thật ở cả Light/Dark: Appearance bị tràn ngang; popup connection/schema quá rộng, nhiều khoảng trống và trùng ScrollArea ID. Gốc nằm ở cấp phát width của Radio/RadioGroup và cách popup cấu hình width/scroll, không chỉ ở trang trí. Đã lưu ảnh, snapshot, raw audit report và mapping component/callsite. Chưa sửa code sản phẩm; không đánh dấu các feature responsive là hoàn tất. Ưu tiên sửa hợp đồng width ở core, sau đó thu gọn popup và tách ID; các rủi ro ở toolbar/split/schema cần runtime xác nhận riêng.
