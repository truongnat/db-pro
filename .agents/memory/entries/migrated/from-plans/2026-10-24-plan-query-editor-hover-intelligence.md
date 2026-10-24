---
schema_version: 1
id: "MEM-PLAN-query-editor-hover-intelligence-00ae72d49ec9"
title: "Plan archive: Query Editor Hover Intelligence"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, query-editor-hover-intelligence]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "00ae72d49ec98c40f040eb7cca5b9313ae51cc2ac932aff89244a261cdcdbbbf"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/query-editor-hover-intelligence/PLAN.md"
  - "docs/plans/active/query-editor-hover-intelligence/FINDINGS.md"
  - "docs/plans/active/query-editor-hover-intelligence/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/query-editor-hover-intelligence/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Verification — Query Editor Hover Intelligence

## Quality Gates Execution Summary

| Gate | Command | Result | Evidence |
|------|---------|--------|----------|
| Rust Format | `cargo fmt --all -- --check` | **PASS** | Exit code 0, 0 format violations |
| Rust Check | `cargo check --workspace` | **PASS** | Exit code 0, 0 errors, 0 warnings |
| Rust Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | Exit code 0, 0 lints |
| Workspace Tests | `cargo test --workspace` | **PASS** | 548 passed; 0 failed; 0 ignored |
| Native Release Build | `cargo build --release -p db-pro-native` | **PASS** | Compiled in release profile |

## Unit & Integration Test Coverage

### 1. Hover FSM Delay & Lifecycle Tests (`crates/ui/src/editor/hover.rs`)
- `hover_confirmed_after_delay`: Validates that hovering on a token requires $\ge 500\text{ ms}$ before confirming.
- `hover_cleared_on_pointer_leave`: Validates that moving the mouse away immediately clears the confirmed/pending token.
- `hover_resets_on_different_token`: Validates that moving across different tokens resets the delay timer and avoids stale popup rendering.
- `pending_repaint_returns_remaining_time`: Validates that egui is requested to repaint at the exact time the delay elapses.

### 2. Rich Hover Tests (`crates/ui/src/query/intelligence.rs`)
- `rich_hover_resolves_table_columns_and_foreign_keys`: Resolves a table hover card with full column list, data types, primary key flags, foreign key relationships, and row count.
- `rich_hover_resolves_qualified_column_with_fk_target`: Resolves a column hover card showing data type, primary key flag, parent table, and target foreign key reference `public.users(id)`.
- `rich_hover_keyword_returns_documentation_and_dialect_note`: Resolves SQL keywords (e.g. `SELECT`, `RETURNING`, `PRAGMA`) with dialect note (`PostgreSQL` / `SQLite`), explanation, and example SQL snippet.
- `signature_help_prefers_introspected_active_schema_function`: Resolves active schema function signature and parameter context.
- `sqlite_and_postgres_builtins_remain_provider_specific`: Ensures dialect-specific functions are only available on matching database targets.

## Provider Matrix Coverage

| Feature | PostgreSQL | SQLite | Verification Method |
|---------|------------|--------|---------------------|
| Table hover (columns, row count, FKs) | Supported | Supported | Unit tests + schema summary model |
| Column hover (type, nullability, PK, FK) | Supported | Supported | Unit tests |
| Keyword hover (DML, DDL, clauses, examples) | Supported | Supported | Unit tests with PostgreSQL & SQLite dialects |
| Function signature hover | Supported (schema + built-ins) | Supported (built-ins) | Unit tests |

