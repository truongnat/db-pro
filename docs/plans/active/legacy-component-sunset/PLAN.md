# Legacy Component Sunset + Core Component Token Audit Fixes

## Goal

Two linked workstreams on `main` (owner workflow: edit directly on `main`, no worktree):

1. **Sunset `crates/ui/src/components/legacy.rs`** — delete dead functions first, then
   migrate live callers onto core components until the file can be removed or reduced
   to a thin compatibility shim that is itself deleted.
2. **Core component token consistency** — fix audit findings (2026-10-22 audit at
   `31c04389`) across `crates/ui/src/components/`: raw `Stroke::new(<numeric>)`,
   floating-surface radius drift, focus-ring recipe divergence, config consts that
   duplicate scale values instead of aliasing tokens, `selection/ui.rs` literals,
   `toggle` `ICON_TEXT_GAP` shadowing.

## Audit summary (HEAD 31c04389, all P2/P3 — no P0/P1 after re-grade)

- `Stroke::new(1.0|1.5|2.0, …)` raw at ~23 sites (tabs, dialog, sheet, overlay, toast,
  hover_card, calendar, agent_primitives, search, legacy).
- Floating-surface radius drift: `POPOVER_RADIUS=12`, `DROPDOWN_RADIUS=12`,
  `TOAST_RADIUS=10`, `CONTEXT_MENU_RADIUS=10`, tooltip `6` vs `RADIUS_*` roles (8).
  Root cause: `floating_surface(theme, corner_radius: f32, …)` takes a raw param.
- Focus ring: `paint_focus_ring` = `border_focus` + `STROKE_THICK` + expanded rect;
  tree `1.0` unexpanded, workspace `STROKE_THIN`, Radio/Slider `accent` + raw width,
  overlay `accent` + `1.5`.
- `toggle/config.rs:4` `ICON_TEXT_GAP = 6.0` shadows global `ICON_TEXT_GAP = 8.0`.
- `select/handler.rs:127` uses `RADIUS_DROPDOWN` as a vertical margin (wrong role).
- Config consts across ~15 modules duplicate scale values (`1.0`, `4`, `8`, `12`,
  `16`, `20`, `24`) instead of `= crate::tokens::*`.
- `selection/ui.rs` bypasses its own config consts; `button` label gap is a literal
  `"  "`; `legacy.rs` dead fns: `input_full_width`, `password_input`, `icon_button`,
  `progress_bar`, `segmented_control`, `kbd_chip`, `tag_chip`, `switch`, `spinner`,
  `skeleton`, `toast` (all verified zero callers incl. tests + token-level scan).
- Colors: CLEAN — zero `is_dark`/`visuals()` branches, zero `from_rgb/from_gray`
  at paint sites; all colors resolve via `DbProTheme` facade.

## Scope

- `crates/ui/src/components/**` (ui/handler/config files).
- `crates/app/**` view callsites for legacy-fn migration.
- Sequential subagent waves (owner request): one wave at a time, each verified by
  `cargo check -p db-pro-ui` (+ `cargo check -p db-pro-app` for caller migrations)
  before the next starts.

## Wave plan

| Wave | Content |
|---|---|
| 0 | Delete 11 verified-dead legacy fns + orphaned helpers |
| 1 | `STROKE_*` token swap (~23 sites) + unify focus rings on `paint_focus_ring` |
| 2 | `floating_surface` radius policy + overlay spacing consts |
| 3 | Config const aliases → `crate::tokens::*`; rename `toggle::ICON_TEXT_GAP`; fix `select/handler.rs:127` wrong-role |
| 4 | `selection/ui.rs` literals → config/tokens |
| 5 | Migrate legacy callers (`badge`, `section_label`, `card_frame`, buttons, `sidebar_item`, `status_dot`, `empty_state`, frames, `icon_text`, `plural_count`, `tab_button`, `input`, grid/panel frames) to core components; shrink/delete `legacy.rs` |
| 6 | Gates: fmt, workspace check, clippy, ui-architecture scan, tests, native captures for radius/focus-ring visual changes |

## Non-goals

- No color-token changes (color layer is already clean).
- No API redesign of component contracts beyond fixing the findings.
- No deletion of review infrastructure.
- Legacy fns with live callers stay until their callers are migrated — no big-bang
  removal of `badge`, `card_frame`, `section_label`, etc.

## Acceptance

- `legacy.rs` deleted or reduced to functions still referenced, with a migration
  note if any remain.
- Zero `Stroke::new(<numeric literal>,` at component paint sites (except clearly
  intentional named consts).
- Focus ring renders identically across focusable components (single recipe).
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`
  (0 baseline), `cargo +nightly fmt --check`, `scripts/check-ui-architecture.sh` PASS.
- Captures (1280×800 light+dark) for surfaces with changed radius/focus ring.

## Risks

- `pub use legacy::*` glob means callers use bare names that collide with module
  names (`badge`, `input`, `toast`) — text search alone misses or over-counts
  callers; verify with compile after each deletion.
- Radius changes (12→8 on popovers/dropdowns) are user-visible; capture before/after.
- Deleting `input` callers is the largest migration surface (~10+ callsites with
  width/hint params mapping onto `components::input::Input` builder API).
