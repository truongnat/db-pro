# Query Workspace Visual Hierarchy and Output Composition — #291

## Goal

Refine the Query workspace (editor header, transaction bar, search bar, and output tabs) into a clean, restrained, high-density developer tool layout with canonical button controls and unified design tokens.

## Scope

- Query header action buttons (`Run`, `Stop`, `Builder`, `More actions`) migrated to canonical `Button` primitives with proper variants/sizes.
- Query search bar controls (`ChevronUp`, `ChevronDown`, `Close`) migrated to canonical `Button` primitives.
- Built-in SQL snippet inserters migrated to canonical `Button` primitives.
- Non-regression tests updated to ensure query view surfaces do not re-introduce raw/ad-hoc controls.

## Non-goals

- Rewriting text buffer/cursor logic in `crates/ui/src/editor/`.
- Changing query execution worker architecture.

## Acceptance

- Query header and toolbar buttons use canonical `Button` components.
- Output tab bar and search bar have clean spacing aligned with design tokens.
- All workspace Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`) pass.
