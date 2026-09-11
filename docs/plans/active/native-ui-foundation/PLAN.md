# Native UI Foundation

## Status

`IMPLEMENTING`

## Goal

Create the first runnable egui/eframe native base and the first real database workspace slice without changing the existing React/Tauri product surface.

## Scope

- Add `db-pro-ui` library crate.
- Add `db-pro-native` binary crate.
- Establish DB Pro visual tokens and a light-first Codex-inspired theme.
- Implement the first shell: activity bar, sidebar, topbar, statusbar, workspace tabs, welcome view, query workspace and Agent preview panel.
- Connect the native shell to the shared Rust runtime through a typed task bridge.
- Define a provider boundary for the native Agent preview, with offline drafts as the safe fallback.
- Add an optional Rust Codex Responses provider with an async worker path and redacted credential handling.
- Add Codex-style Quick Open and Command Palette navigation for the native workspace.
- Provide the first DBeaver-shaped table workspace with Structure, Data and DDL views.
- Add a bounded native ER diagram canvas for the active schema with table columns and foreign-key relationships.

## Out of scope

- Full Tauri DTO facade migration.
- Native SQL editor parity with Monaco.
- Large-schema ER rendering and packaging.
- MCP/tool execution, autonomous database mutations, and persistent API-key management UI.
- Removing React/Tauri.

## Next slice

Continue the native database workspace with provider-specific pagination tests and migration of Tauri command methods to the typed DTO facade adapters. Native Data editing now preserves composite primary-key identity and typed key values across the UI/runtime bridge, and NUMERIC/DECIMAL fields use exact precision-aware validation. The first ER canvas is intentionally bounded to five tables, eight columns and six relationships; large-schema layout, persistence and richer diagram interactions remain follow-up work. Views expose both their definition and bounded Data browsing, while PostgreSQL functions/procedures now have a definition workspace through the same schema-object path.
