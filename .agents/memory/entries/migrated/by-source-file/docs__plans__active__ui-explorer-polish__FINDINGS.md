---
schema_version: 1
id: "MEM-SRC-4317c20b95a72a7a"
title: "Source mirror: docs/plans/active/ui-explorer-polish/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-explorer-polish/FINDINGS.md"
source_sha256: "4317c20b95a72a7a7c39011e3719ccc26611765cec5d9b40dbd0252e5315595e"
mirror_body_sha256: "4317c20b95a72a7a7c39011e3719ccc26611765cec5d9b40dbd0252e5315595e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — UI04 Explorer Polish

## Findings

1. `crates/ui/src/explorer_folders.rs`:
   - Embedded hardcoded RGB colors (`Color32::from_rgb(5, 150, 105)` for views, `Color32::from_rgb(124, 58, 237)` for functions, `Color32::from_rgb(234, 88, 12)` for triggers).
   - Replaced with theme-aware semantic token references (`theme.success`, `theme.accent`, `theme.warning`) ensuring proper contrast across light and dark themes.

2. `crates/ui/src/explorer_view.rs`:
   - Connection context menu Delete action uses destructive styling.
