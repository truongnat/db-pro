---
schema_version: 1
id: "MEM-PLAN-table-indexes-responsive-layout-5b9265b3dbe6"
title: "Plan archive: Table Indexes Responsive Layout"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, table-indexes-responsive-layout]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "5b9265b3dbe64577eec4a2d1dd2f22b8c9f875867d8d829dd799c8999fab1de5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/table-indexes-responsive-layout/PLAN.md"
  - "docs/plans/active/table-indexes-responsive-layout/FINDINGS.md"
  - "docs/plans/active/table-indexes-responsive-layout/VERIFICATION.md"
  - "docs/plans/active/table-indexes-responsive-layout/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/table-indexes-responsive-layout/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Table Indexes Responsive Layout — Plan

State: RUNTIME_VERIFY
Branch: `main` (repository owner requires direct-main workflow)

## Goal

Keep every Indexes table column visible in the standard Table Detail viewport and make long cell values discoverable without overflowing neighboring columns.

## Evidence / baseline

- Baseline source: `crates/ui/src/table_indexes_surface_view.rs` at `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Native 1280×800 capture showed only Index Name, Indexed Columns, Method, INCLUDE, and Predicate; Status was beyond the visible edge.
- The declared fixed widths total 1,020pt before Status. The shared table gives the remaining column an 80pt minimum, yielding 1,100pt in a 928pt viewport.

## Scope

- Size the Indexes columns from the available viewport width, with a minimum table width for narrower windows.
- Truncate cell labels to their actual cell width and expose full values on hover.
- Preserve the shared table component's intended horizontal scrolling when the viewport is below the minimum width.
- Correct the singular index count label.

## Non-goals

- Change index metadata loading, provider introspection, or create/drop behavior.
- Change the shared Table geometry contract or other metadata tabs.

## Architecture and provider matrix

The Indexes pane presents the existing `UiTableInfo.indexes` metadata and issues no database operation. PostgreSQL and SQLite provider behavior is unchanged and not runtime-tested by this layout fix. The active `schema-indexes-runtime` plan continues to own create/drop and provider lifecycle evidence.

| Provider | Support for this change | Runtime evidence |
|---|---|---|
| PostgreSQL | Existing metadata presentation | Not run; deterministic fixture only |
| SQLite | Existing metadata presentation | Not run; deterministic fixture only |

## Acceptance criteria

- [x] At 928pt available width, all six columns fit, including Status.
- [x] Long index names, columns, predicates, and statuses clip to their cell and show the full value on hover.
- [x] A focused geometry regression check fails on the old 1,100pt layout and passes on the responsive layout.
- [x] Native loaded-state capture at 1280×800 reviewed.
- [ ] Review capture at 1440×900 and 1920×1080, plus empty/loading states.
- [x] No PostgreSQL or SQLite query/mutation behavior changed.


## Source: `FINDINGS.md`

# Table Indexes Responsive Layout — Findings

## P2 — Status column is outside the visible Indexes viewport

- Evidence: pre-fix native fixture capture `evidence/indexes-light-1280x800-before.png`; the visible header row ends at Predicate and Status is absent.
- Source: baseline `crates/ui/src/table_indexes_surface_view.rs` at commit `710ba002612c6d71ef2605f99f2a49743b51c3a4` declared fixed widths totalling 1,020pt, plus the shared table's 80pt minimum flex width for Status. The reproduced 928pt viewport cannot contain the 1,100pt request.
- Reproduction: `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport -- --nocapture` failed before the fix with `index columns request 1100 pt from a 928 pt viewport` (exit 101).
- Fix: compute all six widths from the visible viewport; use dynamic cell truncation and hover disclosure. Keep a 640pt minimum content width for genuinely narrow viewports, where horizontal scrolling is expected.
- Verification: the same targeted test passed after the fix; `evidence/indexes-light-1280x800-after.png` shows all six columns and the complete `Enforces uniqueness` status.
- Disposition: addressed in working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`.

## P2 — Singular count uses plural grammar

