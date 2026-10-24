---
schema_version: 1
id: "MEM-SRC-2ff906847b6bb808"
title: "Source mirror: docs/plans/active/ui-query-workspace/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-query-workspace/PLAN.md"
source_sha256: "2ff906847b6bb8085fc8a654e24126a909af4837b145413d0cb99e26c67d65c8"
mirror_body_sha256: "2ff906847b6bb8085fc8a654e24126a909af4837b145413d0cb99e26c67d65c8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
