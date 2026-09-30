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

## Tổng kết bằng tiếng Việt
Đã làm lại đúng cụm bị phản hồi xấu: tách thành 3 panel có nhịp rõ ràng, rút gọn copy, hiển thị calendar inline, sửa lỗi icon Accordion đè lên title, bỏ Card lồng Card. Đã có evidence 1280×800, 1440×900 và 1920×1080; trạng thái loading/error/empty của gallery vẫn pending.
