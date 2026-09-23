# Native Core UI Modernization — Checklist

## Planning

- [x] Evidence and quality gaps recorded (PLAN.md)
- [x] Scope/non-goals explicit
- [x] Theme contract preservation rule stated (P3.1-equivalent)
- [x] No provider behavior changes (UI primitives are provider-agnostic)
- [x] No React/TypeScript work in scope

## Implementation — Wave 1: Missing primitives

- [x] `tooltip` — rich tooltip wrapping a response with label + shortcut chip (`components/overlay.rs`)
- [x] `kbd_chip` / `kbd_badge` — Linear/Vercel-style keyboard shortcut pill (`components/feedback.rs`)
- [x] `spinner` — custom-painted rotating arc (`components/feedback.rs`)
- [x] `progress_bar` / `Progress` — linear bar with animation fill (`components/feedback.rs`)
- [x] `skeleton` — rounded placeholder with shimmer (`components/chrome.rs`)
- [x] `switch` — on/off toggle with animated thumb (`components/selection.rs`)
- [x] `segmented_control` / `SegmentedTabs` — pill bar with selected segment (`components/tabs/segmented.rs`)
- [x] `tag_chip` / `Badge` — removable tag and badges (`components/badge.rs`)
- [x] `status_dot` — animated status indicator with optional glow (`components/database.rs`)
- [x] `toast` / `ToastManager` — bottom-right transient notification (`components/overlay.rs`)
- [x] All re-exported from `components/mod.rs` and `lib.rs`

## Implementation — Wave 2: Polish existing primitives

- [x] `search_input` — input with leading icon and clear button (`components/input/search.rs`)
- [x] `accent_card_frame` / `Card` — card frame with tokens (`components/card.rs`)
- [x] `sidebar_item` — smooth hover background transition (`sidebar_view.rs`)
- [x] `empty_state` — accept optional action closure (`components/chrome.rs`)
- [x] Existing `empty_state` still callable

## Implementation — Wave 3: Custom-painted micro-polish

- [x] `paint_glow_dot` — multi-layer alpha dot (`components/animation.rs`)
- [x] `paint_focus_ring` — soft inset ring (`components/input/layout.rs`)
- [x] `paint_tab_indicator` — animated underline that slides (`components/tabs/underline.rs`)
- [x] `paint_accent_gradient` — vertical gradient for primary buttons (`components/button.rs`)

## Implementation — Wave 4: Apply in views

- [x] `navigation_view.rs` — topbar tooltips + status dot
- [x] `explorer_view.rs` — search input with leading icon
- [x] `result_grid_view.rs` — search input + kbd chip on Copy cell
- [x] `connection_view.rs` — driver segmented control + readonly switch
- [x] `palette_view.rs` — kbd chip for shortcut
- [x] `agent_view.rs` — status dot + kbd chip + toast

## Quality gates

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets`
- [x] `cargo test --workspace`

## Verification

- [ ] Light theme traversal (manual)
- [ ] Dark theme traversal (manual)
- [ ] `reduce_motion` disables animations (unit test)
- [ ] `VERIFICATION.md` written with concrete evidence
