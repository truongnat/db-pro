---
schema_version: 1
id: "MEM-SRC-1d74b6c8168ee88f"
title: "Source mirror: docs/plans/active/ui-schema-workbench-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-schema-workbench-polish/PLAN.md"
source_sha256: "1d74b6c8168ee88fc8f995ac574fcabbbb49bb0bd1d51318cec26c2f21bf3ccb"
mirror_body_sha256: "1d74b6c8168ee88fc8f995ac574fcabbbb49bb0bd1d51318cec26c2f21bf3ccb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
