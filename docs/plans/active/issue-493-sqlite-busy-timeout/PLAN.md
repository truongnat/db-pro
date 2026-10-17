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
