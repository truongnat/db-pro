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
- Focused layout tests: `calendar_frame_size_is_intrinsic_and_includes_symmetric_margin`, `calendar_layout_shrinks_cells_when_the_gallery_column_is_narrow`, and `calendar_layout_keeps_intrinsic_cells_when_space_is_available`. Runtime captures include the narrow regression target `evidence/calendar-datepicker-800x800.png` plus `evidence/calendar-datepicker-1280x800.png`, `evidence/calendar-datepicker-1440x900.png`, and `evidence/calendar-datepicker-1920x1080.png`.

## Tổng kết bằng tiếng Việt
Đã làm lại đúng cụm bị phản hồi xấu: tách thành 3 panel có nhịp rõ ràng, rút gọn copy, hiển thị calendar inline, sửa lỗi icon Accordion đè lên title, bỏ Card lồng Card. Đã có evidence 1280×800, 1440×900 và 1920×1080; trạng thái loading/error/empty của gallery vẫn pending.

## WHERE input geometry correction — 2026-10-06
- P2: the prefix was a focusable action button, the editor frame inflated height independently of the 28px small buttons, and the input/run separation was too small.
- Fix: passive themed WHERE prefix inside a fixed-height editor using shared ButtonSize::Sm dimensions; editable predicate immediately follows it. Reserve Run width using the remaining wrapped-row rectangle, with SPACE_SM separation.
- Source identity: baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted toolbar changes. No SQL/provider behavior change.

## Empty WHERE executes unfiltered SELECT — 2026-10-06
- P2: empty/whitespace draft disabled Run and bypassed Enter; the fixed SQL prefix included WHERE even without a predicate.
- Fix: quote the table/schema in an unconditional SELECT prefix; append WHERE only for a nonempty trimmed predicate. Empty Run/Enter follows the same guarded inline execution path as filtered SQL. Existing configured result row limit remains enforced.
- Source identity: baseline `710ba002612c6d71ef2605f99f2a49743b51c3a4` plus uncommitted toolbar/view changes.
