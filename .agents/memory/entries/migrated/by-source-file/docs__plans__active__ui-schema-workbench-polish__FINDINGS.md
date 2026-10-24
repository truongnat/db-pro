---
schema_version: 1
id: "MEM-SRC-12fc7b0ddb1d7a12"
title: "Source mirror: docs/plans/active/ui-schema-workbench-polish/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-schema-workbench-polish/FINDINGS.md"
source_sha256: "12fc7b0ddb1d7a12d6c714a55852c8f11a3bdfa0d7ace3e0b92d51925728f603"
mirror_body_sha256: "12fc7b0ddb1d7a12d6c714a55852c8f11a3bdfa0d7ace3e0b92d51925728f603"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — UI07 Table/Object/Schema Workbench Quality

1. Schema workbench and object workbench had remaining usages of legacy button helpers (`secondary_button`, `compact_button_with_icon`, `compact_icon_button`, `danger_button`, `ghost_button_with_icon`).
2. Standardizing to canonical `Button` struct with explicit `variant`, `size`, `icon`, and `tooltip` makes the visual layout consistent with the rest of the application design system.
