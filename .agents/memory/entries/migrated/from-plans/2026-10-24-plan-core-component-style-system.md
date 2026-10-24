---
schema_version: 1
id: "MEM-PLAN-core-component-style-system-15061bb58387"
title: "Plan archive: Core Component Style System"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, core-component-style-system]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "15061bb58387e480b6e1532c7a57568645e5aec852a6145836c63f6892677014"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/core-component-style-system/PLAN.md"
  - "docs/plans/active/core-component-style-system/FINDINGS.md"
  - "docs/plans/active/core-component-style-system/VERIFICATION.md"
  - "docs/plans/active/core-component-style-system/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/core-component-style-system/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Core Component Style System

## State
- **State:** IMPLEMENTING
- **Branch:** `main`
- **Baseline SHA:** `dc87da1d`
- **Surface:** `crates/ui` native egui theme and core components.

## Goal
Calibrate the native core component system to a coherent dark database workstation inspired by shadcn Primitives & Controls: crisp borders, restrained surfaces, compact controls, clear focus/selection states, and a blue action accent.

## Design context
- **Users:** database developers and analysts working long sessions.
- **Jobs:** navigate schema, run queries, inspect result sets, filter data, and make controlled edits.
- **Tone:** calm, technical, precise.
- **Reference:** the supplied dense dark IDE screenshot; adapt its hierarchy rather than copying a web implementation.

## First batch scope
- `DbProTheme` dark surface, border, text, accent, focus and widget visual tokens.
- Shared spacing/radius/control-size tokens used by core controls.
- `Button`, `Input` family, `Card`, and shared focus/interaction treatment.
- Characterization tests for token relationships and existing component behavior.
- Runtime capture at 1280×800, 1440×900, and 1920×1080 for the affected component gallery/shell surface when available.

## Non-goals
- No database/runtime/provider behavior changes.
- No broad view-by-view restyle in this batch.
- No new component framework or web/Tauri restoration.
- Preserve public Rust APIs and interaction semantics unless a separate finding proves a correctness issue.

## Design principles
1. Surface hierarchy uses quiet value steps and 1px borders instead of heavy cards or glow.
2. Controls are compact but keyboard/focus states remain visible.
3. Blue is reserved for action, focus and active navigation; semantic colors communicate status only.
4. Shared tokens belong in `DbProTheme`/`tokens.rs`; views do not invent colors.
5. Dense data remains scannable through alignment, row rhythm and restrained selection fills.

## Acceptance criteria
- Core components use semantic theme/tokens with no new raw color literals in views.
- Dark mode matches the reference direction while light mode remains readable.
- Existing public component APIs and focused behavior tests remain intact.
- Rust fmt/check/clippy/tests/native release build pass.
- Runtime visual evidence is recorded separately from automated evidence.

## Provider impact
N/A — native UI styling only; PostgreSQL and SQLite behavior are unchanged.

## Tổng kết bằng tiếng Việt
Batch đầu tập trung vào design system và core primitives, không restyle toàn bộ view cùng lúc. Mục tiêu là nền dark workstation, border rõ, control compact, focus/selection dễ nhận biết và giữ nguyên API/hành vi hiện tại.


## Source: `FINDINGS.md`

# Core Component Style System — Findings

## Baseline
- The current native UI already has centralized `DbProTheme` and `tokens.rs`, but the dark palette is neutral gray and does not establish the screenshot's layered database-workstation hierarchy.
- Core components own additional local dimensions and colors, so the first batch must consolidate shared visual decisions without changing their public APIs.
- The supplied reference is a dense table/database IDE surface: compact controls and strong alignment matter more than decorative cards.

## Evidence-backed scope
- P2 — visual inconsistency and weak hierarchy: theme surfaces, borders, control sizes and focus treatment need a single calibrated contract.
- No P0/P1 correctness finding identified; this is a visual/maintainability batch.
- PostgreSQL and SQLite behavior are unaffected.

## Decisions
- Keep `DbProTheme` as the semantic color owner.
- Keep `tokens.rs` as the shared spacing/radius/type/size owner.
- Preserve component constructors/builders and move only visual constants to canonical tokens where the current behavior allows it.
- Prefer subtle motion and visible focus over hover-only affordances.

