# Findings

Baseline SHA: `78d914887642dd400c7e2048db9a24d0403619c8`.
All runtime evidence collected with `db-pro-native` built `--features capture`,
`DB_PRO_WINDOW_SIZE=1280x800` (2× → 2560×1600 physical), dark + light.

## Confirmed defects and root causes

### F1 — Explain plan stat text slides under badges (P2, user-reported)

`crates/ui/src/components/explain/ui.rs` — `render_node` anchored the per-node
cost/time/rows string via `ui.with_layout(Layout::right_to_left)` filling the
row's remaining width. When remaining width < measured text width, egui places
the label so its left edge underlaps the icon/title/bar/badges → the overlap on
`Seq Scan on users` in the owner's screenshot.

Fix: measure `stat_str` with `FontId::monospace(FONT_SIZE_CAPTION)` and allocate
`available_width().max(stat_width)` via `allocate_ui_with_layout`. When the stat
doesn't fit, the row widens inside the existing `ScrollArea::both()` instead of
overlapping; the label lands flush after the last badge. Header labels
(`Total Exec`/`Planning`) got `Label::truncate()` — same hazard, bounded region.

### F2 — ConnectionCard claims column gutter + 8pt of the next column (P2, user-reported)

`crates/ui/src/components/database/ui.rs` — the host row reserved
`SPACE_XS` before the SSL badge, but `add_sized` + `allocate_exact_size` make
egui also insert `item_spacing.x` (8pt) between them. Reservation counted the
gap once; reality charged it twice → the row's `min_rect` grew 8pt past the
column edge. Measured: column `562..899`, card response `562..907`.

Fix: `ssl_gap = ui.spacing().item_spacing.x + SPACE_XS` when SSL is on.

### F3 — StatusBar left/right item collision (P2, found during capture)

`crates/ui/src/components/workspace/ui.rs` — left items painted forward from
`rect.left()+SPACE_MD`, right items backward from `rect.right()-SPACE_MD`, with
no bound between them → `UTF-8 · READ COMMITTED` overlapped `Ln 42, Col 18`.

Fix: measure the right block first (`right_status_block_width`), stop left items
at `left_status_items_limit` (block left edge − `SPACE_MD`), and collapse the
rest into a `…` indicator whose hover lists the hidden items. Right items
additionally stop before crossing the inner left edge.

### F4 — Glyph alpha samples invisible on light theme (P2, user-reported)

`crates/ui/src/component_gallery_rendering.rs` —
`Color32::from_rgba_unmultiplied(224, 228, 235, alpha)` is a light gray tuned
for dark surfaces; on light surfaces all alpha rows were near-invisible.

Fix: base color = `theme.text_primary`, alpha still modulated — the diagnostic
intent (alpha on identical glyphs) is preserved for both themes.

### F5 — Duplicate egui widget Ids inside `ui.columns` (P2, found during capture)

Debug builds paint "First/Second use of widget/ScrollArea ID" callouts.
`Ui::columns` gives every column child `stable_id = parent.with("child")`
(egui 0.29 `Ui::new_child` default), so unsalted `ScrollArea`s in same-shaped
column content share scroll state — captured pair: explain card `ScrollArea`
vs `TerminalBlock` scroll (`FA5A`), plus the scroll-area drag interact
(`481B`).

Fix: salted each component scroll area —
`explain` uses the existing unique `tree_id`; `DiffViewer` and `TerminalBlock`
use `ui.auto_id_with("…_scroll")`. Verified overlays gone in recapture.

The same latent hazard exists anywhere `ui.id().with(name)` appears inside
`columns()`; systemic audit is out of scope (recorded as follow-up).

## Testability notes

- Pure helpers added/tested: `right_status_block_width`, `left_status_items_limit`.
- Visual acceptance: see `VERIFICATION.md` + `evidence/`.
