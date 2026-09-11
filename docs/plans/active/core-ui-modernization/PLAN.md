# Core UI Modernization — Plan

State: PLANNING → IMPLEMENTING
Branch: `feature/core-ui-modernization`

## Goal

Recompose the React/TypeScript core UI primitives in `frontend/src/components/ui/` to
match the visual quality, micro-interaction polish, and design restraint of leading 2026
desktop/web apps (Linear, Vercel, Cursor, Raycast, Stripe, Codex), without breaking the
locked design token contract (P3.1) or the existing component API surface.

This is a presentation-only change. No business logic, no provider/DTO changes, no native
egui work, no new backend commands.

## Evidence / baseline

### Current state

The 18 primitives in `frontend/src/components/ui/` (alert, alert-dialog, badge, button,
checkbox, context-menu, dialog, dropdown-menu, input, label, popover, scroll-area, select,
sonner, switch, table, tabs, textarea, tooltip) are stock shadcn/ui components installed
via the shadcn CLI. They use the codebase's canonical token layer correctly, but several
quality gaps remain when compared to 2026 references:

- **Button**: `transition-colors` only; no pressed state; primary uses flat `bg-primary`;
  no shimmer / icon-shift / shadow lift patterns; hover simply darkens the same surface.
- **Input / Textarea**: flat `bg-transparent`; no subtle inset or surface tint; focus ring
  is the only feedback signal.
- **Badge**: relies on raw opacity overlays (`/10`, `/20`) with no soft surface tint
  variants; dark mode looks washed out.
- **Dialog / Popover / DropdownMenu / Tooltip**: use stock `tw-animate-css` defaults; no
  directional awareness; no subtle blur on overlay.
- **Tabs**: dual-mode default + line is awkward; no smooth indicator slide; inactive tabs
  rely on color change alone.
- **Switch**: classic thumb slide; no spring or easing refinement.
- **Card / Empty State / Skeleton / Spinner / Kbd / Sheet / Command / Progress / Slider /
  Toggle**: missing entirely — apps must hand-roll or skip them.
- **Dark mode**: surfaces are flat (no subtle elevation gradient); no depth cue beyond
  border opacity.

### Known quality gaps

1. Micro-interactions are limited to `transition-colors`; no transform, shadow lift, or
   icon-shift patterns that 2026 references use as signature cues.
2. Missing primitives force each module to invent its own (search inputs, status badges,
   empty states, loading skeletons) — visual debt accumulates.
3. Light mode looks more "stock shadcn" than dark mode; primary accent lacks depth.

### Existing infrastructure to preserve

- The canonical token contract (`--surface-*`, `--text-*`, `--border-*`, `--accent-*`,
  `--state-*`, `--elevation-*`) is enforced by `npm run check:tokens` in CI.
- All components must keep their current exported API (`data-slot`, props, variants) to
  avoid breaking every module that consumes them.
- Geist Variable + JetBrains Mono are already loaded via `globals.css`.

## Scope

### Wave 1 — Polish existing primitives

- **Button**: add `shadow-xs` default + `shadow-sm` hover lift, smoother hover transition,
  subtle scale-down on active, gradient accent for `default` variant, refined `loading`
  state with spinner position respect, `destructive` gets a clear danger surface (not just
  a tinted background).
- **Input / Textarea**: add a subtle inset surface (`bg-muted/40`), smoother border on
  focus, refined focus ring with offset, invalid state with tinted background, helper
  text slot support.
- **Badge**: soft tinted variants (`success`, `warning`, `destructive`, `info`) with
  proper foreground contrast in both themes; refined dot variant.
- **Switch**: smoother thumb animation with spring-like cubic-bezier, refined track colors
  for unchecked (more visible than current `bg-input`).
- **Tabs (default variant)**: subtle background pill with smoother indicator transition;
  `line` variant gets a smooth underline indicator that animates between triggers via
  a shared `--tabs-indicator-*` CSS variable (no Framer Motion dependency).
- **Tooltip**: refined appearance with subtle blur, restrained arrow, smart offset.

### Wave 2 — Overlay polish

- **Dialog**: smoother backdrop blur, refined scale-in (1 → 1 with very subtle scale
  0.98 → 1), content shadow upgraded to `--elevation-lg`, header/footer divider lines
  made more subtle.
- **Popover**: same elevation upgrade, refined slide distance (8px → 6px for snappier
  feel).
- **DropdownMenu**: menu items get `motion-safe` micro-shift on hover (subtle translateX
  -2px on hover for non-destructive items), refined submenu indicator.
- **AlertDialog**: same motion polish as Dialog, destructive button auto-promoted.

### Wave 3 — Missing primitives

Add (do not invent in modules):

- `skeleton.tsx` — loading placeholders, respects token surface.
- `spinner.tsx` — loading indicator (separate from Button's built-in spinner).
- `kbd.tsx` — keyboard shortcut chip (matches Linear/Vercel aesthetic).
- `separator.tsx` — horizontal/vertical divider.
- `toggle.tsx` — toggle button (Radix primitive, on/off pressed state).
- `slider.tsx` — Radix slider.
- `progress.tsx` — Radix progress.
- `sheet.tsx` — slide-over panel (Radix dialog variant).
- `command.tsx` — Cmd+K palette built on `cmdk` (already a dependency).
- `card.tsx` — basic card primitive (used by many modules).

### Wave 4 — Token refinement (additive, non-breaking)

- Add `--surface-gradient` (subtle elevation gradient for primary surfaces in dark mode).
- Add `--shadow-xs`, `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-xl` aliases
  that compose existing `--elevation-*` tokens (no raw values).
- Add `--motion-spring` and `--motion-emphasized` cubic-bezier tokens.
- Add a `motion-safe:` / `motion-reduce:` utility pattern.
- All new tokens defined in BOTH `:root` and `[data-theme="dark"]` to keep CI clean.

### Wave 5 — Visual primitives

- `empty-state.tsx` — icon + title + description + optional action, used everywhere a
  module currently renders its own ad-hoc empty card.
- `status-dot.tsx` — small colored circle for connection state, run state, etc.
- `kbd-shortcut.tsx` — composes multiple `Kbd` chips with separator (e.g. `⌘ K`).

### Non-goals

- No backend / IPC / Tauri command changes.
- No provider (PostgreSQL / SQLite) behavioral changes.
- No removal of any existing primitive API (additive or polish-only).
- No new design system tokens that conflict with the P3.1 contract.
- No Framer Motion / Motion One / React Spring dependency (CSS-only animations).
- No native egui work (separate branch `feature/native-visual-redesign`).
- No visual changes to `crates/` or `db-pro-*` HTML prototypes.

## Acceptance

- All existing modules still typecheck and pass lint with zero new errors.
- `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`,
  `npm run test`, `npm run build` all pass.
- All new primitives export the same API shape as their shadcn counterparts
  (`data-slot`, `data-*` attributes, `asChild` where applicable, ref forwarding).
- Dark and light themes both render correctly for every new/updated primitive.
- Animations respect `prefers-reduced-motion` (existing `globals.css` rule applies).
- No `--app-*` color tokens introduced; canonical token contract intact.
- At least one Storybook-style render (manual or test) covers each new primitive.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | N/A | UI primitives are provider-agnostic |
| SQLite | N/A | UI primitives are provider-agnostic |
