# Checklist

## Slice 1 — Query workspace

- [x] Long file path/title uses truncation and a full-content tooltip.
- [ ] Connection/schema selectors remain reachable at 1280x800 and below.
- [x] Tab strip has bounded title rendering and full-title tooltips for overflow.
- [ ] Add regression tests for pure sizing/truncation helpers.

## Slice 2 — Sidebar and shell

- [x] Long connection/workspace names use truncation and a full-name tooltip.
- [ ] Sidebar controls keep primary actions visible at minimum width.
- [ ] Activity/sidebar resize handle respects minimum and maximum widths.

## Slice 3 — Dialogs and forms

- [x] Dialog and sheet widths clamp to the available viewport.
- [x] Dialog/sheet titles truncate with a full-title tooltip while keeping close actions reachable.
- [ ] Long errors and translated/large-font content do not clip.
- [ ] Dialog actions remain reachable when content grows.
- [ ] Form fields have explicit min/max width behavior.

## Slice 4 — Data and schema surfaces

- [x] Select popups clamp horizontally to the viewport and truncate long options.
- [ ] Grid/output owns scrolling without accidental nested traps.
- [ ] Long cell/error/definition content has a deliberate wrap or truncate policy.
- [ ] ER labels and controls remain usable in dense mode.

## Slice 5 — Verification

- [x] Run targeted UI tests.
- [x] Run Rust quality gates.
- [ ] Capture runtime evidence at 1024x700, 1280x800, 1440x900, 1920x1080 and maximized/fullscreen.
- [x] Record normal and worst-case fixtures in `VERIFICATION.md`.
