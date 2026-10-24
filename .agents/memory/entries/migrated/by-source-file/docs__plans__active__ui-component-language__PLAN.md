---
schema_version: 1
id: "MEM-SRC-7512082dfcb29586"
title: "Source mirror: docs/plans/active/ui-component-language/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-component-language/PLAN.md"
source_sha256: "7512082dfcb295861a626de3b6cbf162c700fdd19322adfeda637280511b0ca5"
mirror_body_sha256: "7512082dfcb295861a626de3b6cbf162c700fdd19322adfeda637280511b0ca5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# UI Component Language — #288 slice

## Goal

Reduce visible control drift on two primary native surfaces by routing their
actions and search through the existing canonical egui components.

## Scope

- Value inspector mode switching and actions in `result_grid_edit.rs`;
- Explorer search and refresh context action in `explorer_view.rs`;
- no new UI framework, token system, or component abstraction.

## Non-goals

- full `legacy.rs` removal;
- every native surface in #288;
- #287 runtime screenshot matrix and light/dark viewport qualification;
- changes to data editing semantics or provider behavior.

## Acceptance

- inspector modes use `SegmentedTabs`;
- inspector actions use canonical `Button` variants/sizes;
- Explorer search uses `SearchInput` and refresh menu uses `ctx_menu_item`;
- existing behavior remains unchanged;
- Rust quality gates pass; UI runtime evidence remains explicitly pending per #287.
