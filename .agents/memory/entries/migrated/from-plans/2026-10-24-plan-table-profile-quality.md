---
schema_version: 1
id: "MEM-PLAN-table-profile-quality-3cd756394df2"
title: "Plan archive: Table Profile Quality"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, table-profile-quality]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3cd756394df25ead17561dbf391906198e5a8db47deb031ad29705f9582a4592"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/table-profile-quality/PLAN.md"
  - "docs/plans/active/table-profile-quality/FINDINGS.md"
  - "docs/plans/active/table-profile-quality/VERIFICATION.md"
  - "docs/plans/active/table-profile-quality/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/table-profile-quality/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Table Profile Quality — Plan

State: RUNTIME_VERIFY
Branch: `main` (repository owner requires direct-main workflow)

## Goal

Make the Table Detail Profile tab's sample summaries truthful, readable, and compact at narrow and wide window sizes.

## Evidence / baseline

- Current implementation: `crates/ui/src/table_profile_surface_view.rs` computes summary values from the loaded `UiQueryResult` and renders an eight-column egui grid.
- Observed screenshot: the `created_at` sample is labeled `Enum-like`; Min and Max show different timestamp formats; the grid's intrinsic column widths leave large gaps.
- Source failure cases: numeric extrema use string ordering; a result with columns but zero rows is rendered as `100% full` / `Constant` instead of an empty state; the grid only scrolls vertically.
- Existing coverage: `test_profile_result_numeric_and_uniqueness` exercises basic numeric summaries and distinct counts.

## Scope

- Use exact decimal ordering for numeric Min/Max without converting through `f64`.
- Format recognized date/time extrema consistently while preserving any displayed offset.
- Replace inferred `Enum-like` / `Standard` quality labels with neutral patterns explicitly based on the loaded sample.
- Render a dedicated no-rows state and make the profile grid scroll in both directions.
- Reduce unnecessary grid spacing and bound long cell values with the full value available on hover.

## Non-goals

- Change query paging, provider result decoding, or the scope of the loaded sample.
- Change average/sum precision or their existing `f64` representation.
- Add a new chart, new database request, or provider-specific behavior.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes; consumes the existing UI result | N/A for this presentation-only change; live provider UI not exercised |
| SQLite | yes; consumes the existing UI result | N/A for this presentation-only change; live provider UI not exercised |

## Architecture

The Profile pane remains a presentation-only consumer of `UiQueryResult` in `crates/ui`. It does not issue a `UiCommand` or alter provider/runtime behavior. Numeric ordering uses the existing `bigdecimal` dependency; date/time display uses the existing `chrono` dependency.

## Acceptance criteria

- [x] Numeric extrema compare as exact decimals (`2` precedes `10`) and preserve source text.
- [x] Recognized date/time extrema use one readable display format; timezone offsets remain visible.
- [x] Pattern labels do not infer enum semantics from a small distinct count.
- [x] A loaded result with columns and zero rows presents a clear empty state.
- [x] Long values remain discoverable, and both horizontal and vertical grid scrolling are available.
- [x] Existing table/header theme tokens remain the only color source.
- [x] Runtime capture and automated verification status are reported honestly; visual matrix and tests remain pending.

## Owner follow-up scope — 2026-10-06
Reuse shared Table for width/clip behavior, measure actual content widths for complete numeric cells, preserve horizontal overflow, and account for Structure badges. Shared filter header/modal removal for Indexes/Structure/Foreign Keys is tracked in the existing Table Indexes Responsive Layout plan. Full UI release gate remains failed by two connection-form tests with baseline attribution unverified.


## Source: `FINDINGS.md`

# Table Profile Quality — Findings

## P2 — Pattern label overstates what a page sample proves

- Evidence: the prior screenshot marks `created_at` as `Enum-like` based only on four distinct values in six loaded rows.
- Root cause: the renderer infers a category from a small distinct-count threshold, regardless of type or sample size.
- Decision: show neutral sample patterns (`Unique`, `Constant`, `Repeated`, `Varied`) and rename the column to `Pattern`.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Timestamp values have inconsistent presentation