## Data Grid clarity follow-up — 2026-10-04
- P2 — The user clarified the intended DBeaver-like interaction: show `SELECT * FROM <selected table> WHERE` up front and let the user type only the condition. The toolbar now shows a fixed, quoted schema/table prefix, appends only the editable condition on Run/Enter, and keeps Add Row as the first action (disabled when the current result/connection is read-only).
- P2 — The single-page footer showed inert first/previous/next/last controls. The working-tree fix hides page navigation when there is no adjacent page and retains the range and page-size selector.
- Runtime evidence — native light captures at 1280×800 (2560×1600 pixels), 1440×900 (2880×1676 pixels; host-capped to 838 logical px high), and 1920×1080 (3840×1676 pixels; host-capped to 838 logical px high) confirm the fixed prefix, condition-only input, Add Row placement, and no inert one-page navigation.
- Live SQLite evidence — from the `customers` table, entering `id = 2` and pressing Enter showed Bob; entering `id = 1` and clicking Run showed Alice. Both runs reported one row and made no data changes. The capture-driver screenshots use its deterministic PostgreSQL fixture; no live PostgreSQL query was executed.

## Data Grid WHERE badge and Add Row follow-up — 2026-10-04
- P2 — The displayed fixed SQL prefix (`SELECT * FROM ... WHERE`) took too much toolbar width. Keep its qualified and quoted SQL generation internal, and render only a compact `WHERE` badge beside the editable condition.
- P2 — Add Row was disabled after running the inline table condition because toolbar mutation capability treated every inline query result as read-only. Remove that blanket toolbar lock while retaining writable-connection and in-flight-query guards; cell editing continues to use its independent read-only check.
- Regression proof — the focused toolbar-capability test failed before the fix and passed after it; the companion test covers read-only connections and queries in flight. Live SQLite interaction separately verifies the filtered-result case.
- Runtime proof — on the writable SQLite sample, `id = 2` returned Bob and Add Row opened the `Insert Row · customers` form. The form was closed without staging or saving, leaving sample data unchanged.
- Visual evidence — rebuilt native capture at 1280×800, 1440×900, and 1920×1080 shows Add Row first and only the WHERE badge; higher-width captures remain host-height capped. Capture driver uses deterministic fixture data, so only the separate CUA check is live SQLite evidence.

## Data Grid filtered-save refresh follow-up — 2026-10-04
- P2 — After staged insert completion, `staged_apply_completed()` cleared the displayed result and called `request_table_data()`, which always dispatched an unfiltered table load. The inline condition existed only as an editable draft, so the displayed result expanded to all rows and lost the user's filter.
- Fix — store the last successfully completed inline SQL separately from the draft; regular table refreshes now reissue that SQL while it is active. A failed/cancelled newer query leaves the previous successful filter in place; changing tables clears both pending and active SQL.
- Regression proof — `staged_save_completion_reruns_the_active_inline_sql` failed before refresh routing was changed and passed after; query-state coverage checks success, stale request IDs, cancellation, and table reset.
- Reusable lesson — refreshing after mutation must use the committed read query, not the editor's draft or an unconditional base-table load.

## Data Grid condition autocomplete follow-up — 2026-10-04
- P2 — The inline condition field had no table-column completion and used a frameless editor, leaving users to remember identifiers and making focus unclear.
- Fix — suggestions now come from the selected table's `UiTableInfo.columns`, match the identifier around the caret, and appear as a foreground popup without changing toolbar layout. Up/Down changes selection; Enter/Tab or a click inserts a quoted identifier and restores editor focus; Escape dismisses suggestions until the draft changes.
- Guard — candidates are offered at condition starts and after `AND`/`OR`, not in values or single-quoted SQL strings. An exact column name closes the popup so Enter remains available for query execution.
- Automated evidence — four focused toolbar tests cover table-scoped prefix matching, value/string suppression, middle-caret token replacement, and the existing quoted table prefix.
- Runtime limitation — the release build passed, but the running DB Pro Measure process could not be reattached after the CUA close/relaunch attempt. Popup interaction and viewport captures are therefore still pending; do not count source/tests as visual runtime proof.

## Implementation result
- The initial calibration used a charcoal/navy surface ladder (`#17191c` → `#23272d`) with a restrained blue accent (`#4f8cff`). Wave 15 of `native-visual-redesign` supersedes those shared surface and accent values with neutral Codex-aligned tokens and contrast-checked foregrounds.
- Selection uses a muted blue wash instead of a neutral gray fill, preserving scanability in dense data surfaces.
- Shared card padding and control-group spacing are tighter; Button padding and icon gaps are compacted without changing the public builders or click behavior.
- The 1440×900 component-gallery capture confirms the calibrated palette and control hierarchy in the native app.

