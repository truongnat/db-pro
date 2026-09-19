# Query Editor — Hover Intelligence & Zed-like Polish

## Goal

Deliver a rich, polished SQL editor experience comparable to Zed / DBeaver's query tab:

- **Hover tooltips** that explain every meaningful token — keywords, tables, columns, functions,
  types — with full schema context (column list, FK references, row count, constraints).
- **Hover state machine** with a proper 500 ms delay so tooltips don't flicker on mouse movement.
- **Symbol highlight** — hovering a table/column/function highlights all other occurrences in the
  visible viewport.
- **Rich keyword reference** — every major SQL keyword shows syntax, semantics, and dialect note.
- **Hover fires whether the editor is focused or not** — the current guard `response.focused &&`
  blocks hover when the editor loses focus, which is wrong.
- **Interactable hover card** — the popup must be interactable so the user can copy text.
- **Completion popup polish** — filtered matching (bold the matching prefix), smoother keyboard
  navigation, scroll-into-view on keyboard select.
- **Gutter decorations** — diagnostic squiggles drawn under tokens, not just marked in the gutter.

## Scope

### `crates/ui/src/editor/`

| File | Change |
|------|--------|
| `hover.rs` | Add `HoverState` with delay FSM; track `pending_hover_token`, `confirmed_hover_token`, `hover_started_at` |
| `renderer.rs` | Emit hover state machine output instead of raw `hovered_token`; add token-highlight rectangles for confirmed hover |
| `completion.rs` | Add `filter_text` scoring for matched-prefix bolding |

### `crates/ui/src/query/`

| File | Change |
|------|--------|
| `intelligence.rs` | Expand keyword table to 50+ keywords; add rich table help (column list, FK); add constraint display |

### `crates/ui/src/query_editor_panel.rs`

| Change |
|--------|
| Remove `response.focused &&` guard from hover code path |
| Use confirmed (delayed) hover token instead of raw hovered_token |
| Render interactable hover card (remove `interactable(false)`) |
| Draw matched-token highlights in the editor viewport |

## Architecture

```text
MouseMove
  → renderer detects hovered_token (raw, every frame)
  → SqlEditorResponse { hovered_token: Option<HoveredSqlToken> }

HoverState (per QueryDocument)
  .pending = Some(token, start_time)   ← set when raw hover arrives
  .confirmed = Some(token)             ← promoted after 500 ms
  .cleared when cursor leaves token

query_editor_panel
  → if confirmed hover token & no signature help
      → draw_hover_popup (interactable)
      → draw_token_highlights (non-interactable bg pass)
```

The renderer stays stateless — all FSM state lives in `QueryDocument`.

## Non-goals

- Full LSP protocol / language server
- Go-to-definition navigation
- Multi-cursor
- Rename refactoring
- Minimap

## Acceptance

- [ ] Hovering a table name shows full column list, row count, FK references
- [ ] Hovering a column shows type, nullable, PK/FK flags, parent table
- [ ] Hovering a function shows signature, parameter docs, volatility
- [ ] Hovering a keyword shows SQL reference doc (50+ keywords)
- [ ] Tooltip appears after 500 ms hover delay (not immediately)
- [ ] Moving mouse away dismisses tooltip
- [ ] Tooltip fires whether editor is focused or not
- [ ] Tooltip text is selectable / copyable (interactable popup)
- [ ] Token highlights visible for confirmed hover symbol
- [ ] Completion filter bolding shows matched prefix
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo build --release --locked -p db-pro-native` passes

## Provider matrix

| Feature | PostgreSQL | SQLite |
|---------|-----------|--------|
| Table hover (name, row count, columns) | ✅ | ✅ |
| Column hover (type, nullable, PK) | ✅ | ✅ |
| FK references in table hover | ✅ | ✅ (if FK present in schema) |
| Function signature hover | ✅ | ❌ (no UiFunctionSummary from SQLite) |
| Keyword hover | ✅ | ✅ (dialect-specific notes) |