- Evidence: the prior screenshot shows a SQL-style timestamp for Min and an RFC3339 value for Max in the same column.
- Root cause: raw cell strings are rendered unchanged.
- Decision: normalize recognized date/time strings for display and retain offsets; leave unrecognized values intact.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Numeric extrema are ordered lexicographically

- Evidence: source at baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`, `table_profile_surface_view.rs:87-90`, selected Min/Max with string ordering.
- Failure example: values `2` and `10` can produce Min `10` and Max `2`.
- Decision: compare finite numeric strings using `BigDecimal`, then keep the original value for display. If a numeric cell cannot be parsed as a decimal, omit extrema rather than silently reverting to string ordering.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; automated regression coverage pending.

## P2 — Empty sample presents false completeness/pattern signals

- Evidence: a present result with columns and no rows reaches the grid; null-rate arithmetic treats it as zero nulls.
- Decision: show a dedicated no-rows state before profiling.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 native fixture capture reviewed.

## P2 — Long cell values can push columns apart or beyond the viewport

- Evidence: the screenshot's widest sample values expand the Min/Max grid columns; the grid only scrolls vertically.
- Decision: add a padded surface, group column name/type and Min/Max values, collapse Avg/Sum when the loaded sample has no numeric summaries, and ellipsize long values with full-value hover disclosure.
- Disposition: addressed in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`; 1280×800 capture shows the last column filling available width. Long values still ellipsize by actual cell width; hover behavior and 1440×900 / 1920×1080 captures remain pending.

## Independent review — 2026-10-04

### P2 (inherited) — An all-null column is labeled “Varied”

- Evidence: in working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`, `table_profile_surface_view.rs:291-302` falls through to `Varied` when `non_null_count == 0` and `distinct_count == 0`. For a result containing rows whose values in this column are all NULL, the label says values vary although there are no non-null values to compare. At baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`, the same data was called `Enum-like`; this is a pre-existing semantic gap, not a regression from the new pattern vocabulary.
- Recommendation: show a neutral state such as `All null` / `No values` for this case; add a fixture assertion for all-null and mixed-null columns.
- Status: fixed in current working-tree blob `e9c42b45cb03921c6c4630b5defb51bd6075dbbd`: zero non-null values now render the neutral `All null` badge, covered by `all_null_column_has_a_distinct_pattern_label`. The available runtime screenshot still has no all-null column.

### Review coverage gap — typography/hover and viewport matrix remain unverified

- Evidence: `evidence/profile-light-1280x800.png` is a loaded, light-theme, 1280×800 logical capture at 2× physical scale. The `created_at` type (`timestamp wit…`) and UUID range visibly truncate; source lines `199-204` and `281-283` provide hover disclosure, but the tooltip was not exercised. No 1440×900, 1920×1080, constrained-width, empty/loading, dark-theme, or scale-factor 1.0/1.25/1.5 capture is available.
- The screenshot shows clearer panel padding and hierarchy than the supplied original and keeps the range column inside the panel. It supports only this captured state and scale.
- Disposition: do not mark the visual/runtime gate complete until the required evidence is collected or explicitly marked unsupported/not run.

## Profile width / Structure badge clipping — 2026-10-06
- Baseline HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus existing uncommitted changes.
- P2 Profile uses intrinsic-width egui Grid columns and leaves unused space on wide screens. Reuse shared Table fixed-column scaling; horizontal overflow scrolls on narrow screens, and existing full-value hover disclosure remains.
- P2 shared Table clips exactly to the padded content rectangle; centered 1px badge strokes extend 0.5px beyond their allocation and lose the leading border. Expand the clip by 1px into padding, bounded by the actual cell rectangle. The fix applies to Structure Key/Nullable and other shared table badges without disabling clipping.
- No query/provider calculation changes or new dependencies. Added deterministic Structure capture route solely for visual verification.

