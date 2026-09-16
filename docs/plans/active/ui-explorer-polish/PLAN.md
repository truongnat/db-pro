# Explorer Tree, Object Navigation, Search, and Context-Menu Polish — #290

## Goal

Make the Database Explorer feel like a coherent, responsive database IDE navigator with standardized semantic styling, canonical search input, and risk-aware context menus.

## Scope

- Consolidate category folder colors (Views, Functions, Triggers) in `explorer_folders.rs` behind theme/semantic tokens.
- Ensure context menus across Explorer rows use canonical `ctx_menu_item` / `context_action_menu` with proper destructive styling where appropriate.
- Verify search input filtering, empty states, and keyboard/mouse ergonomics.
- Ensure all quality gates and UI tests pass.

## Non-goals

- Altering database introspection or query execution backend.
- Changing tree data models or RPCs.

## Acceptance

- No hardcoded ad-hoc raw button/control drift remains in Explorer sub-views.
- Category folder icons and colors are cohesive with `DbProTheme`.
- Context menus use standardized shortcuts, icons, and risk highlights.
- `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test` pass.
