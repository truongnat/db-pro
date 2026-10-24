---
schema_version: 1
id: "MEM-SRC-53725ac5d68f46b3"
title: "Source mirror: docs/plans/active/issue-493-sqlite-busy-timeout/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-493-sqlite-busy-timeout/CHECKLIST.md"
source_sha256: "53725ac5d68f46b3e89190844e317423641d160ce98c5ec899fcd7e668f866d3"
mirror_body_sha256: "53725ac5d68f46b3e89190844e317423641d160ce98c5ec899fcd7e668f866d3"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — issue #493

- [x] `conn.busy_timeout(Duration::from_secs(10))` in `SqliteActor::spawn`, error propagated
- [x] No `journal_mode=WAL` added
- [x] Test: `spawn_sets_busy_timeout_on_actor_connection` (reads `PRAGMA busy_timeout` through the actor)
- [x] Test: `busy_connection_waits_for_competing_writer` (EXCLUSIVE lock held 200ms; actor insert waits and succeeds)

- [x] `cargo test -p db-pro-infrastructure --lib sqlite` — 36 passed, 0 failed
- [x] `cargo check -p db-pro-infrastructure` — clean