## Dynamic content widths — 2026-10-06
- Owner rejected percentage/minimum widths that still truncated Avg/Sum. Profile now measures headings and rendered values with the same egui font styles. Shared TableColumn::content_width includes the Table cell padding; the existing layout grows columns proportionally only when spare width exists and preserves measured widths when scrolling is required.
- Numeric summary no longer truncates to 20 characters; range and summary labels extend within measured cells. Full-range values, percentages/counts and long numeric summary determine minimum widths.
- Added actual render regression for a long numeric summary, asserting complete text and containment by the cell clip; added Profile-only numeric capture fixture to expose the sixth column.


## Source: `VERIFICATION.md`

# Table Profile Quality — Verification

## Baseline

- Source baseline SHA: `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Implemented Profile source blob: `1208c1372e23abfa151cafffc6f03965bd7ee024` (uncommitted working tree).
- The Profile surface source was clean at baseline; the surrounding table workspace has unrelated working-tree changes and is not modified by this task.
- User-provided visual evidence: screenshot of Table Detail → Profile, 2026-10-04; single wide viewport.

## Current work

- Implementation: visible padded profile card, grouped columns, honest completeness labels, conditional numeric summary, and a final Grid column that fills remaining width are present in the working tree. Range labels truncate to actual cell width and retain full values on hover. Plan remains `IMPLEMENTING` pending broader viewport/state review and regression verification.
- `cargo check --locked -p db-pro-ui`: PASS after correcting an initial compile error in the new pattern badge token reference.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS (exit 0).
- Tests: not run, per task scope.
- Native runtime: deterministic sample fixture capture at 1280×800 completed with exit 0 and reviewed at `evidence/profile-light-1280x800.png` (2560×1600 physical pixels, 2× scale).
- Native runtime: 1440×900 capture attempt was stopped after the capture process stalled (exit 130); 1920×1080 and empty/loading/error states not captured.
- PostgreSQL/SQLite live UI: not run; this change consumes the existing result and issues no database operation.

## Limitations

- The supplied screenshot verifies the wide loaded-data state only. The new deterministic fixture verifies the loaded state at 1280×800; it does not verify 1440×900, 1920×1080, empty/loading/error states, or hover disclosure.
- `avg` and `sum` retain their existing `f64` calculation and precision limitations; changing those outputs is outside this focused pass. Mixed numeric/text columns omit Min/Max rather than applying an ambiguous comparison.

## Profile/Structure layout and dynamic numeric widths — 2026-10-06
- Baseline SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`; uncommitted source blob identities: profile `392f66a7c74e04f0cdaa7e9200bca881e7ec4659`; shared table UI `86c181a99bf15fa86a24791bc44384afad8c877b`.
- `cargo test --release --locked -p db-pro-ui --lib`: FAILED overall, 992 passed / 2 failed / 0 ignored (exit 101). Long numeric rendering, complete badge border and passive index-name regressions passed. Failures: `explicit_disable_selection_is_preserved_on_submit`, `new_postgresql_connection_defaults_to_tls_require`, both report missing create command in app_tests. Baseline attribution is unverified; these connection-form paths are outside this patch and were not changed to make this layout task green.
- Initial debug test builds were interrupted after remaining stalled at rustc; a debug-free attempt was also interrupted. Release tests above provide the actual test results. Initial capture builds failed from an incorrect feature gate / TableView variant; both were corrected before final release build.
- `cargo fmt --all -- --check`, explicit rustfmt check for the included Profile source, and `git diff --check`: PASS. Workspace check/clippy/full workspace tests/benchmarks were not run; no performance improvement claimed.
- Earlier inspected fixture captures: `profile-width-*` and `structure-key-*` at requested 1280×800, 1440×900, 1920×1080. They prove the shared-table width and badge-border change; later numeric/header follow-up screenshots are recorded separately. Larger targets cap at 838 logical pixels high.
- Provider impact: PostgreSQL/SQLite data and commands unchanged; screenshots are deterministic native fixtures, not live-provider evidence. Loading/error/empty-result, horizontal scroll interaction and VoiceOver remain pending.
- Reusable lesson: measure full formatted numeric content with its actual font, include shared cell padding, and preserve that minimum when scrolling is required. Do not truncate number strings or guess percentage widths. Table clips must allow stroked badges to paint into padding without leaking into adjacent cells. Recorded in this plan; no global memory write.