## Gallery review finding
- The supplied gallery screenshot exposed two concrete P2 issues: selection controls painted labels/descriptions at intrinsic no-wrap widths, and the preview included an orphaned `Custom JDBC Driver` label with no state or affordance.
- Root cause was in the reusable `Checkbox`/`Radio` primitives plus gallery copy, not in the database shell.
- The primitives now calculate text width from their local cell, wrap descriptions, and grow row height accordingly. The gallery uses a disabled outlined control with explicit preview copy instead of a misleading free-floating label.
- Disabled buttons now retain a visible border/surface and readable disabled text, so unavailable states do not look like broken black rectangles.

## Disclosure & choice redesign
- The follow-up screenshot showed a second P2 problem: Toggle, ToggleGroup, Collapsible/Accordion and Calendar/DatePicker were packed into one undifferentiated block with weak hierarchy, excessive copy, and an icon/title overlap in Accordion rows.
- The gallery now uses three purpose-built, equal-width panels with compact descriptions: `Toggle & ToggleGroup`, `Collapsible & Accordion`, and `Calendar & DatePicker`.
- The inline calendar preview makes the Calendar primitive visible instead of showing only an unopened DatePicker trigger. The disclosure sample uses shorter database-workstation labels and the Accordion icon advance now reserves the full icon box before the title.
- The outer nested Card was removed from this subsection; panel borders and spacing provide grouping without a card-inside-card stack.

## Runtime evidence
- Updated dark selection-gallery capture: `evidence/disclosure-choice-controls-1440x900.png`.
- Responsive captures: `evidence/disclosure-choice-controls-1280x800.png` and `evidence/disclosure-choice-controls-1920x1080.png`.
- The captures show compact three-panel composition, visible calendar state, readable disclosure icons/titles, and no nested outer card.

## Runtime limitations
- Loading/error/empty state traversal for this gallery subsection remains pending.

## Cards & Metric Displays surface follow-up
- The supplied light gallery screenshot exposed a P2 density problem: the fixed two-column composition stretched metric cards across the whole gallery, creating excessive horizontal whitespace and weakening the visual grouping.
- The surface now uses the shared `ResponsiveGrid` with a `240px` minimum card width and semantic `SPACE_MD` gaps. It keeps four metrics in a stable 2×2 arrangement at medium widths, switches to four columns only when the available region can support all four, and collapses to one column when the gallery becomes narrower.
- Metric titles now use `text_secondary` instead of `text_tertiary`, preserving the theme-owned palette while restoring readable hierarchy against the elevated card surface.
- The first responsive-grid pass only set horizontal `item_spacing`, so stacked metric rows touched vertically. `ResponsiveGrid` now owns both axes: it neutralizes inherited row spacing and inserts the shared gap token between rows, keeping the 2×2 surface rhythm consistent without affecting neighboring gallery content.
- Runtime captures at 1280×800 and 1440×900 show the balanced 2×2 layout; the 1920×1080 capture shows the four-column wide layout. The light capture route is test tooling only and does not change normal startup behavior.

