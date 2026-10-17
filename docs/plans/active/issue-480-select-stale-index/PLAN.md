# #480 — Select dropdown stale index when options shrink

## Goal
`navigate_selection` must clamp `selected` into `0..option_count` before
applying arrow keys so a stale index (options shrank while the menu stayed
open) lands on the last valid option instead of staying out of bounds.

## Scope
- `crates/ui/src/components/select/handler.rs` — clamp `selected` via
  `.min(option_count.saturating_sub(1))` at the top of `navigate_selection`.
- Popup render (`ui/select.rs:365-385`) already reads via
  `options.iter().enumerate()` / `.get()` and `selected_label` uses
  `options.get` — verified safe, no change needed.

## Non-goals
- No wrap-around navigation (existing contract: arrows clamp at ends).
- Signature stays `usize` — callers treat the select as always-indexed.

## Tests
`tests/handler_tests.rs::stale_selection_index_clamps_when_options_shrink`:
5→2 shrink with ArrowDown → 1; idle clamp; ArrowUp from stale index; empty
list → 0.
