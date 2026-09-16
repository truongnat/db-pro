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
