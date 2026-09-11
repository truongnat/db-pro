# Native Core UI Modernization — Plan

State: PLANNING → IMPLEMENTING
Branch: `feature/native-core-ui-modernization`

## Goal

Recompose the **native egui** core UI primitives in `crates/native-app/` and
`crates/ui/` so the DB Pro desktop shell matches the visual restraint,
micro-interaction polish, and information density of leading 2026 desktop apps
(Linear, Vercel, Cursor, Raycast, Stripe, Codex, Arc). All work stays on the
native Rust/egui side; no React/TypeScript changes.

This is presentation-only. No domain/business logic, no provider behavior, no
runtime protocol changes, no public API changes outside of `crates/ui`.

## Scope vs neighboring work

| Plan | What it owns |
|---|---|
| `feature/core-ui-modernization` | React/TypeScript primitives in `frontend/src/components/ui/` |
| `feature/native-visual-redesign` | Initial native visual redesign (theme + shell) — done |
| **`feature/native-core-ui-modernization` (this plan)** | Native primitives quality + missing widget coverage + micro-interaction polish |

The current `crates/ui/src/theme.rs` already follows the codex-neutral surface
contract (light: `249/249/249` app, white panel; dark: `24/24/24` app,
`33/33/33` panel). This plan adds the **component** layer on top of that
foundation.

## Evidence / baseline

### Current native primitives in `crates/ui/src/components.rs`

- Frame helpers: `panel_frame`, `sidebar_frame`, `activity_bar_frame`,
  `toolbar_frame`, `tab_frame`, `card_frame`, `agent_message_frame`,
  `editor_frame`, `grid_frame`
- Buttons: `primary_button`, `primary_button_with_icon`, `secondary_button`,
  `secondary_button_with_icon`, `ghost_button`, `ghost_button_with_icon`,
  `compact_button`, `compact_button_enabled`, `compact_button_with_icon`,
  `danger_button`, `menu_button_with_icon`, `icon_button`,
  `compact_icon_button`, `compact_icon_button_enabled`
- Inputs: `input`, `input_full_width`, `password_input`
- Misc: `empty_state`, `section_label`, `sidebar_item`, `badge`, `icon_text`

### Quality gaps vs 2026 references

| Gap | Evidence | Severity |
|---|---|---|
| No rich tooltip widget (label + shortcut chip + description) | only `.on_hover_text(...)` is used in 30+ call sites | P2 |
| No spinner / progress indicator | loading is rendered as plain `ui.spinner()` (egui default, low polish) or a text label | P2 |
| No skeleton placeholder | loading states jump straight to data | P2 |
| No switch / toggle primitive | `readonly` toggles + SSL mode rendered as combobox-style | P2 |
| No segmented control | topbar `db`, `schema`, `driver` metadata rendered as plain text labels | P2 |
| No kbd chip | palette `shortcut` field is a `String`, rendered as text; no chip styling | P2 |
| No tag chip | filter tags rendered ad-hoc; nothing removable | P2 |
| No status dot (animated) | connection indicator uses an outline icon glyph, not a glowing dot | P2 |
| No toast / snackbar | "Query failed" / "Connection saved" rendered inline via `runtime_message` only | P2 |
| Search input has no leading icon, no clear button | `explorer_search` / `grid_filter` use raw `input(...)` | P2 |
| Sidebar item active indicator is a 2px left line | consistent, but lacks depth (no fill on hover transition) | P2 |
| Empty state is small | 18px icon, title + description only; no action slot | P2 |
| Column divider polish | cursor change + accent stroke on hover is committed but not yet on `main` | P3 |

### Existing infrastructure to preserve

- `DbProTheme` codex-neutral surface contract (P3.1-equivalent on the native
  side). Light + dark tokens must stay distinct (`theme::tests::light_tokens_*`).
- `lib.rs` exports the `crates/ui` API. Adding new primitives requires adding
  them to `pub use components::{...}`.
- All existing call sites must keep working — no breaking signature changes
  for `primary_button`, `secondary_button`, `ghost_button`, `input`, etc.
- `lucide_icons` icon font already installed via `DbProTheme::install_fonts`.
- Existing tests in `crates/ui/src/theme.rs` and `crates/ui/src/app_tests.rs`
  must continue to pass.

## Scope

### Wave 1 — Missing primitive widgets (additive)

Add to `crates/ui/src/components.rs` and re-export from `lib.rs`:

- `tooltip(ui, body_response, text, shortcut, theme)` — wraps a widget with a
  rich tooltip showing label + optional `kbd_chip` for the shortcut.
- `kbd_chip(ui, label, theme)` — rounded monospace pill, matches Linear /
  Vercel shortcut chips.
- `spinner(ui, color, theme)` — custom-painted rotating arc, 14px, replaces
  the default egui spinner in any call site that opts in.
- `progress_bar(ui, progress, theme)` — linear bar with subtle gradient fill
  and rounded ends.
