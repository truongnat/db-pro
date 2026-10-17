# Checklist — issue #493

- [x] `conn.busy_timeout(Duration::from_secs(10))` in `SqliteActor::spawn`, error propagated
- [x] No `journal_mode=WAL` added
- [x] Test: `spawn_sets_busy_timeout_on_actor_connection` (reads `PRAGMA busy_timeout` through the actor)
- [x] Test: `busy_connection_waits_for_competing_writer` (EXCLUSIVE lock held 200ms; actor insert waits and succeeds)

- [x] `cargo test -p db-pro-infrastructure --lib sqlite` — 36 passed, 0 failed
- [x] `cargo check -p db-pro-infrastructure` — clean

