---
schema_version: 1
id: "MEM-PLAN-issue-510-cursor-diagnostic-resy-1002cdca3cf0"
title: "Plan archive: Issue 510 Cursor Diagnostic Resync"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-510-cursor-diagnostic-resync]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "1002cdca3cf090da62a8faafd32f63bb38658e4d981730695749fedd5efc9617"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-510-cursor-diagnostic-resync/PLAN.md"
  - "docs/plans/active/issue-510-cursor-diagnostic-resync/FINDINGS.md"
  - "docs/plans/active/issue-510-cursor-diagnostic-resync/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-510-cursor-diagnostic-resync/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# #510 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `cursor_move_to_broken_statement_resyncs_syntax_diagnostic`