- Evidence: baseline header formatted `Total: 1 indexes`.
- Fix: render `index` for one and `indexes` otherwise.
- Disposition: addressed in working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`; visible in the after capture.

## Other paths reviewed

- Empty or filtered results use the existing empty state; metadata absence uses the loading state.
- Clicking an index name emits `SelectIndex`, stores the selected name, and renders the detail dialog from the same metadata record.
- These paths were source-reviewed but not exercised in the runtime capture. No provider call or mutation occurs in this presentation path.

## Independent review — 2026-10-04

### P2 (inherited) — Index detail action cannot be triggered by the name label

- Evidence: current Indexes screenshot `evidence/indexes-light-1280x800-after.png`, region `Indexes table / Index Name cell`; the name is visually plain text. In working-tree blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`, `table_indexes_surface_view.rs:158-162` checks `response.clicked()` but creates the widget with `Label::new(...).truncate()` and never assigns click sense. egui 0.29.1 `Label::new` defaults to hover-only sense unless `.sense(Sense::click())` is set ([upstream source](https://github.com/emilk/egui/blob/0.29.1/crates/egui/src/widgets/label.rs)). The same click handling existed at baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` (`table_indexes_surface_view.rs:165-170`), so this is inherited, not introduced by the responsive-width change.
- Failure scenario: clicking the only index name cannot emit `SelectIndex`; repository search shows `TableMetadataView` is the only path assigning `table_index_detail`, so the detail dialog is unreachable through normal UI interaction.
- Recommendation: give the index-name control click and keyboard/accessibility semantics and a clear hover affordance, or remove the misleading click path if details are not intended.
- Status: fixed in current working-tree blob `600e950c84b7128ceee37cba2a938de6d971df94`: the label has click sense, pointer affordance, and button accessibility metadata; `clicking_index_name_opens_its_details` verifies pointer activation. Follow-up visual review also confirmed icon/name spacing at 1280×800. Dialog title text is now hidden while its close button remains. Keyboard and detail-dialog native runtime behavior remain unverified.

### Review coverage gap — wider viewports and other states remain unverified

- Evidence: only the loaded, light-theme 1280×800 logical capture is available. 1440×900 was previously stopped after a stall; 1920×1080, constrained width, empty/loading, and detail-dialog captures are absent.
- The captured state shows all six columns, including Status, within the viewport; no visual overflow is visible at this size. This does not establish the remaining required viewport/state combinations.
- Disposition: keep `RUNTIME_VERIFY`; do not claim the full UI acceptance gate passed.

## Metadata header and modal removal — 2026-10-06
- P2 legacy input had 32px content height plus vertical margins and inherited a different font from the surrounding section/total labels. Indexes and Structure now use one header renderer with the shared 30px compact input token and 13px UI-label font for title/input/total.
- Removed index and column modal renderers, click actions, adapters and their unused TableState selection fields. Names are passive metadata; long names/values retain full content on hover, with index definition available there.
- Baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted changes; no provider command or persisted-data changes.

Foreign Keys now also uses the same metadata filter header. Only its header changed; existing navigation/query behavior is retained. Deterministic Foreign Keys capture route includes a fixture relationship for visual verification.

## Constraints header follow-up — 2026-10-06
- P2: Constraints used the default accent selectable label rather than the parent `tab_button`, and a separate input renderer/font. Type cells inherited tight horizontal spacing.
- Scope: reuse the shared metadata header with a trailing slot, right-align existing category tabs in their original order, reuse parent tab styling, and set Type icon/badge spacing to SPACE_SM. Preserve category/search predicates and provider metadata.

## Dependencies follow-up — 2026-10-06
- P2: separate oversized search header, accent category controls and cramped direction icon/badge gap. Shared metadata header and parent tab component replace the inconsistent renderers. Category filters remain right aligned in their original visible order; total remains visible below on the right to avoid crowding at 1280.


## Source: `VERIFICATION.md`

# Table Indexes Responsive Layout — Verification

## Baseline

- Repository baseline SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Implemented `table_indexes_surface_view.rs` blob: `46f85b3a119c511633dc9a31bc81b74882e05fa6` (uncommitted working tree).
- Native 1280×800 logical capture is 2560×1600 physical pixels at 2× scale.

## Automated evidence

- Before fix: `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport -- --nocapture` → 0 passed / 1 failed / 0 ignored, exit 101; requested width was 1,100pt for 928pt available.
- After fix: same command → 1 passed / 0 failed / 0 ignored, exit 0.
- `cargo build --release --locked -p db-pro-native --features capture` → exit 0.
- `git diff --check -- crates/ui/src/table_indexes_surface_view.rs crates/native-app/src/capture.rs crates/ui/src/workspace_actions.rs` → exit 0.

## UI runtime evidence

- Deterministic PostgreSQL-labeled capture, before: `evidence/indexes-light-1280x800-before.png`.
- Deterministic PostgreSQL-labeled capture, after: `evidence/indexes-light-1280x800-after.png`; all six columns fit at the standard 1280×800 logical viewport, including full Status text.
- The fixture is local and does not connect to a live PostgreSQL database. SQLite UI runtime was not run.
- 1440×900 and 1920×1080 captures, empty/loading states, and the detail dialog remain unverified. A previous 1440×900 capture attempt stalled and was stopped; do not treat it as a pass.

## Follow-up UI fix — 2026-10-04

- Source blobs: `crates/ui/src/table_indexes_surface_view.rs` at `600e950c84b7128ceee37cba2a938de6d971df94`; shared Dialog implementation at `01db1a334ce323d529cf87db0995a4390a048c72`.
- The index icon/name spacing now uses `SPACE_SM`. The detail dialog skips its DDL row and separator when `definition` is empty, as with SQLite autoindexes whose `sqlite_master.sql` is NULL. Its title text is hidden; the close button remains.
- `cargo test --locked -p db-pro-ui clicking_index_name_opens_its_details` → passed.
- `cargo test --locked -p db-pro-ui index_columns_fit_the_standard_table_detail_viewport` → passed.
- `cargo build --release --locked -p db-pro-native` → passed. `cargo fmt --all -- --check` and `git diff --check` → passed.
- After hiding the title text, 24 dialog-filtered tests, the index click test, release build, formatting check, and diff check all passed.
- Fresh native light-theme capture: `evidence/indexes-light-1280x800-spacing-fix.png` (2560×1600 physical pixels at 2×). It confirms the icon/name gap and full count label. The dialog itself was not captured after the change.

## Review limits

- The active `schema-indexes-runtime` plan still owns live PostgreSQL/SQLite index lifecycle verification.
- No workspace-wide test suite, clippy, or independent Kilo review was run.

## Aligned headers and modal removal — 2026-10-06
- Source SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted changes; header blob `671340bb0fee783f5d44d1d6d9c4b3186ec8effe`, Indexes `2e2e28ec8e97d22bf0ac38cd029016a3e8e3e68d`, Structure `e6f37db2ab7cec51c2d09fc72f95b41241535c4c`, Foreign Keys `f0774bf36127da6c96b64058f7df5cfd459a2ec1`.
- Headers share 30px compact field geometry and 13px label font. Indexes/Structure no longer have modal renderer, activation enum/adapter, or selected-detail state. Passive index labels disable text-selection click semantics; full-name/definition and long-value hover disclosure remains.
- UI release tests: 992 passed / 2 failed / 0 ignored, overall exit 101. The passive-index regression passed after disabling selectable label semantics; the two remaining failures concern create-connection command dispatch, outside this patch, baseline attribution unverified. Full command and failures recorded in the Table Profile Quality verification.
- fmt/diff checks pass. Workspace gates/CI skipped. Database commands unchanged; deterministic capture routes are tooling only.
- Reusable lesson: filter headers should share font and control-height tokens, rather than combining legacy TextEdit content height plus margins with separately sized section labels. Modal-only state and actions should be removed with a redundant modal. Recorded here; no global memory write.

## Final code gates — 2026-10-06
- Final native command: `cargo build --release --locked -p db-pro-native --features capture` → PASS, exit 0.
- Final Profile test command: `cargo test --release --locked -p db-pro-ui --lib table_profile_surface_view` → 3 passed / 0 failed / 0 ignored, exit 0.
- Final Profile blob `0c38004312fbcffa6bfa66b3bd5e9408e8b2e555`; shared header blob `b30acdf40773676f337676503e4bdca111917061`, against HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Horizontal Profile overflow keeps measured widths and locates its scrollbar immediately after the rows. Header input border uses semantic theme colors even when idle.

## Final native fixture evidence — 2026-10-06
- `evidence/indexes-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/indexes-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/indexes-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/foreign-keys-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- Requested widths 1280/1440/1920 produced physical images 2560×1600, 2880×1676 and 3840×1676; larger heights capped to 838 logical pixels. Selected 1280 and 1920 captures inspected: input border/font/alignment, full long numeric values in wide layout and horizontal overflow in narrow layout. Static fixtures only; scrolling/key/pointer/provider interactions not exercised. The Foreign Keys fixture exposes an inherited narrow Action-cell wrap/clip limitation; this header task does not modify it.

## Constraints header / Type spacing — 2026-10-06
- Baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted working-tree patch.
- `draw_metadata_filter_header_with_trailing` shares the same label/input typography, height and semantic borders as the other metadata headers. The existing total-label wrapper remains unchanged for its callers.
- Constraints category controls reuse parent `tab_button` (selected fill `theme.surface_hover`) and render in reversed iteration under right-to-left layout, preserving visible All → Primary Key → Unique → Check → Not Null order. Type icon/badge spacing uses SPACE_SM (8 px). Filtering predicates unchanged.
- Native capture build `cargo build --release --locked -p db-pro-native --features capture`: PASS, exit 0. Formatting and diff checks: PASS.
- Loaded fixture captures `evidence/constraints-header-{1280x800,1440x900,1920x1080}.png`: PNG integrity checked. 1280 screenshot inspected for input/label alignment, category position/style, Type spacing. Physical images 2560×1600, 2880×1676, 3840×1676; larger heights capped by host.
- Pending: category/search interaction and loading/error/empty-state captures; live PostgreSQL and SQLite not exercised. Existing full UI suite result 992 passed / 2 failed remains unresolved; those connection tests were not rerun for this display-only patch. Workspace check/clippy/tests skipped.
- Reusable lesson: use the shared parent tab component for child categories and expose a trailing header slot instead of duplicating form sizing/colors. Recorded here; no global memory writes authorized.

- Required shipped build `cargo build --release --locked -p db-pro-native`: PASS, exit 0 (7.96 s); log `/tmp/db-pro-constraints-release.log`.

## Dependencies follow-up — 2026-10-06
- Baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Shared header and parent `tab_button` reused; input sizing/borders/font, right-aligned categories and SPACE_SM direction icon gap match Constraints. Search/direction predicates and OpenTable action unchanged. Removed unused duplicated search renderer.
- Compact `matching of total` count shares the right-aligned tab row (for example, `2 of 3`), with a hover explanation. It uses the same direction and text predicates as the table. Initial screenshot exposed full-height allocation for a row right alignment; wrapping `with_layout` in `ui.horizontal` keeps the count compact above the table.
- Reusable lesson: right-to-left `with_layout` in a vertical parent may consume available height; constrain row allocation with `horizontal` and verify the table remains visible in the screenshot. Recorded here, no global memory writes.
- Existing UI full-suite 992 passed / 2 failed connection tests remain unclassified; not rerun for this reversible display-only patch. Workspace check/clippy/tests and live SQLite/PostgreSQL interactions skipped. Loading/error/empty and category/search/navigation interaction matrix pending.
- Final capture build: PASS, exit 0 (28.47 s). Fresh loaded screenshots `evidence/dependencies-header-{1280x800,1440x900,1920x1080}.png` verified as valid PNGs, sizes 2560×1600, 2880×1676, 3840×1676. 1280 native screenshot inspected: table remains directly beneath compact header/count rows; input border/font, gray active category, right alignment and direction gap visible. Host caps larger logical height to 838.
- Formatting and diff checks passed; no new tests added for presentation-only changes.

- Required `cargo build --release --locked -p db-pro-native`: PASS, exit 0 (5.15 s).
- Final count-layout captures `evidence/dependencies-count-{1280x800,1440x900,1920x1080}.png` are valid PNGs at physical sizes 2560×1600, 2880×1676, 3840×1676. 1280 screenshot inspected: `2 of 3` sits at the right edge of the filter row, while the two outgoing rows remain directly under the header. Capture build and shipped release build both PASS; formatting and diff checks PASS.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/03-data-grid-table-editor.md` verdict: **EVIDENCE_GAP**
(implementation verified; empty/loading + interactive matrix still open). Confirmed
at HEAD: viewport-derived fractional widths with 640pt floor, `.truncate()` +
`on_hover_text`, passive index-name label (modal removed), and
`index_columns_fit_the_standard_table_detail_viewport` **passes at HEAD**.

**Stale-fact correction:** the "992 passed / 2 failed connection tests remain
unclassified" note above is stale — both named tests
(`new_postgresql_connection_defaults_to_tls_require`,
`explicit_disable_selection_is_preserved_on_submit`) **PASS at HEAD**.

Fresh captures: `docs/plans/_audit-2026-10-13/captures/table-indexes_*.png`,
`table-foreign-keys_*.png`, `table-constraints_*.png`, `table-dependencies_*.png`
at 3 sizes (heights capped at 838). Remaining: empty/loading captures, interactive
filter/category/search matrix, live provider round-trip. State stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Table Indexes Responsive Layout

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI bug-fix lane |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Fix the Indexes detail table width so Status is visible at a standard viewport; preserve metadata behavior. |
| Out of scope | Provider index lifecycle, create/drop actions, other Table Detail tabs, unrelated working-tree changes. |

## 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4` (implementation uncommitted).
- Completed acceptance rows: responsive six-column width budget; cell-width truncation and hover values; corrected one-index grammar; 1280×800 loaded capture.
- Remaining acceptance rows: 1440×900 / 1920×1080 and empty/loading/detail-dialog runtime captures; independent review.
- Findings / risks: P2 missing Status column resolved at standard width; narrow viewports below 640pt intentionally retain horizontal scrolling.
- Tests already run: targeted geometry check before fix 0 passed / 1 failed / 0 ignored, exit 101; after fix 1 passed / 0 failed / 0 ignored, exit 0.
- Dependency / blocker changes: none.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `46f85b3a119c511633dc9a31bc81b74882e05fa6` source blob; baseline repository HEAD remains `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Commit list | none; uncommitted |
| File / surface inventory | `crates/ui/src/table_indexes_surface_view.rs` — responsive widths, cell clipping/tooltips, singular count, regression test; `crates/native-app/src/capture.rs` — deterministic Indexes capture route; `crates/ui/src/workspace_actions.rs` — capture helper and consistent fixture name/definition; this directory — plan/evidence |
| Acceptance mapping | P2 width overflow → responsive width calculation + geometry regression test + before/after capture. |
| Commands and counts | Targeted test after fix 1/0/0 exit 0; release capture build exit 0; diff check exit 0. The pre-fix targeted test failed 0/1/0 as expected. |
| CI run IDs / status | not run |
| Known limitations | only the loaded fixture state at 1280×800 was reviewed; long text intentionally needs hover for full content. |
| Migrations / config implications | none; capture-only environment variable `DB_PRO_CAPTURE_TABLE_INDEXES` |
| Out-of-scope changes | existing user working-tree edits were preserved; no shared Table geometry changes. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `46f85b3a119c511633dc9a31bc81b74882e05fa6` working-tree blob; repository baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Verdict | `BLOCK` — the detail control is now pointer-activatable and exposed as a button; required runtime coverage remains incomplete. |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 0; inherited: 0 / 0 / 0 open. Runtime coverage gap is recorded separately. |
| Findings | `FINDINGS.md` — index-name activation fix is verified by a focused egui input test; keyboard/native runtime, wider viewports, and other states remain unverified. |
| CI disposition | not run |
| Next task(s) unblocked | collect remaining viewport/state evidence before closing `RUNTIME_VERIFY`. |

### Fix recheck — 2026-10-04

- Source blobs: `crates/ui/src/table_indexes_surface_view.rs` at `600e950c84b7128ceee37cba2a938de6d971df94`; shared Dialog implementation at `01db1a334ce323d529cf87db0995a4390a048c72`.
- Change: index names use click sense, pointing-hand cursor, and Button widget metadata; the icon/name gap uses `SPACE_SM`; empty definitions no longer leave a blank DDL row or separator; the selected-index dialog hides its title text and keeps the close button.
- Verification: `cargo test --locked -p db-pro-ui clicking_index_name_opens_its_details` passed. A fresh light-theme 1280×800 native capture confirms the index row spacing; keyboard and detail-dialog native GUI were not exercised.
- Dialog API: `Dialog::without_title()` is documented in `crates/ui/src/components/dialog/API.md`; 24 dialog-filtered tests passed after the addition.
- Remaining gate: 1440×900, 1920×1080, narrow width, empty/loading, and detail-dialog runtime captures.

## 5. Research / audit handoff

- Source date: 2026-10-04.
- Source references: `crates/ui/src/table_indexes_surface_view.rs` at blob `46f85b3a119c511633dc9a31bc81b74882e05fa6`; baseline at `710ba002612c6d71ef2605f99f2a49743b51c3a4`; `crates/ui/src/components/table/DESIGN.md`; egui 0.29.1 `ScrollArea` and `Grid` implementation.
- Factual findings: the pre-fix screen omitted Status; the fixed-column request was 1,100pt; the focused test failed before and passed after responsive sizing; the post-fix 1280×800 capture shows all six columns.
- Inference: the user's “Index tab broken” report is explained at standard width by the width budget and invisible overflow, consistent with the supplied 1280 fixture capture.
- Decision / recommendation: keep the shared table contract unchanged and adapt only the Indexes presentation widths.
- Unresolved questions: wider viewport behavior, loading/empty state visuals, and live provider data remain unverified.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã sửa lỗi cột Status bị đẩy ra ngoài viewport: ở 928pt, cấu hình cũ đòi 1.100pt. Chia lại sáu cột theo bề rộng, cắt nội dung theo ô và giữ tooltip; test hình học đã đỏ trước sửa, xanh sau sửa. Capture fixture 1280×800 xác nhận đủ cột; viewport lớn hơn, trạng thái rỗng/loading, dialog và provider thật còn chờ xác minh.

## Consolidated owner UI follow-ups — 2026-10-06

### 1. Claim
- Agent: Codex implementer; task state Review. Branch main; PR/commits: none.
- Baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted changes. Final Profile blob `0c38004312fbcffa6bfa66b3bd5e9408e8b2e555`; metadata-header blob `b30acdf40773676f337676503e4bdca111917061`.
- Scope: Profile fits and sizes columns from actual text, preserves numeric content with horizontal overflow; shared Table protects badge border; aligned Indexes/Structure/Foreign Keys filter headers; redundant Indexes/Structure modals removed. Provider behavior and connection-form defects outside scope.

### 2. Progress checkpoint
- [x] Shared Table width allocation and badge-border clip.
- [x] Content-measured numeric column widths; no numeric-string ellipsis; horizontal scrollbar directly under rows.
- [x] Shared 30px input / 13px label font, centered header and visible idle input border.
- [x] Index/column modal, activation adapters and unused selected-detail state removed; full-value hover disclosure retained.
- [ ] Full feature gates: two create-connection tests failed, baseline attribution not verified; loading/error/empty-result and direct interaction/VoiceOver matrix pending.

### 3. Implementation handoff
- Code inventory: `table_profile_surface_view.rs` (shared measured table/render regression); `components/table/mod.rs` (content-width builder), `components/table/ui.rs` (stroke-safe cell clip), `components/table/tests.rs` (clip regression); `table_workspace_surface_view.rs` (shared header); `table_indexes_surface_view.rs`, `table_structure_surface_view.rs` (passive metadata/modal removal); `table_metadata_view.rs`, `table_structure_view.rs`, `table_state.rs` (remove modal adapters/state); `table_relations_surface_view.rs` (header); `workspace_actions.rs`, `crates/native-app/src/capture.rs` (deterministic numeric/Structure/Foreign Keys routes). Plan/checklist/findings/verification/status/evidence captures updated in the two existing plan directories.
- Commands: `cargo test --release --locked -p db-pro-ui --lib` → 992 passed / 2 failed / 0 ignored, exit 101; `cargo test --release --locked -p db-pro-ui --lib table_profile_surface_view` → 3 passed / 0 failed / 0 ignored, exit 0 after final scrollbar change; `cargo build --release --locked -p db-pro-native --features capture` → exit 0 after final code changes; fmt/explicit included-source rustfmt/diff checks → exit 0. CI, workspace checks/clippy/benchmarks skipped. Initial failed/interrupted attempts are recorded in VERIFICATION.
- Two failing tests: `explicit_disable_selection_is_preserved_on_submit`, `new_postgresql_connection_defaults_to_tls_require`. No code change made to those flows; inherited-versus-regression attribution unverified.
- No persisted-data/config migration. Capture env flags are fixture tooling only. No performance gain claimed.

### 4. Review outcome
- Source reviewed: baseline SHA above plus identified working-tree patch; self-review only, not independent approval.
- Narrow patch verdict ACCEPT WITH P2; broader release readiness BLOCK until failed gates are resolved/classified. No introduced P0/P1 identified in this presentation patch; P2 pending interaction/state coverage. Inherited P0/P1/P2 counts unknown without baseline reproduction.

### 5. Research / audit handoff
- n/a; source inspection/capture references and actual commands are in VERIFICATION. No live database claim: PostgreSQL and SQLite command paths unchanged; fixtures only.

### 6. Tổng kết bằng tiếng Việt
Đã sửa Profile giãn theo khung và đo nội dung để không cắt số; viền badge Key còn nguyên. Header Indexes/Structure/Foreign Keys cùng chiều cao/cỡ chữ và viền input; đã bỏ modal cùng state/action thừa. Build native đạt, 3 test Profile đạt; toàn bộ UI 992 đạt nhưng còn 2 lỗi form connection chưa phân loại baseline. Chưa commit/push; các gate tương tác/loading/error/empty-result và VoiceOver vẫn chờ.

## Constraints follow-up handoff — 2026-10-06
### 1. Claim
P2 presentation fix; state Review / RUNTIME_VERIFY. Baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`, uncommitted on main, no PR.
### 2. Progress checkpoint
Shared input/label, right-aligned parent-style category controls, and Type icon/badge spacing implemented. Three loaded fixture capture widths collected; full runtime matrix pending.
### 3. Implementation handoff / review request
Files: `table_metadata_surface_view.rs`, `table_workspace_surface_view.rs`, `workspace_actions.rs`, native `capture.rs`, and this plan's evidence/docs. Preserve provider metadata, category values and search predicates. Capture build, formatting and diff checks passed. No new tests for this reversible display-only change; existing full-suite two connection failures remain unclassified. Self-review: ACCEPT WITH P2 for scoped visual patch; no independent approval.
### 4. Research / audit handoff
n/a; source and deterministic native fixture evidence only, no live provider verification.
### 5. Review outcome
P0 0 / P1 0 / P2 1 pending full interaction/loading/error/empty matrix. Broader release gate remains open due existing unclassified tests. No commit/push.
### 6. Tổng kết bằng tiếng Việt
Constraints dùng chung header, tab lọc đồng màu tab cha và căn phải; icon Type cách badge 8 px. Đã có ảnh native ba chiều rộng; chưa xác nhận toàn bộ ma trận trạng thái và provider.

- Required shipped build `cargo build --release --locked -p db-pro-native`: PASS, exit 0 (7.96 s); log `/tmp/db-pro-constraints-release.log`.

## Dependencies follow-up handoff — 2026-10-06
### 1. Claim
P2 presentation fix, state Review / RUNTIME_VERIFY. Baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch on main; no PR.
### 2. Progress checkpoint
Shared header, parent-style right-aligned categories and 8 px direction icon/badge gap implemented. Loaded native fixtures at three widths captured; complete interaction/state matrix pending.
### 3. Implementation handoff / review request
Files: metadata surface, workspace capture helper, native capture route and this plan's evidence. Duplicate search renderer removed once both metadata tabs use the shared header. Search/direction predicates and table navigation preserved. Capture build, formatting and diff checks passed; no new tests for reversible visual change. Nativ

… [truncated for memory entry; see source file]

