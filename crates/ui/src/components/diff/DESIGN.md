# Diff design

`mod.rs` re-exports `DiffViewer`, `DiffLine`, `DiffLineType`, visual/stat helpers, and the public layout constants from `config.rs`. `ui.rs` paints the framed header, change counts, line-number gutters and rows; `handler.rs` constructs line data and derives counts, styles, geometry and formatted number columns; `config.rs` owns the diff viewer's sizing and typography.

## Rendering flow and cost

The caller supplies an already ordered slice of context/added/removed lines. Each frame the viewer counts changes, measures the widest line and line-number width, lays out each content galley once, then paints the rows inside a horizontal scroll area. Empty input gets a centered no-changes message. Cost is linear in supplied lines and text measured, with retained per-frame line galleys; there is no diff algorithm, vertical virtualization, or input size limit in this widget.

## Layout and accessibility

The title is clipped before the summary counts; line contents scroll horizontally when wider than the viewport. Old/new line numbers share a minimum three-character gutter that expands to the largest input number. Added and removed rows use semantic soft fills and explicit `+`/`-` markers so meaning is not conveyed by color alone. Long lines remain available through horizontal scrolling, while the title can be clipped on narrow panels. Each row publishes its content as an egui label; provide meaningful line text in caller data.
