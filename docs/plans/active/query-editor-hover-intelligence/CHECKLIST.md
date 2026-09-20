# Checklist — Query Editor Hover Intelligence

## Phase 1: Hover state machine (editor layer)

- [x] Define `HoverState` struct in `editor/hover.rs` with FSM fields
- [x] Add `hover_state: HoverState` to `QueryDocument`
- [x] Thread `hovered_token` from `SqlEditorResponse` through hover FSM in `query_editor_panel`
- [x] Confirm hover after 500 ms delay; clear when pointer leaves token
- [x] Expose `confirmed_hover_token` from hover state for downstream use

## Phase 2: Rich symbol help (intelligence layer)

- [x] Expand `keyword_help` in `intelligence.rs` to 50+ SQL keywords with documentation, examples, and dialect notes
- [x] Add `rich_table_hover` function: column list, FK references, row count, PK/FK badges
- [x] Add `rich_column_hover` function: type, nullable, PK, FK target
- [x] Add helper to format foreign keys and columns for hover display
- [x] Add `SchemaSymbolIndex::rich_hover` that routes to table/column/function/keyword help

## Phase 3: Hover popup rendering (query_editor_panel)

- [x] Remove `response.focused &&` guard from hover code path so hover works unfocused
- [x] Use `confirmed_hover_token` from FSM instead of immediate raw `hovered_token`
- [x] Make hover popup interactable (`interactable(true)`) allowing text selection/copying
- [x] Draw full table card: table/view badge, row count, column table, FK list
- [x] Draw full column card: type badge, nullable/PK/FK flags, parent table, reference target
- [x] Draw keyword card: SQL badge, dialect note, documentation, code example
- [x] Clamp hover popup to screen edges with margin (`clamp_popup_to_screen`)

## Phase 4: Quality gates & automated verification

- [x] `cargo fmt --all -- --check` passes
- [x] `cargo check --workspace` passes  
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [x] `cargo test --workspace` passes (548 passed, 0 failed, 0 ignored)
- [x] `cargo build --release -p db-pro-native` passes

## Phase 5: Documentation & plan

- [x] Fill FINDINGS.md with evidence
- [x] Fill VERIFICATION.md
- [x] Update STATUS.md