## Final code gates — 2026-10-06
- Final native command: `cargo build --release --locked -p db-pro-native --features capture` → PASS, exit 0.
- Final Profile test command: `cargo test --release --locked -p db-pro-ui --lib table_profile_surface_view` → 3 passed / 0 failed / 0 ignored, exit 0.
- Final Profile blob `0c38004312fbcffa6bfa66b3bd5e9408e8b2e555`; shared header blob `b30acdf40773676f337676503e4bdca111917061`, against HEAD `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted patch. Horizontal Profile overflow keeps measured widths and locates its scrollbar immediately after the rows. Header input border uses semantic theme colors even when idle.

## Final native fixture evidence — 2026-10-06
- `evidence/profile-dynamic-numeric-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/profile-dynamic-numeric-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/profile-dynamic-numeric-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1440x900.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1280x800.png`: final-code native framebuffer, PNG integrity verified.
- `evidence/structure-header-final-1920x1080.png`: final-code native framebuffer, PNG integrity verified.
- Requested widths 1280/1440/1920 produced physical images 2560×1600, 2880×1676 and 3840×1676; larger heights capped to 838 logical pixels. Selected 1280 and 1920 captures inspected: input border/font/alignment, full long numeric values in wide layout and horizontal overflow in narrow layout. Static fixtures only; scrolling/key/pointer/provider interactions not exercised. The Foreign Keys fixture exposes an inherited narrow Action-cell wrap/clip limitation; this header task does not modify it.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/03-data-grid-table-editor.md` verdict: **EVIDENCE_GAP**
(implementation verified in code; loading/error/empty + interactive evidence still
open). Confirmed at HEAD: BigDecimal extrema ordering, `profile_text_width` +
`TableColumn::content_width`, horizontal scroll, sample-scoped labels, dedicated
empty states, theme-only colors; the 3 profile render tests pass
(`cargo test -p db-pro-ui --lib profile`).

**Stale-fact correction:** the "992 passed / 2 failed" suite totals above are stale —
`explicit_disable_selection_is_preserved_on_submit` and
`new_postgresql_connection_defaults_to_tls_require` **both PASS at HEAD** (audit
re-ran them); they are no longer baseline failures to attribute.

Fresh captures: `docs/plans/_audit-2026-10-13/captures/table-profile_*.png` and
`table-structure_*.png` at 3 sizes (heights capped at 838 for 1440/1920). Remaining:
loading/error/empty capture matrix, hover-disclosure + h-scroll interaction,
VoiceOver pass. State stays RUNTIME_VERIFY.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — Table Profile Quality

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI implementation lane |
| Issue(s) | n/a |
| Task state | In Progress |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | `main` / no PR |
| Scope interpretation | Correct the Profile tab's misleading sample presentation and improve its grid layout. |
| Out of scope | Provider/runtime changes, query paging, average/sum precision, and unrelated working-tree changes. |

