# Findings — issue #493

- `rusqlite::Connection::open` defaults to `busy_timeout = 0`: `sqlite3_step`
  returns `SQLITE_BUSY` on the first lock collision.
- `actor.rs` had exactly one `Connection::open` site (`spawn`, old line 425);
  no other open site needed the fix.
- Precedent: `backup/sqlite_backup.rs` propagates `busy_timeout` errors via
  `map_err(crate::error::from_rusqlite)` — same pattern used here (stricter than
  the issue's `.ok()` suggestion, consistent with the codebase).
