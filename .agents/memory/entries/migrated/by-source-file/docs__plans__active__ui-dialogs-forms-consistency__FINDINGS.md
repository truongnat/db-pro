---
schema_version: 1
id: "MEM-SRC-67d0dfdd554c5a51"
title: "Source mirror: docs/plans/active/ui-dialogs-forms-consistency/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-dialogs-forms-consistency/FINDINGS.md"
source_sha256: "67d0dfdd554c5a5160cf00745da935f256c86c3e869e0f24ef6ae5b5e04d255f"
mirror_body_sha256: "67d0dfdd554c5a5160cf00745da935f256c86c3e869e0f24ef6ae5b5e04d255f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — UI08 Dialog, Form, and Confirmation Consistency

1. Modals and confirmation dialogs in `query_dialogs_view.rs` and `connection_view.rs` were still relying on legacy helper functions (`danger_button`, `compact_button`, `secondary_button_with_icon`, `primary_button_with_icon`).
2. Standardizing to canonical `Button` with appropriate `variant` (`Destructive`, `Default`, `Secondary`, `Ghost`) and `size` (`Sm`) ensures clear visual risk hierarchy and consistent keyboard focusability.
