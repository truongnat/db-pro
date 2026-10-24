---
schema_version: 1
id: "MEM-PLAN-issue-500-palette-history-finger-b9b8303a6e2f"
title: "Plan archive: Issue 500 Palette History Fingerprint"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-500-palette-history-fingerprint]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b9b8303a6e2f9b73b954279c2ea789f1ab52c90f11bf5574f01d51f93f837b0f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-500-palette-history-fingerprint/PLAN.md"
  - "docs/plans/active/issue-500-palette-history-fingerprint/FINDINGS.md"
  - "docs/plans/active/issue-500-palette-history-fingerprint/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-500-palette-history-fingerprint/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #500 — Palette history fingerprint stale + position-indexed actions

## Goal
Palette history rows must refresh when entries are pushed past the 500-entry
retention cap, and activating a history row must open the displayed entry, not
whatever happens to sit at the captured position.

## Root cause
- `search_fingerprint` (`palette_view.rs`) keyed history on
  `query_history_entries.len()` only. `record_query_history` drains the front
  once len exceeds 500, so a push at the cap keeps len == 500 → fingerprint
  unchanged → cached `SearchIndex` serves rows built from older entries.
- `query_history_items` enumerated `.iter().take(30)` (oldest-first) and stored
  the position in `PaletteAction::OpenHistoryEntry(usize)`; after a drain the
  index addresses a different entry.

## Fix
- Fingerprint key = `len:last_entry_id` (`SearchFingerprintParts.history`
  `usize` → `&str`). Every push mints a fresh `Uuid` id so pushes (with or
  without drain) always change the key; len still catches clears/restores.
  Chosen over a version counter: no state field, covers storage restores.
- Items iterate `.rev().take(30)` (newest-first, matching the history surface).
- `OpenHistoryEntry` carries the entry `id: String`; `palette_actions` resolves
  via `iter().find(|e| e.id == entry_id)` — entries already have unique UUIDs,
  so no index-translation bookkeeping is needed.

## Scope
- `app_types.rs` (enum payload), `palette_search_view.rs`,
  `palette_actions.rs`, `palette_view.rs`, `tools/search_service.rs`.
- Tests: `palette/tests/palette_view_tests.rs`,
  `query_workbench/tests/query_history_events_tests.rs`.

## Acceptance
- Push #501 drains entry #1 → `search_fingerprint` differs → index rebuilds.
- Clicking a palette history row opens the entry whose SQL was displayed.


## Source: `FINDINGS.md`

# #500 — Findings

- `UiQueryHistoryEntry.id` (`crates/ui/.../runtime_query_types.rs:64`) is a
  `Uuid::new_v4` minted per push → stable identifier already exists; no
  index-from-end translation needed.
- History is stored oldest→newest (`record_query_history` pushes, drains front
  at `QUERY_HISTORY_RETENTION = 500`); sidebar/output views already render
  `.iter().rev()`. Palette previously rendered oldest-first — also fixed here.
- `query_history_entries` is bulk-assigned in `app_storage.rs` (restore) and a
  workspace test fixture; `len:last_id` still changes on both (restore of
  identical vec legitimately keeps the cache, which is correct).
- Entries are never mutated in place after push → last-id is a sound key.
- `SearchFingerprintParts` has a single construction site
  (`palette_view.rs`); no test fixtures referenced `history`.
- `DbProApp` fields and `record_query_history`/`QueryHistoryRecord` are
  `pub(super)`/`pub(crate)` within `app` — reachable from the new sibling test
  modules under `app::palette_view::tests` and `app::query_history_events::tests`.


## Source: `VERIFICATION.md`

# #500 — Verification

Ran 2026-10-17 on branch `main` (uncommitted worktree):

- `cargo check -p db-pro-app -p db-pro-core` — clean, no warnings in touched files.
- `cargo test -p db-pro-app --lib` — 591 passed, 0 failed.
  - `palette_view::tests::fingerprint_changes_when_push_drains_history_front` ✔
  - `palette_view::tests::palette_lists_history_newest_first_with_entry_id_actions` ✔
  - `palette_view::tests::open_history_entry_action_opens_the_matching_entry_not_a_position` ✔
  - `palette_view::tests::open_history_entry_action_reports_missing_id` ✔
  - `query_history_events::tests::history_is_capped_at_retention_limit` ✔
  - `query_history_events::tests::push_that_drains_the_front_still_changes_fingerprint_key` ✔
  - `query_history_events::tests::every_entry_gets_a_unique_stable_id` ✔

Note: mid-run `cargo check`/`test` runs failed inside sibling-owned files
(`safety*.rs`, `monitoring_service.rs`) while other agents were editing; after
those settled, all gates above pass.

