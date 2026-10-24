---
schema_version: 1
id: "MEM-SRC-3c60ff95d00cee97"
title: "Source mirror: docs/plans/active/issue-496-497-fk-dropdown/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-496-497-fk-dropdown/FINDINGS.md"
source_sha256: "3c60ff95d00cee97c15e6db4c1831a4b098be15006f0e907ececb68c33a0ef52"
mirror_body_sha256: "3c60ff95d00cee97c15e6db4c1831a4b098be15006f0e907ececb68c33a0ef52"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #496 + #497

- `foreign_key_for_column` is shared by the dropdown and `open_referenced_row`;
  gating it directly would have broken composite-FK navigation. The 1:1 gate
  lives in `foreign_key_lookup_sql` (the dropdown path) via `dropdown_applies`.
- `lookup_sql` had no `ORDER BY`/`LIMIT`; the pending lookup runs as a normal
  `UiCommand::RunQuery`, so the cap must live in the SQL text.
- Truncation indicator: `apply_foreign_key_lookup` sets a runtime message
  ("Foreign key values truncated to the first 500 rows") when
  `result.rows.len() >= FK_LOOKUP_LIMIT` — keeps the dropdown functional.
- `referenced_row_filters` unchanged: composite keys still navigate.
