# Query editor Zed-like feel

## Goal

Make the SQL query editor feel smooth and polished like Zed: real scrolling,
blinking caret, softer chrome, comfortable line rhythm.

## Scope

- `crates/ui/src/editor/renderer.rs` — ScrollArea, caret blink, gutter/line/selection polish
- Light tests for caret blink / scroll content size
- Non-goals: LSP, minimap, multi-cursor, sticky scroll headers

## Acceptance

- Wheel / trackpad scrolls the SQL surface (vertical + horizontal when wide)
- Caret blinks when idle; stays solid briefly after typing/moving
- Cursor stays in view after keyboard navigation
- Current line + selection + gutter read softer (Zed-like density)
- Existing editor unit tests still pass

## Provider

UI-only; PG/SQLite n/a beyond existing editor dialect paths.