## Calendar & DatePicker follow-up
- The large right-side blank area was caused by `Frame::show` inheriting the gallery column's full available width. The Calendar content had a fixed width, so the frame painted a wide surface around a left-aligned grid.
- Calendar now allocates an intrinsic `272px` frame (`248px` seven-column grid plus symmetric `12px` margins), removing the former one-sided `16px` width reserve. DatePicker uses the same measured size when clamping its foreground popup to the viewport.
- A second clipping defect remained at narrow gallery widths: the fixed `ui.columns(3)` layout could give the Calendar panel less than `272px`, and fixed `32px` cells then pushed the last columns outside the clip rect. The gallery now uses `ResponsiveGrid` and Calendar scales its cells from the local available width, so all seven columns remain visible instead of being clipped.
- The remaining right-side gap came from the header's flexible middle layout: the month title consumed the row's remaining width before the next-button allocation. The header now uses fixed `prev | title | next` regions with zero inter-item spacing and the same content width as the grid.
- Existing correctness coverage includes Gregorian leap/month lengths, strict ISO parsing, month/year wrapping, weekday math, normalized dates, current-date initialization, selection-close, Escape-close, disabled-popup cleanup, cross-month day selection, and click-outside close behavior in source.
- Coverage is not complete for a production DatePicker: there is no keyboard focus/navigation path, direct editable text input, min/max or per-day disabled state, locale/week-start configuration, or clear/today action. Those are explicit follow-up scope rather than silently counted as covered.
- Focused layout tests: `calendar_frame_size_is_intrinsic_and_includes_symmetric_margin`, `calendar_layout_shrinks_cells_when_the_gallery_column_is_narrow`, and `cale

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Core Component Style System — Verification

## Current checkpoint
- Baseline commit: `dc87da1d`.
- Design context confirmed by the user: database developers/analysts; dark database workstation; shadcn Primitives & Controls reference.
- Source changes: `theme.rs`, `tokens.rs`, Button config/handler, selection wrapping, Accordion spacing, Calendar intrinsic sizing/popup clamping, responsive narrow-column Calendar scaling, fixed-width Calendar header regions, responsive gallery panel layout, focused gallery capture routing, gallery panel state, plan files, and `.impeccable.md`.
- `cargo fmt --all -- --check`: PASS.
- `cargo test -p db-pro-ui theme::tests --lib`: PASS (4 passed).
- `cargo test -p db-pro-ui components::button --lib`: PASS (7 passed).
- `cargo test -p db-pro-ui components::input --lib`: PASS (13 passed).
- `cargo test -p db-pro-ui --lib`: PASS (929 passed, 0 failed, 0 ignored).
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; the `db-pro-ui` segment reported 929 passed / 0 failed / 0 ignored, with provider/SSH/SQL Server fixture cases ignored where external fixtures were unavailable.
- `cargo build --release --locked -p db-pro-native`: PASS.
- Clean-code scan: PASS (14 pass, 2 warning categories, 0 fail); warnings are inherited numeric casts and existing long UI methods.
- Runtime evidence: disclosure/choice captures written to [`evidence/disclosure-choice-controls-1280x800.png`](evidence/disclosure-choice-controls-1280x800.png), [`evidence/disclosure-choice-controls-1440x900.png`](evidence/disclosure-choice-controls-1440x900.png), and [`evidence/disclosure-choice-controls-1920x1080.png`](evidence/disclosure-choice-controls-1920x1080.png).
- The captures verify the three-panel composition, inline Calendar preview, compact Toggle/ToggleGroup, readable Accordion icon/title spacing, and responsive behavior at all three viewports. The narrow capture also verifies that header navigation does not expand the right side beyond the grid.
- Calendar verification: the capture-only `calendar` gallery target produced [`evidence/calendar-datepicker-800x800.png`](evidence/calendar-datepicker-800x800.png), [`evidence/calendar-datepicker-1280x800.png`](evidence/calendar-datepicker-1280x800.png), [`evidence/calendar-datepicker-1440x900.png`](evidence/calendar-datepicker-1440x900.png), and [`evidence/calendar-datepicker-1920x1080.png`](evidence/calendar-datepicker-1920x1080.png). The narrow capture proves all seven columns remain inside the frame; wider captures retain the intrinsic size and balanced margins.
- Calendar unit tests: `cargo test -p db-pro-ui components::calendar:: --lib` → 10 passed / 0 failed / 919 filtered.
- Loading/error/empty state traversal remains pending.

## Cards & Metric Displays surface follow-up
- Implementation SHA: `8ca9cac5470ec6688c52aff4cc7e09a0aacdad16` (`polish(ui): tighten metric card surface`).
- Source changes: `component_gallery_surfaces.rs` replaces the fixed two-column rows with a responsive metric grid; `components/card/ui.rs` promotes metric labels to `text_secondary`; `workspace_actions.rs` adds a light-theme/cards capture route.
- Runtime evidence: [`evidence/cards-metrics-1280x800.png`](evidence/cards-metrics-1280x800.png), [`evidence/cards-metrics-1440x900.png`](evidence/cards-metrics-1440x900.png), and [`evidence/cards-metrics-1920x1080.png`](evidence/cards-metrics-1920x1080.png).
- Visual result: medium widths remain balanced at 2×2 without a one-card orphan row; the wide target uses four columns; labels remain legible in the light semantic theme.
- Verification after the implementation SHA: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (UI segment: 929 passed), `cargo build --release --locked -p db-pro-native`, clean-code scan (14 pass / 2 warning categories / 0 fail), and `git diff --check` all passed.
- Loading/error/empty state traversal for this static gallery surface remains pending.

# Data Grid condition autocomplete — 2026-10-04

- Baseline source SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; implementation is uncommitted in the working tree, so this SHA identifies the base only.
- `cargo fmt --all` → exit 0.
- `cargo check --locked -p db-pro-ui` → exit 0.
- `cargo test --locked -p db-pro-ui table_data_toolbar_surface_view::tests -- --nocapture` → 4 passed / 0 failed / 0 ignored, exit 0.
- `cargo clippy --locked -p db-pro-ui --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` and `git diff --check` → exit 0.
- `cargo build --release --locked -p db-pro-native` → exit 0.
- CUA runtime capture/interaction → not verified. The previous DB Pro Measure app could not be reattached after relaunch; the new release binary started from the shell but was not exposed as a CUA app target.
- PostgreSQL / SQLite runtime matrix: not applicable to completion matching; no database request is issued until the existing Run action is invoked.

## Data Grid clarity and one-page follow-up — 2026-10-04
- `cargo fmt --all` and `git diff --check`: PASS.
- `cargo check -p db-pro-ui --lib`: PASS.
- `cargo test -p db-pro-ui table_data_pagination_view::tests --lib`: 2 passed / 0 failed / 0 ignored, exit 0.
- `cargo build --release --locked -p db-pro-native`: PASS.
- Native runtime evidence: `db-pro-native` opened the SQLite sample database. Captures at 1280×800, 1440×900, and 1920×1080 are in [`evidence/data-grid-clarity-light-1280x800.png`](evidence/data-grid-clarity-light-1280x800.png), [`evidence/data-grid-clarity-light-1440x900.png`](evidence/data-grid-clarity-light-1440x900.png), and [`evidence/data-grid-clarity-light-1920x1080.png`](evidence/data-grid-clarity-light-1920x1080.png). The 1440×900 and 1920×1080 targets were capped by the host display to 838 logical pixels high (physical captures 2880×1676 and 3840×1676); 1280×800 measured 2560×1600 physical pixels.
- Live SQLite interaction: Run with `SELECT * FROM customers WHERE id = 1` and Enter with `SELECT * FROM customers WHERE id = 2` each rendered exactly the matching row; the native status reported `Query completed · 1 rows`. This exercised the read-only sample database only; PostgreSQL remains unverified.
- Startup emitted missing replacement-glyph warnings (`◻` / `?`); they did not prevent rendering or query execution.
- Visual changes under test: single-page pagination hides navigation controls while retaining row range/page size; SQL draft is explicitly labeled and has a visible Run action.

## DBeaver-style WHERE condition follow-up — 2026-10-04
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo test -p db-pro-ui table_data_toolbar_surface_view::tests --lib`: 1 passed; `cargo test -p db-pro-ui table_data_pagination_view::tests --lib`: 2 passed.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS.
- Live SQLite UI: condition `id = 2` submitted with Enter rendered Bob; `id = 1` submitted with Run rendered Alice; both status messages reported one row.
- Normal-state captures: [`evidence/data-grid-condition-light-1280x800.png`](evidence/data-grid-condition-light-1280x800.png), [`evidence/data-grid-condition-light-1440x900.png`](evidence/data-grid-condition-light-1440x900.png), and [`evidence/data-grid-condition-light-1920x1080.png`](evidence/data-grid-condition-light-1920x1080.png). Physical captures are 2560×1600, 2880×1676, and 3840×1676 pixels; host display caps the latter two to 838 logical pixels high. Capture driver uses deterministic PostgreSQL fixture data.
- The rebuilt UI shows Add Row before the quoted fixed prefix, with only the predicate editable. PostgreSQL live execution and loading/error/empty capture states remain unverified.

