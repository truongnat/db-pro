# Checklist

## Slice 1 — Query workspace

- [x] Long file path/title uses truncation and a full-content tooltip.
- [x] Connection/schema selectors remain reachable at 1280x800 and below (fixed combo widths +
    char-elided labels + full-name hover tooltips).
- [x] Tab strip has bounded title rendering and full-title tooltips for overflow.
- [x] Add regression tests for pure sizing/truncation helpers (`elide_chars` tests in
    `crates/ui/src/query_helpers.rs`).

## Slice 2 — Sidebar and shell

- [x] Long connection/workspace names use truncation and a full-name tooltip.
- [x] Sidebar controls keep primary actions visible at minimum width (action buttons render
    right-aligned in a reserved `right_to_left` slot; the selector label truncates).
- [x] Activity/sidebar resize handle respects minimum and maximum widths
    (`SIDEBAR_MIN_WIDTH`/`SIDEBAR_MAX_WIDTH` clamp both drag and restored sessions).

## Slice 3 — Dialogs and forms

- [x] Dialog and sheet widths clamp to the available viewport.
- [x] Dialog/sheet titles truncate with a full-title tooltip while keeping close actions reachable.
- [x] Long errors and translated/large-font content do not clip (dialog body is capped and scrolls).
- [x] Dialog actions remain reachable when content grows (footer actions live inside the
    height-capped scrollable card).
- [ ] Form fields have explicit min/max width behavior.

## Slice 4 — Data and schema surfaces

- [x] Select popups clamp horizontally to the viewport and truncate long options.
- [x] Grid/output owns scrolling without accidental nested traps (output panes use bounded
    `ScrollArea::max_height`; the result grid virtualizes its own scroll).
- [ ] Long cell/error/definition content has a deliberate wrap or truncate policy.
- [ ] ER labels and controls remain usable in dense mode.

## Slice 5 — Verification

- [x] Run targeted UI tests.
- [x] Run Rust quality gates.
- [ ] Capture runtime evidence at 1024x700, 1280x800, 1440x900, 1920x1080 and maximized/fullscreen.
- [x] Record normal and worst-case fixtures in `VERIFICATION.md`.
