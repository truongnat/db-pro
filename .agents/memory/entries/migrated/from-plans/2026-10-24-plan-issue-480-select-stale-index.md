---
schema_version: 1
id: "MEM-PLAN-issue-480-select-stale-index-626c93f00ba8"
title: "Plan archive: Issue 480 Select Stale Index"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-480-select-stale-index]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "626c93f00ba8f32d001f5c4be22f5c72301efe99fb427cdb8a6e1581fbc2c135"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-480-select-stale-index/PLAN.md"
  - "docs/plans/active/issue-480-select-stale-index/FINDINGS.md"
  - "docs/plans/active/issue-480-select-stale-index/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-480-select-stale-index/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #480 Findings

## Evidence
- `components/select/handler.rs:112-125`: `navigate_selection` applied
  `selected.saturating_add(1) < option_count` to an already-stale index —
  with `selected=5, option_count=2` both guards failed and 5 was returned
  forever ("ArrowDown stuck at 5").
- `ui/select.rs:332-339` writes the return value back to `*self.selected`
  each frame the menu is open → self-healing once clamped here.
- Popup render is safe: options iterated by `enumerate`, icons/disabled via
  `.get(idx)` (`ui/select.rs:365-377`); trigger label via
  `selected_label` → `options.get` (`handler.rs:59-64`).

## Severity
P2 — stuck highlight, wrong "current" option perception; no data corruption.


## Source: `VERIFICATION.md`

# #480 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `stale_selection_index_clamps_when_options_shrink`

