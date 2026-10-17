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
