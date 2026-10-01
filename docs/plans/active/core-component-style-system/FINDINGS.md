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

## Implementation result
- The dark theme now uses a charcoal/navy surface ladder (`#17191c` → `#23272d`) with crisp border steps and a restrained blue accent (`#4f8cff`).
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
