---
schema_version: 1
id: "MEM-SRC-7122d01a1452eb2a"
title: "Source mirror: docs/plans/active/issue-500-palette-history-fingerprint/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-500-palette-history-fingerprint/CHECKLIST.md"
source_sha256: "7122d01a1452eb2a561fc0d2bb76ebd0190a70fa57efdc2c4c5f88d1cccd0352"
mirror_body_sha256: "7122d01a1452eb2a561fc0d2bb76ebd0190a70fa57efdc2c4c5f88d1cccd0352"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
