# _archive/bench/ — LEGACY (React-era ER renderer benchmarks)

**Status: historical / archived. Not part of the product build.**

These harnesses compare the React-based ER diagram renderers (React Flow / xyflow,
Cytoscape) that were used before the UI moved to native `eframe`/`egui`. The React frontend
was archived under `_archive/frontend/` on 2026-09-11; this benchmark harness was archived
alongside it on the same day. The ER diagram is now a native `egui::Painter` in
`crates/ui/src/diagram_view.rs`.

Nothing here is built, tested, or packaged by CI. It is kept only as a record of the
renderer comparison that informed the original ER design.

| Path | Purpose |
|---|---|
| `er-reactflow.html`, `er-cytoscape.html`, `er-renderer-runtime.html` | Browser renderer comparison harnesses |
| `build-er-renderer-runtime.mjs`, `run-bench.js`, `fixture-gen.js` | Fixture generation and benchmark runners |
| `*.png` | Captured comparison screenshots (100 / 500 / 1000 nodes) |
| `token-contract.html` | Design-token contract demo for the retired shadcn/Tailwind token layer |
| `vendor/` | Vendored React, xyflow, Cytoscape, and dagre bundles |

Current ER performance targets and the measurement method live in
`.skills/perf-audit/references/perf-budgets.md`.

The harness is kept read-only for historical reference. Deleting it is safe once the
React-era renderer comparison is no longer needed.
