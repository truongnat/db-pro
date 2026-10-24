---
schema_version: 1
id: "MEM-SRC-10c8de77502afefd"
title: "Source mirror: docs/plans/active/issue-493-sqlite-busy-timeout/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-493-sqlite-busy-timeout/PLAN.md"
source_sha256: "10c8de77502afefde9701e0d18b4efcd0af919c1693d1a0cc97645631612e4b8"
mirror_body_sha256: "10c8de77502afefde9701e0d18b4efcd0af919c1693d1a0cc97645631612e4b8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #493 — SqliteActor missing busy_timeout

## Problem
`SqliteActor::spawn` opened the `rusqlite::Connection` with no `busy_timeout`. Any
second connection to the same database file that hits a lock gets `SQLITE_BUSY`
immediately instead of waiting.

## Fix
In `crates/infrastructure/src/sqlite/actor.rs`, `SqliteActor::spawn`, after
`Connection::open`, call
`conn.busy_timeout(std::time::Duration::from_secs(10)).map_err(crate::error::from_rusqlite)?;`
Same propagation style as `backup/sqlite_backup.rs` (which uses 5s for a cold
backup path; the long-lived user connection gets 10s per the issue).

Deliberately NOT setting `journal_mode=WAL` — explicitly out of scope per issue.

Only `Connection::open` call site in `actor.rs` is `spawn` (the test helper
`open_in_memory` in tests is in-memory, lock contention impossible there, left as is).
