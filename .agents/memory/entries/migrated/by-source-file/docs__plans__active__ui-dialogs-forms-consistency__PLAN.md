---
schema_version: 1
id: "MEM-SRC-0e7657a6e1c47ec1"
title: "Source mirror: docs/plans/active/ui-dialogs-forms-consistency/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-dialogs-forms-consistency/PLAN.md"
source_sha256: "0e7657a6e1c47ec1104202ce4cf9c674fbfd6be527ab48f5b707182dff536b82"
mirror_body_sha256: "0e7657a6e1c47ec1104202ce4cf9c674fbfd6be527ab48f5b707182dff536b82"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