## 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4` (implementation is uncommitted).
- Completed acceptance rows: exact decimal Min/Max; normalized date/time display; sample-scoped pattern labels; empty-sample state; two-axis scrolling and bounded values; theme-token use.
- Remaining acceptance rows: regression tests and native runtime review at the remaining sizes/states.
- Findings / risks: five screenshot/source P2 findings are addressed in the working tree; 1280×800 visual fixture capture reviewed; broader runtime matrix and regression revalidation remain pending.
- Tests already run: no tests; `cargo check --locked -p db-pro-ui` and `cargo build --release --locked -p db-pro-native --features capture` passed.
- Dependency / blocker changes: none; existing `bigdecimal` and `chrono` dependencies are available.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` baseline; working-tree changes uncommitted |
| Commit list | none |
| File / surface inventory | `crates/ui/src/table_profile_surface_view.rs` (working-tree blob `1208c1372e23abfa151cafffc6f03965bd7ee024`) — Profile summaries and grid; this evidence directory — task plan, handoff, and 1280×800 capture; `docs/plans/STATUS.md` — active plan row |
| Acceptance mapping | See `PLAN.md` acceptance criteria and `CHECKLIST.md`. |
| Commands and counts | `cargo check --locked -p db-pro-ui` → exit 0; `cargo build --release --locked -p db-pro-native --features capture` → exit 0; capture at 1280×800 → exit 0; 1440×900 capture stopped after stall → exit 130; no tests run. |
| CI run IDs / status | not run |
| Known limitations | Only the loaded-state 1280×800 fixture capture was reviewed; average/sum remain `f64`; malformed or mixed numeric values omit Min/Max. |
| Migrations / config implications | none |
| Out-of-scope changes | All pre-existing user working-tree changes remain untouched. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `1208c1372e23abfa151cafffc6f03965bd7ee024` working-tree blob; repository baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Verdict | `BLOCK` — the all-null label is corrected and covered by a focused test; required visual/runtime coverage remains incomplete. |
| P0 / P1 / P2 counts | introduced: 0 / 0 / 0; inherited: 0 / 0 / 0 open. Runtime coverage gap is recorded separately. |
| Findings | `FINDINGS.md` — all-null profile fix is covered by a focused test; hover behavior, typography across scales, and wider/state captures remain unverified. |
| CI disposition | not run |
| Next task(s) unblocked | collect remaining viewport/state/scale evidence. |

### Fix recheck — 2026-10-04

- Source blob: `crates/ui/src/table_profile_surface_view.rs` at `e9c42b45cb03921c6c4630b5defb51bd6075dbbd`.
- Change: columns with zero non-null values now show a neutral `All null` badge.
- Verification: `cargo test --locked -p db-pro-ui all_null_column_has_a_distinct_pattern_label` passed.
- Remaining gate: hover behavior, 1440×900, 1920×1080, constrained width, empty/loading/error, dark theme, and font scale captures.

## 5. Research / audit handoff

- Source date: 2026-10-04.
- Source references: user-provided Table Detail → Profile screenshot; `crates/ui/src/table_profile_surface_view.rs`; `crates/ui/src/components/legacy.rs`; `crates/ui/src/theme.rs`; `docs/architecture/system-overview.md`.
- Factual findings: page stats are derived from loaded `UiQueryResult`; screenshot shows the date pattern badge and mixed timestamp presentation.
- Inference: the wide gaps are caused by intrinsic grid sizing around long cell contents.
- Decision / recommendation: preserve the compact grid and bound its long-value presentation; do not infer enum semantics from distinct count alone.
- Unresolved questions: 1440×900 capture stalled; 1920×1080 and empty/loading/error states and runtime hover behavior remain unverified.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã nâng cấp Profile với card có padding rõ, nhóm tên/type và Min/Max, completeness đúng nghĩa, và chỉ hiện Avg/Sum khi mẫu có cột số. Grid không giới hạn 300pt nữa nên cột cuối lấp khoảng trống còn lại; Range tự cắt theo độ rộng ô. Capture native fixture 1280×800 đã được xem; check và release build đạt. Chưa chạy test; capture 1440×900 bị treo, các kích thước/trạng thái còn lại và hover chưa xác minh. Các thay đổi có sẵn khác trong working tree được giữ nguyên.

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