## WHERE badge and Add Row follow-up — 2026-10-04
- Baseline / source identity: HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4`; working-tree blob IDs: `table_data_toolbar_surface_view.rs` `e8dda4c6f471ef2ff728fc593e5bfc305460e841`, `table_data_view.rs` `9ebcd0770650ea7c24b9deff1c3da0caacf2a5ca`.
- Red/green regression: focused toolbar-capability test failed before the fix (0 passed / 1 failed, exit 101); after the fix, `cargo fmt --all -- --check && cargo test -p db-pro-ui table_data_view::tests --lib` passed (2 passed / 0 failed, exit 0). Live SQLite interaction separately verified the filtered-result case.
- Build: `cargo build --release --locked -p db-pro-native --features capture` passed after the UI changes. Earlier focused prefix and pagination tests remain recorded above.
- Live SQLite interaction: entered `id = 2`, pressed Enter, and saw Bob with `Query completed · 1 rows`; clicked Add Row and verified the `Insert Row · customers` form opens while filtered. Closed with Escape without staging or saving; refreshed and confirmed all three original rows remain.
- Fresh captures: `evidence/data-grid-where-badge-light-1280x800.png` (2560×1600 physical), `evidence/data-grid-where-badge-light-1440x900.png` (host-capped to 838 logical px high, 2880×1676 physical), and `evidence/data-grid-where-badge-light-1920x1080.png` (host-capped to 838 logical px high, 3840×1676 physical). The 1280 capture was inspected; captures use deterministic PostgreSQL fixture data, not a live PostgreSQL connection.
- Result: the full prefix is no longer visible; `WHERE` is the only prefix badge. Add Row and Save toolbar availability no longer follow the filtered result-set flag, while cell edit restrictions remain independent.
- Limitations: live PostgreSQL not verified. No database insert was committed; the verified interaction covers opening the insert form only. Loading/error/empty states remain pending.

## Filtered result after staged save — 2026-10-04
- Baseline HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`. Working-tree source blob IDs: `table_data_query_state.rs` `59b69eebaa9936226017b42c0ffc45ee64ea8f29`; `table_editor_view.rs` `fcb02fb218c2314fa27446f456133d16cc10504d`; `events_query_dispatch.rs` `237c8ba0b4020ddffcabd7b5738307eb952d57df`; `query_result_events.rs` `09b241274eb0ec572e42e027cd9b95076b601fea`; `runtime_event_handlers.rs` `392748a2fb85af9f3db3a2335eac53e5e7d8bfb7`; `app_tests.rs` `c7493a1668bd87e9a2de25eda6986c0b7cc6674f`.
- Red/green: `cargo test -p db-pro-ui staged_save_completion_reruns_the_active_inline_sql --lib` failed before refresh routing (0 passed / 1 failed, exit 101); passed after (1 passed / 0 failed, exit 0).
- Query state: `cargo test -p db-pro-ui table_data_query_state::tests --lib` → 5 passed / 0 failed / 0 ignored, exit 0. `cargo fmt --all -- --check` and `git diff --check` passed.
- Native build: `cargo build --release --locked -p db-pro-native --features capture` passed.
- The regression test simulates a successfully applied inline `SELECT`, completes staged-save handling, receives the next runtime command, and verifies it is the same filtered `RunQuery`. Live database insert/save was not repeated; no new user data was written during this verification.
- Behavior: only a successful inline query becomes the active refresh query. Query failure/cancellation does not replace it; table reset clears it. Manual Refresh also preserves the currently executed condition.

