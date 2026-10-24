---
schema_version: 1
id: "MEM-SRC-86ae8cabcebf844b"
title: "Source mirror: docs/plans/active/issue-510-cursor-diagnostic-resync/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-510-cursor-diagnostic-resync/FINDINGS.md"
source_sha256: "86ae8cabcebf844beec425ec950687983b9e7f0c032c4adf359163c1c9ba65e9"
mirror_body_sha256: "86ae8cabcebf844beec425ec950687983b9e7f0c032c4adf359163c1c9ba65e9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #510 Findings

## Evidence
- `query/query_document.rs:266-281`: `reanalyze` returns early when
  `analysis.version == buffer.version()` — but the statement syntax
  diagnostic depends on `cursor.offset`, which changes without a version
  bump.
- Only call site gated on text change:
  `crates/app/src/query_workbench/query_editor_surface_view.rs:198-202`
  (`if response.changed`). Pure caret moves never resynced → diagnostic
  stayed on the statement the cursor left.

## Decision
`diagnostic_cursor: Option<usize>` on the document +
`sync_cursor_diagnostic(dialect)` called unconditionally after each editor
response; no-op guard keeps it free when `reanalyze` already synced or the
cursor never moved. Re-parses only the semicolon-bounded statement — the
cheapest correct trigger; tracking the statement *range* would still parse
to decide.

## Severity
P2 — stale squiggle on a valid statement / missing squiggle on a broken one.
