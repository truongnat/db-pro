---
schema_version: 1
id: "MEM-SRC-24033d9d34afed16"
title: "Source mirror: docs/plans/active/ui-explorer-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-explorer-polish/PLAN.md"
source_sha256: "24033d9d34afed1644b042b389bf0d3d8a03f96db115d652e2411d13004cb44c"
mirror_body_sha256: "24033d9d34afed1644b042b389bf0d3d8a03f96db115d652e2411d13004cb44c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
