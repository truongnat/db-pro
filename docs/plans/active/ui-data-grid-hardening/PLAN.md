# Data and Result Grid Interaction, Density, and Editing Hardening — #292

## Goal

Standardize the Data Grid and Result Grid toolbar actions, pagination controls, staged mutation actions, and export buttons to use canonical components and design tokens.

## Scope

- Result grid toolbar actions (Copy Cell, Copy Row, CSV, JSON, Record Inspector toggle, Cell Inspector) in `result_grid_view.rs`.
- Table data toolbar mutation controls (Add Row, Apply, Discard, Reload Row, Resolve Conflict, Retry) in `table_editor_view.rs`.
- Table data pagination buttons (First, Previous, Next, Last) in `table_editor_view.rs`.
- Expand non-regression tests in `crates/ui/src/components/mod.rs` to prevent raw/ad-hoc button regressions across grid surfaces.

## Non-goals

- Altering grid cell virtual projection algorithms or performance cache logic in `result_grid_view.rs` / `result_grid.rs`.
- Changing backend transaction mutation execution semantics.

## Acceptance

- Grid and Table toolbar actions use canonical `Button` primitives with unified sizes and variants.
- Pagination buttons use canonical `Button` primitives with proper `enabled` binding.
- All workspace Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`) pass.
