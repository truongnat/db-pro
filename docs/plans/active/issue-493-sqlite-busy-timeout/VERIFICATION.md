# Verification — issue #493

- `cargo test -p db-pro-infrastructure --lib sqlite` — **36 passed, 0 failed**
- `cargo check -p db-pro-infrastructure` — **clean** (earlier failures were
  transient sibling edits to `postgres/monitoring.rs` and `core/safety_lexer.rs`,
  unrelated to this change)
  - `spawn_sets_busy_timeout_on_actor_connection`
  - `busy_connection_waits_for_competing_writer` (real file lock contention:
    `BEGIN EXCLUSIVE` held by a second connection, released after 200ms while
    the actor's INSERT is in flight)