## ResponsiveGrid vertical-gap follow-up
- Implementation SHA: `42d78e203b67bde917438ec424a8f9c8a2d206b` (`fix(ui): add vertical responsive grid gaps`).
- Root cause: `ResponsiveGrid` configured only horizontal `item_spacing`; successive rows inherited no intentional vertical separation and metric cards visually touched.
- Fix: the primitive now owns both axes of spacing, neutralizes inherited vertical spacing inside its local scope, and inserts the shared gap between rendered rows. The focused test asserts the exact 8px row gap for a two-row grid.
- Runtime evidence refreshed at 1280×800 and 1440×900; the 1920×1080 wide layout remains one row and therefore has no inter-row gap to render.
- Verification: focused responsive-grid te

… [truncated for memory entry; see source file]


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Calendar & DatePicker intrinsic layout

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-reported Calendar & DatePicker layout defect |
| Task state | Done |
| Baseline SHA | `85e48b45c9a4d59bb882190e0ae2742eab407ea1` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Remove the Calendar frame's gallery-column stretch, keep its surface intrinsically sized, make the seven-column grid responsive to narrow gallery columns, constrain the header to the grid width, clamp the DatePicker popup, and record the current coverage boundary. |
| Out of scope | Keyboard DatePicker navigation, editable text entry, date constraints, localization/week-start configuration, and clear/today actions. |

## 2. Progress checkpoint

