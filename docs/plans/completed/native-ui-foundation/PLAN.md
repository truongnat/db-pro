# Native UI Foundation

## Status

`COMPLETED`

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

- Native SQL editor parity with Monaco.
- Large-schema ER rendering and packaging.
- MCP/tool execution, autonomous database mutations, and persistent API-key management UI.
- Removing React/Tauri.

## Next slice

The native database workspace and shared runtime boundary are implemented and verified. Native Data editing preserves composite primary-key identity and typed key values across the UI/runtime bridge, NUMERIC/DECIMAL fields use exact precision-aware validation, ER opens large schemas in a search-first bounded view, views expose definition and bounded Data browsing, and PostgreSQL functions/procedures have a definition workspace. Tauri commands consume `DbProRuntime` facades. Monaco parity, MCP/autonomous writes, large-schema packaging and deeper Tauri lifecycle work remain explicitly out of scope for this plan.
