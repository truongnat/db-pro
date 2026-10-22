# Findings — Legacy Component Sunset

## 2026-10-22 audit (HEAD 31c04389)

Full audit of `crates/ui/src/components/` against three criteria (SPACE / BORDER /
COLOR light-dark). Re-graded severities: **no P0/P1** — all findings are
visual-consistency drift (P2) or polish (P3). Color layer verified clean: zero
`is_dark`/`visuals()` branches at paint sites, zero `from_rgb`/`from_gray`; all
colors resolve through `DbProTheme` facade / `theme.semantic.*` / `*_soft()`.

Key P2 clusters: raw `Stroke::new` widths (~23 sites), floating-surface radius
drift (popover/dropdown 12, toast/ctx-menu 10, tooltip 6 vs `RADIUS_*` roles 8),
four divergent focus-ring recipes, config consts duplicating scale values,
`toggle::ICON_TEXT_GAP` shadowing global token, `select/handler.rs:127`
radius-as-margin wrong role.

## Dead-code verification (Wave 0)

Method: `rg -w <fn> -g '*.rs' crates` + call-shape scan, incl. tests and
import-level references; `pub use legacy::*` glob means callers use bare names —
verified compile-clean after deletion (`cargo check --workspace` PASS, zero
unused-import warnings even at deleted sites).

Deleted: `input_full_width`, `password_input`, `icon_button`, `spinner`,
`progress_bar`, `skeleton`, `switch`, `segmented_control`, `kbd_chip`, `tag_chip`,
`toast`, plus `UiLevel` + `level_colors` (only served `toast`).

Kept (live callers confirmed): `input` (~10 app callsites), `badge`, `card_frame`,
`section_label`, `sidebar_item`, `status_dot`, `empty_state`, `toolbar_frame`,
`panel_frame`, buttons, `tab_button`, `grid_frame`, `shell_surface_frame`,
`icon_text`, `plural_count`, frames. `tinted` kept (used by `status_dot`).
`legacy.rs` 957 → ~516 lines.

## Decisions

- `legacy.rs` is NOT dead — staged sunset: dead fns first, then caller migration.
- Severity policy: consistency drift is P2; nothing here blocks a release.
