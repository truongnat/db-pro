# Native Visual Redesign — Checklist

## Wave 1 — shell composition

- [x] Trace latest native commits to completed `native-ui-foundation` / `native-ide-redesign`.
- [x] Record baseline screenshot and identify structural visual failures from `goal-2.md`.
- [x] Establish dark-first v4 theme tokens and reset stale persisted light preference.
- [x] Rebuild compact native application header and workspace tab strip.
- [x] Recompose activity rail and Explorer as a calm tree/workbench surface.
- [x] Recompose welcome/empty workspace without changing runtime behavior.
- [x] Add focused native regression tests for shell/theme behavior.
- [x] Run Rust quality gates for the native workspace.
- [x] Capture fresh native screenshots at 1280×800 and 1440×900.

## Completion gates

- [ ] P0 = 0
- [ ] P1 = 0
- [ ] Existing PostgreSQL/SQLite runtime contracts remain unchanged
- [ ] Runtime evidence is recorded in `VERIFICATION.md`
- [ ] External review findings are resolved or explicitly deferred

## Wave 2 — productive surface polish

- [x] Keep the native window maximized by default, including after persisted eframe frame restore,
  with a safe inner-size fallback.
- [x] Paint the ER empty state with the same grid language as the populated canvas.
- [x] Wrap Agent context badges instead of clipping them in a horizontal strip.
- [x] Center the Transfers and Monitor placeholder badge at its intrinsic width.
- [x] Re-run Rust quality gates after Wave 2 edits.
- [x] Re-capture native runtime evidence for ER, Agent, Transfers and Monitor.

## Wave 3 — full-window data surfaces

- [x] Center native connection, delete and Insert row dialogs against the maximized workspace.
- [x] Center the welcome content using a width-aware horizontal layout.
- [x] Make shared result-grid columns fill the available maximized viewport by default.
- [x] Preserve manual column resizing and narrow-window horizontal overflow behavior.
- [x] Reserve a stable grid viewport for sparse result sets.
- [x] Remove implementation-only virtualization wording from the result-grid status copy.
- [x] Verify the SQLite connection → schema → Data Editor → Query Results runtime path.
- [x] Capture fresh full-window screenshots for the Wave 3 surfaces.
- [x] Run the full Rust quality gates and clean-code diff scan after Wave 3 edits.
