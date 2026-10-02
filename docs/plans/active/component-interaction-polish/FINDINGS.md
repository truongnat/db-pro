# Findings and reusable lessons

Source: `cd58b354a43f55458759c298bc053c5a4dd36c0b`; 2026-10-02.

1. `Alert::show` requested a wide child UI, but egui shrank its allocation to short content. `set_min_width(text_avail)` keeps the reserved copy column and places dismiss at the trailing edge. Test covers narrow/wide and wrapped/short copy; native screenshot confirms placement.
2. Button previously shrank only its paint rectangle. Transforming the entire egui shape scales glyph meshes, icon, underline and chrome together without relayout or changing hit geometry. Regression test also checks original galley/UVs remain untouched.
3. egui 0.29.1 `paint_text_selection` inserts a background quad but does not recolor glyphs. Setting `selection.stroke` alone cannot make selected text white. A single theme-installed end-pass adapter recognizes the native selection quad by its triangle placement and color, then updates selected glyph colors only. Labels and TextEdit retain native selection, clipboard and accessibility behavior.
4. The adapter depends on egui 0.29's mesh representation. Preserve its Label/TextEdit regression tests on dependency upgrades. It skips frames with no focus/label selection and does not reshape or rasterize fonts. No measured performance improvement is claimed.

P0/P1: none found in self-review. P2: formal viewport/animation evidence gaps below. Providers: n/a, no database changes. Learning pass recorded here; no global memory writes.
