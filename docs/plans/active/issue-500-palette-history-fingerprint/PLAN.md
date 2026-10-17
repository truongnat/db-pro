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
