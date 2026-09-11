# Native Core UI Modernization — Checklist

## Planning

- [x] Evidence and quality gaps recorded (PLAN.md)
- [x] Scope/non-goals explicit
- [x] Theme contract preservation rule stated (P3.1-equivalent)
- [x] No provider behavior changes (UI primitives are provider-agnostic)
- [x] No React/TypeScript work in scope

## Implementation — Wave 1: Missing primitives

- [ ] `tooltip` — rich tooltip wrapping a response with label + shortcut chip
- [ ] `kbd_chip` — Linear/Vercel-style keyboard shortcut pill
- [ ] `spinner` — custom-painted rotating arc
- [ ] `progress_bar` — linear bar with gradient fill
- [ ] `skeleton` — rounded placeholder (shimmer on repaint)
- [ ] `switch` — on/off toggle with animated thumb
- [ ] `segmented_control` — pill bar with selected segment
- [ ] `tag_chip` — removable filter tag
- [ ] `status_dot` — animated status indicator with optional glow
- [ ] `toast` — bottom-right transient notification
- [ ] All 10 re-exported from `lib.rs`

## Implementation — Wave 2: Polish existing primitives

- [ ] `search_input` — input with leading icon and clear button
- [ ] `accent_card_frame` — card frame with colored left strip
- [ ] `sidebar_item` — smooth hover background transition
- [ ] `empty_state` — accept optional action closure
- [ ] Existing 4-arg `empty_state` still callable

## Implementation — Wave 3: Custom-painted micro-polish

- [ ] `paint_glow_dot` — multi-layer alpha dot
- [ ] `paint_focus_ring` — soft inset ring
- [ ] `paint_tab_indicator` — animated 2px underline that slides
- [ ] `paint_accent_gradient` — vertical gradient for primary buttons

## Implementation — Wave 4: Apply in views

- [ ] `navigation_view.rs` — topbar tooltips + status dot
- [ ] `explorer_view.rs` — search input with leading icon
- [ ] `result_grid_view.rs` — search input + kbd chip on Copy cell
- [ ] `connection_view.rs` — driver segmented control + readonly switch
- [ ] `palette_view.rs` — kbd chip for shortcut
- [ ] `agent_view.rs` — status dot + kbd chip + toast

## Quality gates

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets`
- [ ] `cargo test --workspace`

## Verification

- [ ] Light theme traversal (manual)
- [ ] Dark theme traversal (manual)
- [ ] `reduce_motion` disables animations (unit test)
- [ ] `VERIFICATION.md` written with concrete evidence
