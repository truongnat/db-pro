---
schema_version: 1
id: "MEM-PLAN-ui-schema-workbench-polish-d456f3aae20d"
title: "Plan archive: Ui Schema Workbench Polish"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-schema-workbench-polish]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d456f3aae20d8ba249d444df2d1c9b0275614d7b932c80e19d880dab888e26ce"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-schema-workbench-polish/PLAN.md"
  - "docs/plans/active/ui-schema-workbench-polish/FINDINGS.md"
  - "docs/plans/active/ui-schema-workbench-polish/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-schema-workbench-polish/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — UI07 Table/Object/Schema Workbench Quality (#293)

## Objective
Standardize Table, Object, and Schema Workbench hierarchy, forms, DDL preview, metadata grids, and mutation actions using canonical design tokens and `Button` components from `crates/ui/src/components/`.

## Scope
1. **Schema Workbench Form (`schema_workbench_form.rs`)**:
   - Replace legacy `secondary_button` with canonical `Button::new(self.theme).text(...).variant(ButtonVariant::Secondary).size(ButtonSize::Sm)`.
2. **Schema Object Workspace (`schema_object_view.rs`)**:
   - Replace legacy `primary_button_with_icon`, `secondary_button_with_icon`, `danger_button`, `ghost_button_with_icon` with canonical `Button` primitives.
3. **Table DDL View (`table_ddl_view.rs`)**:
   - Replace legacy `compact_button_with_icon` with canonical `Button` primitives.
4. **Table Structure / Metadata View (`table_metadata_view.rs`)**:
   - Replace legacy `compact_icon_button` for search clearing and navigation links with canonical `Button` primitives.
5. **Table Workspace / Welcome Screen (`table_view.rs`)**:
   - Replace legacy `primary_button_with_icon`, `secondary_button_with_icon`, `compact_button_with_icon`, `compact_icon_button` with canonical `Button` primitives.
6. **Non-Regression Coverage (`crates/ui/src/components/mod.rs`)**:
   - Extend `primary_native_surfaces_do_not_reintroduce_raw_buttons` to assert no raw buttons in all schema and object views.


## Source: `FINDINGS.md`

# Findings — UI07 Table/Object/Schema Workbench Quality

1. Schema workbench and object workbench had remaining usages of legacy button helpers (`secondary_button`, `compact_button_with_icon`, `compact_icon_button`, `danger_button`, `ghost_button_with_icon`).
2. Standardizing to canonical `Button` struct with explicit `variant`, `size`, `icon`, and `tooltip` makes the visual layout consistent with the rest of the application design system.


## Source: `VERIFICATION.md`

# Verification — UI07 Table/Object/Schema Workbench Quality

## Test Matrix

- `cargo fmt --all -- --check` (passed)
- `cargo check --workspace` (passed)
- `cargo clippy --workspace --all-targets -- -D warnings` (passed)
- `cargo test --workspace` (passed, 483 tests)
- `cargo build --release --locked -p db-pro-native` (passed)

