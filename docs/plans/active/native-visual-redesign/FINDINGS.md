# Native Visual Redesign — Findings

## Baseline finding — structural visual mismatch (UX-P1)

### Evidence

- Baseline native screenshot: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7debe2b1-2127-4613-8c73-6b0d88ef3254-screenshot.png`.
- The baseline is light-first despite the native theme code having dark tokens.
- The topbar, workspace tabs, connection row and table controls are each rendered as separate
  outlined/filled controls, so the workbench hierarchy is weak.
- The Explorer keeps `Edit`, delete and `New connection` visible as routine buttons.
- The current frame therefore fails the explicit `goal-2.md` requirement that the redesign be
  immediately obvious before interaction.

### Failure scenario

Launch the native binary with an existing light theme preference and open a table. The first
frame still reads as the previous prototype: pale surfaces, button-heavy chrome, and a wide
sidebar consuming workspace area. A small token or spacing adjustment would not satisfy the
redesign requirement.

### Scope decision

Wave 1 changes only the native presentation layer and theme persistence version. Runtime state,
database commands, DTOs and provider behavior remain out of scope.

## Open risks

- egui does not expose the same accessibility tree as the React/Tauri surface; visual runtime
  screenshots are required for the acceptance decision.
- Native text input, clipboard, DPI scaling and close/reopen behavior need separate runtime
  evidence after the structural shell is in place.
- Existing persisted eframe state can hide the new default until the theme storage version is
  bumped and tested.

## Wave 1 evidence

- Native screenshot at 1280×800 content: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/bc48a216-856d-4464-80a3-d1290fa72785-screenshot.png`.
- Native screenshot at 1440×900 content: `/var/folders/bh/lc9yszwj2vg5gpqn_8g5n60h0000gn/T/orca-computer-use/7676317c-9b76-4b65-a7a5-dd5898392cb5-screenshot.png`.
- Fresh launch after the native default-maximized change reported a top-aligned 1280×832
  window in the macOS harness, with the 1280×800 content viewport intact.

The 1280×832 result was traced to eframe restoring its persisted non-maximized frame after the
initial viewport request. The native entrypoint now sends `ViewportCommand::Maximized(true)` after
that restore. A fresh launch of the rebuilt binary reported a full-monitor 2473×1409 window.

## Wave 2 audit findings

### P2 — ER empty canvas lost the graph-grid language

The populated ER canvas paints a grid, but the empty/search states rendered only a flat editor
surface. This made the empty state look disconnected from the relationship-map workspace. The
smallest fix is presentation-only: paint the same grid behind the existing empty-state copy.

### P2 — Agent context badges clipped in narrow panels

The Agent context row used a horizontal scroll region. At the runtime panel width, the screenshot
showed the database/provider/table badges continuing past the visible edge, so the active context
was not fully legible. Wrapping the badges preserves all information without changing Agent
behavior.

### P2 — activity placeholders looked like unfinished full-width stripes

Transfers and Monitor intentionally have no provider implementation yet, but their `COMING SOON`
badge expanded across the centered column in the native screenshot. Centering the intrinsic badge
keeps the state honest while making it read as a compact status marker.
