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

- [x] Keep the native window maximized by default with a safe inner-size fallback.
- [x] Paint the ER empty state with the same grid language as the populated canvas.
- [x] Wrap Agent context badges instead of clipping them in a horizontal strip.
- [x] Center the Transfers and Monitor placeholder badge at its intrinsic width.
- [x] Re-run Rust quality gates after Wave 2 edits.
- [x] Re-capture native runtime evidence for ER, Agent, Transfers and Monitor.
