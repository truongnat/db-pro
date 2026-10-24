---
schema_version: 1
id: "MEM-SRC-0c65028cd32e12ea"
title: "Source mirror: docs/plans/active/issue-480-select-stale-index/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-480-select-stale-index/PLAN.md"
source_sha256: "0c65028cd32e12eacf4f21d8025573f4ddce6ab160dba33759d60dc058fa16b2"
mirror_body_sha256: "0c65028cd32e12eacf4f21d8025573f4ddce6ab160dba33759d60dc058fa16b2"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #480 — Select dropdown stale index when options shrink

## Goal
`navigate_selection` must clamp `selected` into `0..option_count` before
applying arrow keys so a stale index (options shrank while the menu stayed
open) lands on the last valid option instead of staying out of bounds.

## Scope
- `crates/ui/src/components/select/handler.rs` — clamp `selected` via
  `.min(option_count.saturating_sub(1))` at the top of `navigate_selection`.
- Popup render (`ui/select.rs:365-385`) already reads via
  `options.iter().enumerate()` / `.get()` and `selected_label` uses
  `options.get` — verified safe, no change needed.

## Non-goals
- No wrap-around navigation (existing contract: arrows clamp at ends).
- Signature stays `usize` — callers treat the select as always-indexed.

## Tests
`tests/handler_tests.rs::stale_selection_index_clamps_when_options_shrink`:
5→2 shrink with ArrowDown → 1; idle clamp; ArrowUp from stale index; empty
list → 0.
