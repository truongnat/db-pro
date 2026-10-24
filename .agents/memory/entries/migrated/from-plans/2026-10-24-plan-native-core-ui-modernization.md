---
schema_version: 1
id: "MEM-PLAN-native-core-ui-modernization-61cbf06ba28d"
title: "Plan archive: Native Core Ui Modernization"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-core-ui-modernization]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "61cbf06ba28d47238b4e8bd458fe5c0e166c08ae29a362d1c1345f3f6383bc63"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-core-ui-modernization/PLAN.md"
  - "docs/plans/active/native-core-ui-modernization/FINDINGS.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-core-ui-modernization/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Native Core UI Modernization — Findings

## F1 — Tooltips are inline-only; no rich tooltip primitive exists

**Evidence**: `crates/ui/src/components.rs` defines `icon_text(...)` and the
egui-native `.on_hover_text(...)` is used in 30+ call sites
(see `grep -rn on_hover_text crates/ui/src`).

**Failure scenario**: A modern IDE/DB tool surfaces a label plus its keyboard
shortcut in the tooltip. Without that, users have to memorize or look up the
shortcut in a help dialog. Linear, Vercel, Raycast, Cursor all show the
shortcut inline in the tooltip.

**Severity**: P2.

**Smallest coherent fix**: Add a `tooltip(ui, body, text, shortcut, theme)`
wrapper that:
1. shows `on_hover_text(text)` on the wrapped response;
2. when hovered, also paints a `kbd_chip` next to the text inside a tooltip
   popup using egui's `show_tooltip_at` API.

## F2 — Spinner / skeleton are missing; loading states are jarring

**Evidence**: `crates/ui/src/components.rs` has no `spinner` or `skeleton`
function. `egui::Spinner` is used in some places (e.g. `runtime progress`)
but the default egui spinner is visually flat.

**Failure scenario**: When the agent provider is loading or the schema is
refreshing, the UI shows a static text label. Modern apps use either a small
spinner next to the action or a skeleton placeholder in the loading region.

**Severity**: P2.

**Smallest coherent fix**: Add `spinner(ui, color, theme)` that paints a
rotating arc using `ctx.animate_value` (rotation 0..1 → 0..360°).
Add `skeleton(ui, width, height, theme)` that paints a rounded rect filled
with `surface_hover` and overlays a moving shimmer using
`ctx.animate_value` (offset 0..1 → -width..+width).

## F3 — Switch / segmented control primitives are missing

**Evidence**: `crates/ui/src/components.rs` does not define `switch` or
`segmented_control`. The connection dialog and settings use raw egui
`ComboBox` or text labels.

**Failure scenario**: Boolean settings (readonly, SSL mode) and mode pickers
(driver, output tab) look like raw form fields rather than the modern
toggle/picker pattern.

**Severity**: P2.

**Smallest coherent fix**: Add `switch(ui, &mut bool, label, theme)` that
paints a track + thumb with smooth thumb animation via
`ctx.animate_value_bool`. Add `segmented_control(ui, &[&str], selected,
theme) -> Option<usize>` that paints a pill bar with the selected segment
highlighted using `surface_active`.

## F4 — No kbd chip for keyboard shortcuts

**Evidence**: `crates/ui/src/palette_view.rs:18` stores `shortcut:
Option<String>` and renders it as a `RichText` label. There is no styling
that visually distinguishes a shortcut from regular text.

**Failure scenario**: Keyboard shortcuts are listed but don't pop visually.
Modern apps use rounded monospace pills (Linear's `⌘K`, Vercel's `⌘P`).

**Severity**: P2.

**Smallest coherent fix**: Add `kbd_chip(ui, label, theme)` that draws a
rounded monospace pill with `surface_elevated` fill and `border_default`
stroke. Use it in `palette_view.rs` and on topbar buttons that have
shortcuts.

## F5 — Status indicator is a glyph, not a glowing dot

**Evidence**: `crates/ui/src/app.rs:579` `connection_indicator` returns
`Icon::CircleCheck` / `Icon::Circle`. `crates/ui/src/navigation_view.rs:32`
uses `icon_text(connection_icon, "", connection_color)`.

**Failure scenario**: Trending apps (Linear, Raycast) use a small colored dot
that can pulse for live state. The current glyph-based indicator looks
utilitarian.

**Severity**: P2.

**Smallest coherent fix**: Add `status_dot(ui, color, pulsing, theme)` that
paints a filled circle with optional outer ring pulse (multi-layer alpha,
pulse radius animated via `ctx.animate_value_bool`).

## F6 — No toast / snackbar for transient notifications

**Evidence**: `crates/ui/src/app.rs:304` stores `runtime_message: String`
that is shown in the statusbar only. There is no toast/snackbar primitive.

**Failure scenario**: When the user saves a query, the feedback is "Saved
query" appearing in the statusbar. Modern apps use a transient toast
that fades in/out near the cursor or in the bottom-right corner.

**Severity**: P2.

**Smallest coherent fix**: Add `toast(ui, level, message, theme)` that
paints a small rounded panel with a colored left accent strip and a
subtle elevation shadow. Use it for `runtime_message` feedback when the
message is at "Success" or "Failed" level.

## F7 — Search input has no leading icon, no clear button

**Evidence**: `crates/ui/src/explorer_view.rs` and `result_grid_view.rs` use
the basic `input(ui, &mut value, hint, width, theme)` helper. The user has
no visual hint that the field is a search field, and no quick way to clear.

**Severity**: P2.

**Smallest coherent fix**: Add `search_input(ui, value, hint, leading_icon,
width, theme)` that:
1. allocates an inner rect inside the input rect for a leading icon and a
   clear X;
2. renders the leading icon in `text_muted` color;
3. shows the clear X only when `value.is_empty() == false`;
4. adjusts the input text padding to leave room for both icons.

The existing `input` API stays untouched.

## F8 — Empty state lacks an action slot

**Evidence**: `crates/ui/src/components.rs:141` `empty_state(ui, icon, title,
description, theme)` does not take a closure for a CTA button.

**Failure scenario**: When no connection is selected, the empty state could
suggest "Add a connection" with a button, but currently shows text only.
Modern apps always offer a next action from the empty state.

**Severity**: P2.

**Smallest coherent fix**: Add `empty_state_with_action(ui, icon, title,
description, theme, action: impl FnOnce(&mut Ui) + 'static)` and keep the
existing `empty_state` as a thin wrapper that passes `|_| {}` to the new
function.

## F9 — Custom-painted micro-polish opportunities

**Evidence**: `crates/ui/src/workspace_view.rs` already calls
`paint_tab_indicator(...)` (a 2px left accent line). Modern apps animate
the indicator sliding between tabs.

**Severity**: P3 (visual polish, not a correctness defect).

**Smallest coherent fix**: Replace the static 2px left line with a 2px tall
underline that interpolates between the previous and current tab's center-x
using `ctx.animate_value_with_time`. Skip the animation when
`reduce_motion` is true.

