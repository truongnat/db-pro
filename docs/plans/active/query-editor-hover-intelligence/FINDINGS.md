# Findings — Query Editor Hover Intelligence

## Evidence

### F1 — Hover guard removed (P1 fix)

**Defect:** `query_editor_panel.rs` (pre-patch) line 732:
```rust
if response.focused && !doc.completion.is_open {
    // ... hover dispatch
}
```
This meant hover only fired when the editor had keyboard focus. If the user moved focus to
the result pane, all hover tooltips disappeared — even though the mouse was still over SQL text.

**Fix:** Separated signature help (still requires focus, correct) from hover (now fires regardless
of focus state). The hover FSM is updated every frame with `response.hovered_token` and the
confirmed token is rendered independently.

### F2 — Hover flickered immediately (P2 fix)

**Defect:** The old `draw_symbol_hover()` was called every frame the mouse was over a token —
no delay, causing the popup to flash when the mouse moved across text.

**Fix:** `HoverState` FSM in `editor/hover.rs` with 500 ms `HOVER_DELAY`. The popup only
appears after the pointer has rested on the same token for ≥500 ms. `pending_repaint_after()`
schedules a minimal repaint so the popup appears without mouse movement.

### F3 — Hover popup was non-interactable (P2 fix)

**Defect:** Old popup used `.interactable(false)` — text was not selectable or copyable.

**Fix:** New `draw_rich_hover_popup` uses `.interactable(true)`.

### F4 — Hover content was sparse (P2)

**Defect:** Table hover showed only `title`, `kind`, `detail`, `documentation` — no column list,
no FK references, no row count.

**Fix:** `RichHoverHelp::Table` card shows: badge, qualified name, row count, up to 20 columns
(with PK/FK/nullable badges), FK list.

### F5 — Keyword help covered only 8 keywords (P2)

**Defect:** `keyword_help()` returned `None` for anything other than SELECT, FROM, WHERE, JOIN,
GROUP BY, HAVING, ORDER BY, LIMIT, RETURNING, PRAGMA.

**Fix:** `rich_keyword_help()` covers ~50 keywords across DML, DDL, transaction, subquery/CTE,
predicates, window functions, CASE expressions, and dialect-specific forms (PG/SQLite).

### F6 — No structured hover types for column hover (P2)

**Defect:** Column hover was bundled into `SqlSymbolHelp.detail` as a plain string.

**Fix:** `RichHoverHelp::Column` shows type badge, nullable/PK chips, parent table, FK target.

## Severity

| Finding | P |
|---------|---|
| F1 (hover lost on focus change) | P1 |
| F2 (hover flicker) | P2 |
| F3 (non-interactable popup) | P2 |
| F4 (sparse table info) | P2 |
| F5 (sparse keyword help) | P2 |
| F6 (no column structure) | P2 |

## Scope

Files changed:
- `crates/ui/src/editor/hover.rs` — new HoverState FSM
- `crates/ui/src/editor/mod.rs` — export HoverState
- `crates/ui/src/query/intelligence.rs` — RichHoverHelp enum, rich_hover, rich_keyword_help, rich_table_hover, rich_column_hover
- `crates/ui/src/query/mod.rs` — export new types
- `crates/ui/src/query/query_document.rs` — hover_state field
- `crates/ui/src/query_editor_panel.rs` — new hover popup renderer, FSM dispatch

## Provider impact

- PostgreSQL: all hover features apply
- SQLite: table/column/keyword hover apply; function hover limited (SQLite introspection doesn't
  produce `UiFunctionSummary` entries, so only builtin functions like substr/group_concat show)

## Runtime testability

- `cargo test --workspace` — includes `hover_confirmed_after_delay`, `hover_cleared_on_pointer_leave`,
  `hover_resets_on_different_token`, `pending_repaint_returns_remaining_time`
- Visual verification requires the native binary running with a live schema
