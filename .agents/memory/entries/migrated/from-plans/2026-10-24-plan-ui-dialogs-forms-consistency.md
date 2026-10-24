---
schema_version: 1
id: "MEM-PLAN-ui-dialogs-forms-consistency-419ec4dd7e62"
title: "Plan archive: Ui Dialogs Forms Consistency"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, ui-dialogs-forms-consistency]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "419ec4dd7e62cb12653e778bb7522850aa21996e49d6db5c25c9807c9fb1c00f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/ui-dialogs-forms-consistency/PLAN.md"
  - "docs/plans/active/ui-dialogs-forms-consistency/FINDINGS.md"
  - "docs/plans/active/ui-dialogs-forms-consistency/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/ui-dialogs-forms-consistency/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan — UI08 Dialog, Form, Confirmation, and Destructive-Action Consistency (#294)

## Objective
Standardize all modal dialogs, forms, confirmations, and destructive action controls across connection setup, query workflows, export/overwrite gates, and navigation inspection dialogs using canonical `Button` primitives and semantic design tokens.

## Scope
1. **Query Dialogs (`query_dialogs_view.rs`)**:
   - Destructive statement confirmation dialog.
   - Export query results dialog.
   - Export overwrite confirmation dialog.
   - Save Query As modal dialog.
   - Unsaved query close confirmation modal dialog.
2. **Connection Management (`connection_view.rs`)**:
   - Delete connection confirmation modal dialog.
   - Delete folder confirmation modal dialog.
   - Preset apply, URI import, file browsing, and SSH profile save buttons.
3. **Table Metadata Inspectors (`table_metadata_view.rs`)**:
   - Column inspector window close button.
   - Index inspector window close button.
4. **Non-Regression Test Suite (`crates/ui/src/components/mod.rs`)**:
   - Expand `primary_native_surfaces_do_not_reintroduce_raw_buttons` to cover `query_dialogs_view.rs` and `connection_view.rs`.


## Source: `FINDINGS.md`

# Findings — UI08 Dialog, Form, and Confirmation Consistency

1. Modals and confirmation dialogs in `query_dialogs_view.rs` and `connection_view.rs` were still relying on legacy helper functions (`danger_button`, `compact_button`, `secondary_button_with_icon`, `primary_button_with_icon`).
2. Standardizing to canonical `Button` with appropriate `variant` (`Destructive`, `Default`, `Secondary`, `Ghost`) and `size` (`Sm`) ensures clear visual risk hierarchy and consistent keyboard focusability.


## Source: `VERIFICATION.md`

# Verification — UI08 Dialog, Form, and Confirmation Consistency

## Test Matrix

- `cargo fmt --all -- --check` (passed)
- `cargo check --workspace` (passed)
- `cargo clippy --workspace --all-targets -- -D warnings` (passed)
- `cargo test --workspace` (passed, 483 tests)
- `cargo build --release --locked -p db-pro-native` (passed)

