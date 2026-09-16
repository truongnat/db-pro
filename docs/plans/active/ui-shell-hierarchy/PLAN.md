# Shell, Activity Bar, Top Bar, and Sidebar Hierarchy — #289

## Goal

Harden the native shell, top bar, activity bar, and sidebar hierarchy so that DB Pro feels like a coherent, restrained, and stable database IDE at a glance.

## Scope

- Top bar layout and information hierarchy (active connection breadcrumb, driver badge, quick search trigger, action buttons).
- Activity Bar layout, spacing, tooltips, active states, and keyboard/mouse ergonomics.
- Sidebar header (workspace/connection dropdown, new connection, palette search, new query button).
- Sidebar resize grip, min/max limits, smooth divider lines, and clean integration with the CentralPanel.
- Ensure all components use canonical tokens (`DbProTheme`, `tokens.rs`, `Button`, `Badge`).

## Non-goals

- Rewriting internal panels of all individual activities (Explorer tree in #290, Query workspace in #291, Data Grid in #292).
- Adding complex new window frame management.

## Acceptance

- Shell hierarchy is visually crisp, consistent, and uses canonical design tokens without hardcoded magic numbers where standard tokens exist.
- Sidebar header actions, breadcrumbs, search triggers, and new query button use canonical component styles.
- Rust quality gates (`cargo check --workspace`, `cargo clippy`, `cargo test`) pass.
