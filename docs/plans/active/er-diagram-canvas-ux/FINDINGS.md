# Findings — ER Diagram Canvas UX

## Root causes (pre-change SHA: see VERIFICATION)

1. **Dual navigation**: `draw_diagram_canvas` allocates a painter sized
   `world_size * zoom` inside `ScrollArea::both()`, then *also* applies
   `ctx.diagram.pan` in `ErViewport`. Scroll handles drag on the bar/gesture
   layer while pan shifts inside the allocated rect — pointer drags land on the
   scroll area, so the map feels locked ("không drag được") and content can be
   pushed outside the allocated rect ("không xem được").
2. **No node interaction**: positions come from `ErGraph::build` grid only;
   nothing hit-tests a node for drag. Click exists (`hit_test_node` →
   `OpenTable`) but no move.
3. **No wheel zoom / no fit-on-open**: zoom is only the floating −/+ widget.
4. **Truncation by chars**: header title hard-capped at 18 chars regardless of
   node width (280px ≈ 36+ chars at 13px), so every card reads
   `main.HumanResourc…`.
5. **Edge stacking**: single `bend_x = (from.x + to.x)/2` shared by every edge
   between adjacent columns; labels centered on the bend overlap each other.

## Owner direction (2026-10-07)

- Toolbar → compact icon row (context chips left, icon actions right).
- Interactions: node drag + pan/wheel zoom + auto fit + minimap.
- Manual positions persist across sessions.

## Findings during verification

- **Compact LOD band was too wide** (zoom < 0.75): at ~0.6, nodes rendered as
  large empty cards. Band lowered to < 0.45 so mid zooms render real columns.
- **Capture driver is feature-gated** (`--features capture`, off by default):
  `DB_PRO_CAPTURE_*` env vars are inert without it — the app opens normally and
  never writes a PNG, which masquerades as a "hang".
- **Two flaky first-paint stalls** in the capture path on macOS (fixed):
  invisible windows receive no `drawRect`, and both eframe window-state
  restore and the app's 3-frame startup `Maximized(true)` fought
  `pin_viewport`. See `VERIFICATION.md`.
- Persisted viewport/zoom is restored across runs — confirmed incidentally by
  the light capture rendering at a previously saved 60% zoom.
