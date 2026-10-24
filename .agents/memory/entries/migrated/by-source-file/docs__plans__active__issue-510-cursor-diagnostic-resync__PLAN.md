---
schema_version: 1
id: "MEM-SRC-35539214ee4cfe78"
title: "Source mirror: docs/plans/active/issue-510-cursor-diagnostic-resync/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-510-cursor-diagnostic-resync/PLAN.md"
source_sha256: "35539214ee4cfe78fa621620631859d5ab5c9a898bd2ae9ab035aac09775d40f"
mirror_body_sha256: "35539214ee4cfe78fa621620631859d5ab5c9a898bd2ae9ab035aac09775d40f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #510 — Statement syntax diagnostic goes stale on cursor move

## Goal
Moving the cursor into a different statement must re-sync the
statement-under-cursor syntax diagnostic even when `buffer.version()` is
unchanged.

## Root cause
`QueryDocument::reanalyze` early-returns when `analysis.version ==
buffer.version()`, and the only caller (`query_editor_surface_view.rs:198`)
invokes it on `response.changed` — a pure cursor move never reached
`sync_statement_syntax_diagnostic`, so the diagnostic stayed pinned to the
statement the cursor left.

## Fix
- `crates/ui/src/query/query_document.rs`: new `diagnostic_cursor:
  Option<usize>` field records the cursor offset the diagnostic was synced
  for; `reanalyze` sets it; new `pub fn sync_cursor_diagnostic(dialect)`
  re-runs `sync_statement_syntax_diagnostic` only when the offset changed
  (cheapest correct trigger — parsing the statement is bounded).
- `crates/app/src/query_workbench/query_editor_surface_view.rs`: call
  `doc.sync_cursor_diagnostic(self.dialect)` after every editor response —
  internally a no-op when `reanalyze` just ran or the cursor is unchanged.

## Tests
`query/tests/query_document_tests.rs::cursor_move_to_broken_statement_
resyncs_syntax_diagnostic`: move into `SELEC` → syntax diagnostic appears;
move back → clears; repeat call is a no-op.
