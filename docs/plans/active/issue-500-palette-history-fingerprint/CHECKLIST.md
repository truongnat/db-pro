# #500 — Checklist

- [x] `OpenHistoryEntry(usize)` → `OpenHistoryEntry(String)` (entry id)
- [x] `query_history_items`: `.rev().take(30)`, action carries `entry.id`
- [x] `palette_actions`: resolve entry by id; "no longer available" fallback kept
- [x] `SearchFingerprintParts.history: usize` → `&str`; caller passes
      `"{len}:{last_id}"`
- [x] Test: fingerprint changes on push+drain at cap
- [x] Test: newest-first ordering + id-carrying actions
- [x] `cargo check -p db-pro-app -p db-pro-core` — clean
- [x] `cargo test -p db-pro-app --lib` — 591 passed, 0 failed
