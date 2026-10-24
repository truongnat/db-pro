---
schema_version: 1
id: "MEM-PLAN-issue-493-sqlite-busy-timeout-d3c64f05efab"
title: "Plan archive: Issue 493 Sqlite Busy Timeout"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-493-sqlite-busy-timeout]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d3c64f05efab2eab3f6a44ab8851c28211156dcacc32fbc8a28e22095e71716a"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-493-sqlite-busy-timeout/PLAN.md"
  - "docs/plans/active/issue-493-sqlite-busy-timeout/FINDINGS.md"
  - "docs/plans/active/issue-493-sqlite-busy-timeout/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-493-sqlite-busy-timeout/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — issue #493

- `rusqlite::Connection::open` defaults to `busy_timeout = 0`: `sqlite3_step`
  returns `SQLITE_BUSY` on the first lock collision.
- `actor.rs` had exactly one `Connection::open` site (`spawn`, old line 425);
  no other open site needed the fix.
- Precedent: `backup/sqlite_backup.rs` propagates `busy_timeout` errors via
  `map_err(crate::error::from_rusqlite)` — same pattern used here (stricter than
  the issue's `.ok()` suggestion, consistent with the codebase).


## Source: `VERIFICATION.md`

# Verification — issue #493

- `cargo test -p db-pro-infrastructure --lib sqlite` — **36 passed, 0 failed**
- `cargo check -p db-pro-infrastructure` — **clean** (earlier failures were
  transient sibling edits to `postgres/monitoring.rs` and `core/safety_lexer.rs`,
  unrelated to this change)
  - `spawn_sets_busy_timeout_on_actor_connection`
  - `busy_connection_waits_for_competing_writer` (real file lock contention:
    `BEGIN EXCLUSIVE` held by a second connection, released after 200ms while
    the actor's INSERT is in flight)

