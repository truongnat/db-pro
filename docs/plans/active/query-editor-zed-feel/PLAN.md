# Query editor — reliable Zed-like interaction

## Goal

Make the native SQL editor predictable and responsive enough for daily database work: Zed-like typing and navigation quality, with at least the core query-tab interaction reliability expected from DBeaver.

Correctness comes before suggestion volume. Ordinary typing must never trigger AI traffic, apply stale completion ranges, insert unrelated text, or move the caret unexpectedly.

## Scope

### Editor interaction engine

- `crates/ui/src/editor/renderer.rs` — keyboard/IME intent capture, scrolling, caret and text rendering
- `crates/ui/src/editor/interaction.rs` — centralized policy for completion and AI-prediction intents
- `crates/ui/src/editor/completion.rs` — version-bound completion sessions and navigation state
- `crates/ui/src/query_editor_panel.rs` — application orchestration, guarded completion application and popup presentation

### User experience

- Stable multilingual typing and undo grouping
- Completion opens explicitly with `Ctrl/Cmd+Space` or contextually after `.`
- AI prediction is opt-in and explicitly requested; never scheduled by ordinary typing/caret movement
- Completion rows expose kind, detail and explanatory documentation on selection/hover
- Wheel/trackpad scrolling, blinking caret and keep-caret-visible behavior
- `Ctrl/Cmd+S` saves without inserting text

## Architecture

```text
egui Event
  -> SqlEditor (renders and emits SqlEditorResponse intents)
  -> EditorInteractionPolicy (decides allowed state transition)
  -> CompletionState / explicit prediction schedule
  -> version-checked buffer edit
```

The renderer does not call providers or decide application behavior. Completion sessions are tied to the document version that produced their replacement ranges. The query panel rejects stale or invalid ranges instead of attempting a best-effort edit.

## Non-goals for this slice

- Full SQL LSP protocol/client
- Multi-cursor, minimap or sticky-scroll headers
- Pixel-perfect Zed clone
- Automatic AI requests while typing
- Replacing the existing query runtime/provider command path

## Acceptance

- [ ] Normal typing and IME input insert exactly the committed text
- [ ] Alphanumeric typing does not automatically open completion
- [ ] `.` and `Ctrl/Cmd+Space` open contextual/manual completion respectively
- [ ] Completion cannot apply after its source document version changes
- [ ] Hovering/selecting a suggestion explains its kind/detail/documentation
- [ ] AI prediction requires explicit user intent and an enabled mode
- [ ] Enter/Tab/navigation ownership is deterministic while completion is open
- [ ] Wheel/trackpad scrolling and keep-caret-visible remain functional
- [ ] `Ctrl/Cmd+S` saves without modifying the SQL buffer
- [ ] UI tests, clippy, formatter and native release build pass
- [ ] Runtime visual/interaction verification is recorded separately

## Provider matrix

The editor interaction engine is provider-neutral. Schema suggestions consume the already loaded `UiSchemaSummary`; SQL dialect selection remains independently capability-driven for PostgreSQL and SQLite. No query execution/provider behavior changes in this slice.