- Current HEAD (implementation checkpoint): `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` (source behavior; documentation follows in the next commit)
- Completed acceptance rows: `[x] intrinsic Calendar surface`, `[x] viewport-clamped DatePicker popup`, `[x] responsive seven-column grid`, `[x] fixed three-region header`, `[x] focused layout tests`, `[x] runtime captures at the three requested viewport targets plus narrow regression target`
- Remaining acceptance rows: `[ ] loading/error/empty gallery traversal`, `[ ] keyboard/accessibility interaction evidence`, `[ ] broader DatePicker feature coverage`
- Findings / risks: `P2` — the component remains click-oriented and has no editable or keyboard DatePicker path; this is documented as follow-up in `FINDINGS.md` and was not silently counted as covered.
- Tests already run: `cargo test -p db-pro-ui components::calendar:: --lib` → 10 passed / 0 failed / 0 ignored in the filtered test target, exit 0; `cargo test -p db-pro-ui --lib` → 929 passed / 0 failed / 0 ignored, exit 0; `cargo test --workspace` → exit 0 with the UI segment at 929 passed / 0 failed / 0 ignored.
- Dependency / blocker changes: none. Runtime capture uses the repository's existing `capture` feature; no new dependency was added.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` |
| Commit list | `b8bf057b fix(ui): tighten calendar surface and popup placement`; `335f5012f fix(ui): remove calendar grid width reserve`; `cb1ced7a fix(ui): keep calendar grid visible in narrow gallery`; `d5ab476f fix(ui): constrain calendar header to grid width` |
| File / surface inventory | `crates/ui/src/components/calendar/ui.rs` — intrinsic frame allocation, responsive cell sizing, fixed three-region header, popup clamping, focused layout tests; `crates/ui/src/components/calendar/config.rs` — popup screen margin; `crates/ui/src/component_gallery_inputs.rs` and `crates/ui/src/workspace_actions.rs` — responsive gallery and focused Calendar capture route; `docs/plans/active/core-component-style-system/{CHECKLIST,FINDINGS,VERIFICATION}.md` — scope, findings, gates and evidence; `evidence/calendar-datepicker-*.png` — native captures. |
| Acceptance mapping | Large right padding → `Calendar::show` now allocates a `272px` intrinsic frame instead of letting `Frame::show` inherit the gallery column, with no one-sided grid reserve; narrow clipping → `ResponsiveGrid` reflows the gallery and `calendar_layout` shrinks cells to the local width while preserving all seven columns; header overflow → fixed `prev | title | next` regions share the grid width and zero inter-item spacing; popup edge case → `clamp_popup_to_screen` with the same preferred size and an `8px` screen margin; coverage question → explicit covered/not-covered matrix in `FINDINGS.md`; visual acceptance → committed wide and narrow PNG captures. |
| Commands and counts | `cargo fmt --all -- --check` → exit 0; `cargo check --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo test --workspace` → exit 0 with the UI segment at 929 passed / 0 failed / 0 ignored; `cargo build --release --locked -p db-pro-native` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0; `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → 14 pass / 0 fail / 2 warning categories, exit 0. |
| CI run IDs / status | not run — local main workflow only |
| Known limitations | The host produced 2560×1600 for the 1280×800 target, 2880×1676 for the 1440×900 target, and 3840×1676 for the 1920×1080 target; the latter two are host-height capped. Capture proves the static surface, not keyboard focus or pointer interaction. |
| Migrations / config implications | none; no persisted state, provider behavior, or database contract changed. The capture-only `DB_PRO_CAPTURE_GALLERY_SECTION=calendar` route is test tooling only. |
| Out-of-scope changes | no provider/runtime/database changes; no worktree created; no push performed. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `d5ab476fb8a9c67aca0e3cf89f881dfbf840b33d` |
| Verdict | ACCEPT WITH P2 |
| P0 / P1 / P2 counts | introduced by this SHA: 0 / 0 / 1; inherited: 0 / 0 / 0 |
| Findings | P2 follow-up only: keyboard/accessibility and richer DatePicker feature cases remain outside this focused layout fix. |
| CI disposition | not run; all required local gates listed above passed at the implementation SHA. |
| Next task(s) unblocked | broader DatePicker interaction design and accessibility pass |

## 5. Research / audit handoff

- Source date: 2026-10-01, local repository inspection and native capture.
- Source URLs / references: `crates/ui/src/components/calendar/ui.rs`, `crates/ui/src/components/calendar/handler.rs`, `crates/ui/src/components/calendar/config.rs`, `crates/ui/src/components/common/layout.rs`, `crates/ui/src/component_gallery_inputs.rs`, and `docs/10-egui-native-migration-plan.md`.
- Factual findings: egui `Frame::begin` starts from `available_rect_before_wrap`, so the former Calendar frame inherited the gallery column width; the fixed content width did not constrain the frame's outer paint rect. A fixed three-column gallery could also clip the Calendar when its local width fell below the seven-column grid, and the former flexible header middle could push the next button outside the grid width. The final code wraps the frame in an explicit `allocate_ui_with_layout` region, reflows the gallery, scales cells to local width, fixes the header to three regions, and reuses a preferred size for popup placement.
- Inference: a future full DatePicker should likely share the same semantic size/position helper while adding an explicit keyboard and constraint model; that is a recommendation, not current behavior.
- Decision / recommendation: accept the focused P2 visual fix; schedule the richer interaction model separately instead of expanding this patch.
- Unresolved questions: desired locale and week-start policy; whether direct text entry should replace or complement the click calendar; product rules for min/max and disabled dates.
- Downstream tasks activated: none.

## 6. Tổng kết bằng tiếng Việt

Đã sửa đủ ba lớp lỗi: frame không còn kéo dài theo cột gallery, grid không còn bị clip khi viewport hẹp, và header không còn đẩy nút phải làm phát sinh padding dư. Calendar co cell để vẫn đủ 7 cột; header dùng ba vùng cố định. Test Calendar 10/10, UI 929 passed, workspace exit 0, clippy/build/clean-code đều đạt. Các case keyboard, nhập text, min/max, disabled date, locale và clear/today vẫn là P2 follow-up, chưa coi là đã cover.

## 7. Component Gallery surface follow-up

| Field | Value |
|---|---|
| Exact SHA | `8ca9cac5470ec6688c52aff4cc7e09a0aacdad16` |
| Task state | Done |
| Scope | Recompose Cards & Metric Displays for responsive density, restore metric-label contrast through `DbProTheme`, and capture the light surface at 1280×800, 1440×900, and 1920×1080. |
| Runtime result | 2×2 at medium widths, four columns when the content region is wide enough, and one-column fallback below the minimum two-card width; no orphan third-row card at the supported gallery widths. |
| Evidence | `evidence/cards-metrics-1280x800.png`, `evidence/cards-metrics-1440x900.png`, `evidence/cards-metrics-1920x1080.png`. |
| Gates | fmt, workspace check, clippy, workspace test (929 UI tests), release build, clean-code scan, and diff check all passed after the implementation SHA. |
| P0 / P1 / P2 | 0 / 0 / 1; loading/error/empty traversal remains a documented P2 follow-up for the static gallery surface. |
| Workflow | Implemented directly on local `main`; no worktree and no push. |

### Tổng kết bằng tiếng Việt

Cards & Metric Displays đã được đưa về layout responsive có chủ đích: vùng trung bình giữ 2×2, vùng đủ rộng mới dùng 4 cột, nhãn metric dùng token semantic dễ đọc hơn. Đã capture light theme ở cả ba kích thước và chạy đủ gate Rust/UI; loading/error/empty vẫn là follow-up vì gallery hiện là surface tĩnh.

## 8. ResponsiveGrid vertical-gap follow-up

| Field | Value |
|---|---|
| Exact SHA | `42d78e203b67bde917438ec424a8f9c8a2d206b` |
| Task state | Done |
| Scope | Add intentional vertical gaps between wrapped `ResponsiveGrid` rows while preserving the existing horizontal token and caller spacing. |
| Root cause | The primitive set `item_spacing.x` only; the second metric row therefore had no explicit vertical rhythm. |
| Fix / coverage | `ResponsiveGrid` now scopes vertical spacing, inserts the configured gap between rows, and has focused coverage asserting the exact row separation. |
| Runtime evidence | Refreshed `evidence/cards-metrics-1280x800.png` and `evidence/cards-metrics-1440x900.png`; the 1920×1080 four-column capture has one row. |
| Gates | Focused test, workspace tests (929 UI tests), fmt, check, clippy, release build, clean-code scan, and diff check passed. |
| Workflow | Implemented directly on local `main`; no worktree and no push yet at this evidence checkpoint. |

### Tổng kết bằng tiếng Việt

Đã sửa đúng lỗi spacing: `ResponsiveGrid` trước đó chỉ có gap ngang, nên các row dọc bị dính. Primitive giờ dùng cùng gap token cho cả hai chiều, có test xác nhận khoảng cách 8px và capture 1280/1440 đã thấy khoảng thở giữa hai hàng.

## 9. Data Grid pagination footer follow-up

### 1. Claim

| Field | Value |
|---|---|
| Agent identity | codex · native UI polish lane |
| Issue(s) | n/a — user-requested Data Grid layout correction |
| Task state | Done |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` (implementation remains uncommitted in the working tree) |
| Branch / PR | `main` / no PR |
| Scope interpretation | Move Data Grid pagination to the bottom result-grid footer and place Save, Discard, Refresh at its left. Keep Add Row, filter, and sort controls in the top toolbar. |
| Out of scope | Loading/error/empty placeholder design and unrelated prior UI changes. |

### 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`; it is the baseline, not an immutable representation of the uncommitted implementation.
- Completed acceptance rows: `[x] footer actions and pagination moved below the grid`, `[x] staged-change safety retained`, `[x] lesson added to CHECKLIST.md`, `[x] normal-state runtime captures at all three requested width targets`.
- Remaining acceptance rows: `[ ] loading/error/empty state captures`.
- Findings / risks: `P2` — non-result placeholders do not show this footer; the three 1440/1920 captures are host-height capped and document only the normal state.
- Tests already run: no tests run (not requested). `cargo fmt --all -- --check`, `git diff --check`, and release capture-feature build passed (exit 0).
- Dependency / blocker changes: none.

### 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | No implementation commit; working-tree diff is based on `710ba002612c6d71ef2605f99f2a49743b51c3a4`. |
| Commit list | none |
| File / surface inventory | `crates/ui/src/table_data_toolbar_surface_view.rs` — split top toolbar from footer

… [truncated for memory entry; see source file]