- `skeleton(ui, width, height, theme)` — rounded placeholder that uses the
  `surface_hover` token as base + a translucent `surface_active` overlay for
  shimmer (drawn as a frame on every frame using `ui.ctx().request_repaint()`
  while shown).
- `switch(ui, value: &mut bool, label, theme)` — on/off toggle with smooth
  thumb animation (state-based animation; egui is immediate-mode so we use
  `ctx.animate_value_bool`).
- `segmented_control(ui, options, selected, theme) -> Option<usize>` — pill
  bar with selected segment using `surface_active` background.
- `tag_chip(ui, label, removable, theme) -> bool` — rounded tag with optional
  close X; returns true if the close X was clicked.
- `status_dot(ui, color, pulsing, theme)` — small dot, 8px, with optional
  pulsing glow (pulsing uses `ctx.animate_value_bool`).
- `toast(ui, level, message, theme)` — toast notification in the bottom-right
  with backdrop dim and a colored left accent strip.

### Wave 2 — Polish existing primitives (signature-compatible)

- `input` — add optional `leading_icon` and clear-button support. Keep
  signature backwards compatible by adding a new `search_input` helper.
- `search_input(ui, value, hint, leading_icon, width, theme)` — wraps input
  with a leading icon and a clear X (visible only when value is non-empty).
- `card_frame(theme)` — add optional accent border variant:
  `accent_card_frame(theme, accent_color)`.
- `sidebar_item` — keep the 2px left accent line but add a smoother
  hover/active background transition (state already encoded by the
  `active: bool` parameter — we only need to swap colors, no signature change).
- `empty_state` — accept an optional `action` closure to render a CTA button
  underneath. Keep the existing 4-arg version via a thin wrapper.

### Wave 3 — Custom-painted micro-polish

- `paint_glow_dot(painter, center, color, radius)` — multi-layer alpha dot
  for status indicators.
- `paint_focus_ring(painter, rect, color, theme)` — soft inset ring used
  inside `search_input` on focus.
- `paint_tab_indicator(painter, rect, accent, theme)` — extend the existing
  tab indicator with a 2px tall underline that animates between tabs (state
  interpolation via `ctx.animate_value`).
- `paint_accent_gradient(painter, rect, accent, accent_hover)` — vertical
  gradient used by `primary_button` for the "premium" lift.

### Wave 4 — Apply new primitives in views

Targeted updates (small, focused):

- `navigation_view.rs` — replace inline `compact_icon_button(...).on_hover_text(...)`
  with `tooltip(ui, ..., label, shortcut_chip)`. Use `status_dot` for the
  connection indicator on the topbar.
- `explorer_view.rs` — replace `input(ui, &mut explorer_search, ...)` with
  `search_input(ui, ..., Icon::Search, ...)`.
- `result_grid_view.rs` — replace inline "Filter" `ui.label` + `input` with
  `search_input`; show `kbd_chip` after "Copy cell" button with the
  shortcut hint.
- `connection_view.rs` — replace the connection dialog's inline form layout
  with `input` + `password_input` + new `switch` for `readonly` /
  `ssh_tunnel_enabled`. Use `segmented_control` for the driver picker
  (Postgres / SQLite).
- `palette_view.rs` — render `kbd_chip` for `shortcut` instead of plain text.
- `agent_view.rs` — render a `toast` (or an inline status badge) for the
  agent's provider state (Ready / Loading / Failed) using `status_dot` +
  `kbd_chip` for the model shortcut.

### Non-goals

- No backend / IPC / runtime command changes.
- No provider (PostgreSQL / SQLite) behavioral changes.
- No React/TypeScript changes.
- No new design tokens that conflict with the existing codex-neutral surface
  contract (additive only — new tokens are derived from existing ones).
- No new third-party dependencies.
- No changes to `crates/native-app/src/main.rs` startup logic (we touch only
  what is needed for the new primitives).
- No animations longer than 200ms; all state animations respect a
  `reduce_motion` flag stored in `DbProApp` (already exists).

## Acceptance

- All existing call sites still compile with no signature break.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets`, `cargo test --workspace` pass.
- All new primitives have unit tests in `crates/ui/src/components.rs` (or a
  new `components_tests.rs`) covering default values, hover/animation state,
  and the public re-exports.
- Visual sweep: each new primitive renders correctly in both light and dark
  theme (verified manually; runtime evidence recorded in VERIFICATION.md).
- `reduce_motion = true` disables all animate_value usage in the new widgets
  (verified by unit test).
- No new dependencies added.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | N/A | UI primitives are provider-agnostic |
| SQLite | N/A | UI primitives are provider-agnostic |

## Verification plan

- `cargo test --workspace` — all green.
- Manual light + dark traversal of every new primitive (recorded as
  screenshots in VERIFICATION.md if possible; otherwise source-level evidence).
- `bash .skills/perf-audit/scripts/perf-scan.sh` — no new performance
  regressions.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — clean.

## Roll-out

Each wave is committed as a single coherent change so the diff is easy to
review. Wave order is intentionally front-loaded (Wave 1 primitives must land
before Wave 4 can use them).
