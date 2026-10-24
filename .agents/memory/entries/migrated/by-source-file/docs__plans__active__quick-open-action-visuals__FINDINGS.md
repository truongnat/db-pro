---
schema_version: 1
id: "MEM-SRC-a42be093a6780ba8"
title: "Source mirror: docs/plans/active/quick-open-action-visuals/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/quick-open-action-visuals/FINDINGS.md"
source_sha256: "a42be093a6780ba8d9a9d63e865b5280a289a60271bac8361276d5bf96870fbb"
mirror_body_sha256: "a42be093a6780ba8d9a9d63e865b5280a289a60271bac8361276d5bf96870fbb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

At baseline `57cea2bf746e132564cde952cc06d0d04d796aa0`, the selected Quick Open scope was painted by `ui.selectable_label`. `DbProTheme::apply` sets `Visuals::override_text_color`, which egui bakes into the label galley before selected widget visuals are applied. The baseline regression test observed `#1A1C1F` in the selected glyph while the selection foreground was `#FFFFFF`.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: selected filter labels now pass the current egui selection foreground explicitly. The separate selected-row outline and scope-filter hover/active strokes were removed.

P2, fixed in `78d914887642dd400c7e2048db9a24d0403619c8`: Button Outline, disabled, and loading palettes emitted decorative strokes. All action Button variants now stay stroke-free through hover interpolation, loading, and disabled states. The dedicated keyboard focus ring remains.

Dark-theme solid buttons can still have black text by design. `DbProTheme::text_on_solid` chooses black when the accent fill is bright enough to provide stronger normal-text contrast; this is separate from selected-label color.

No P0/P1 findings. No database or provider impact.

## 2026-10-19 regression evidence

Baseline SHA: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`, followed by an uncommitted regression test.

- P2: `SearchInput::show` used `with_layout` with a centered horizontal layout but no row-height constraint. In a tall vertical container the frame consumed the remaining height. The regression command `cargo test -p db-pro-ui search_keeps_results_visible_in_a_tall_container -- --nocapture` failed with the first result at `[[8.0 796.0] - [130.2 812.0]]`. The shared component now allocates an explicitly row-sized left-to-right layout.
- P2: shell TopBottomPanel/SidePanel defaults independently painted separator strokes over existing semantic subtle frame borders. Disable idle separators on the topbar, statusbar, Agent panel, and shell output dock; preserve resize behavior.
- PostgreSQL: n/a, no SQL/provider path changes. SQLite: n/a, no SQL/provider path changes.

### Theme and modal follow-up

Baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; candidate remains an uncommitted patch, bound by `evidence/2026-10-19/source-manifest.json`.

- P2: egui 0.36 `set_visuals` did not select the context's Light/Dark style. Eframe also creates the root `Ui` before calling application logic, so that UI retained its earlier style snapshot. The fix selects the egui theme explicitly, mutates its global style consistently, and refreshes the root style before drawing the shell.
- Toolbar toggles now synchronize the Settings value and request repaint; Settings and Gallery follow the same behavior. The pointer-driven regression covers toolbar, Settings, three subsequent frames, rendered fills, and save/restore. Settings apply-on-restore was not a per-frame overwrite; that initial hypothesis was rejected.
- P2: after fixing SearchInput, runtime inspection still showed excess empty list space and keyboard help clipped by the scrolling modal body. The list now shrinks to content and the help lives in DialogFrame's fixed footer. `empty_palette_keeps_keyboard_help_inside_the_dialog` failed before and passes after.
- Observed verification limitations, not UI regressions fixed here: native sample SQLite initialization cannot open its fixture path; full-workspace tests stop on that existing-path test; Clippy reports errors in untouched core/UI paths. No baseline checkout was executed, so those gates are not presented as a measured before/after comparison.
- Initial captures encountered AppKit `NSRangeException` while shutting down after writing PNGs; six larger captures timed out. Final native matrix is 36/36 exit 0 with valid images. The successful environment attempted a separate HOME and backend data directory, but the rendered app still restored the saved PostgreSQL connection; do not infer full storage isolation.
- Existing saved PostgreSQL connection reports database-not-found in the background. Quick Open's Schema scope therefore has no live schema entries; captures prove compact search, empty content, and visible keyboard help, not populated live-database search.
