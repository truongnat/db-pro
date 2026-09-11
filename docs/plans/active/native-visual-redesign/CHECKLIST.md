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

## Wave 4 — context-aware status bar

- [x] Scope line/column and encoding metadata to the Query workspace.
- [x] Show context labels for non-query workspaces instead of SQL editor metadata.
- [x] Add a native regression test for the editor-status visibility invariant.
- [x] Capture fresh native runtime evidence for the Data Editor status bar.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 4 edits.

## Wave 5 — full ER canvas

- [x] Make the ER canvas fill the available native viewport before content overflow.
- [x] Preserve scroll behavior for diagrams larger than the viewport.
- [x] Add a regression test for viewport fill and content overflow sizing.
- [x] Capture fresh native runtime evidence for the full ER canvas.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 5 edits.

## Wave 6 — live query cursor status

- [x] Read line and column from the active egui SQL cursor.
- [x] Reset cursor metadata when query documents change.
- [x] Add a regression test for cursor metadata reset.
- [x] Capture native runtime evidence with a non-default cursor position.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 6 edits.

## Wave 7 — keyboard grid focus

- [x] Add Arrow/Home/End navigation for the visible filtered/sorted row projection.
- [x] Keep filter/editor text inputs isolated from grid navigation.
- [x] Distinguish the active cell from the selected row in the grid treatment.
- [x] Add regression coverage for row identity and boundary navigation.
- [x] Capture native SQLite runtime evidence for cross-row and cross-column movement.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 7 edits.

## Wave 8 — ER search mode recovery

- [x] Prove the large-schema “Show all” state made the visible search field inert.
- [x] Make non-empty search input return the ER diagram to focused mode.
- [x] Add a compact “Focus search” action for explicit recovery from “Show all”.
- [x] Add regression coverage for large-schema search/show-all state transitions.
- [x] Capture native SQLite runtime evidence with a 202-table fixture and a focused search result.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 8 edits.

## Wave 9 — staged grid interaction correctness

- [x] Prove Enter left the active Data Editor cell uncommitted and selection changes left a stale editor visible.
- [x] Commit the active cell on Enter and before row/cell selection changes.
- [x] Copy staged Data Editor values while preserving raw Query Results values.
- [x] Add regression coverage for provider-neutral copy behavior.
- [x] Capture native SQLite runtime evidence for Enter, selection transition and clipboard payload.
- [x] Re-run the full Rust quality gates and clean-code diff scan after Wave 9 edits.

## Wave 10 — Codex visual parity

- [x] Record the installed Codex desktop light/dark token reference from the local app bundle.
- [x] Calibrate the shared native theme colors, icons, typography, spacing, radii and interaction states.
- [ ] Verify every native workspace surface in both appearance modes.
- [ ] Record intentional database-IDE deviations and close the visual P1 baseline finding.

## Wave 11 — metadata empty-state composition

- [x] Prove empty metadata cards collapsed to content width in the native SQLite runtime.
- [x] Add a shared Lucide/Codex empty-state component for metadata workspaces.
- [x] Make metadata cards span the available native workspace width.
- [x] Track explicit constraint presence independently from the empty-state condition.
- [x] Preserve populated metadata rows and provider-neutral introspection behavior.
- [x] Capture dark and light SQLite runtime evidence for the changed metadata surfaces.
- [x] Re-run Rust gates, native build, clean-code diff scan and whitespace validation.
- [ ] Traverse remaining native workspaces in both appearance modes and complete independent review.

## Wave 12 — query output empty-state composition

- [x] Prove bare query output placeholders inside large empty cards in the native runtime.
- [x] Reuse the shared Lucide/Codex empty-state component for Results, Messages, Explain and History.
- [x] Make Messages, Explain and History output cards span the available workspace width.
- [x] Preserve query execution, explain, history and populated result behavior.
- [x] Capture dark and light SQLite runtime evidence for query output empty states.
- [x] Re-run Rust gates, native build, clean-code diff scan and whitespace validation.
- [ ] Complete the remaining native all-surface traversal, provider matrix and independent review.
